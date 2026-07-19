use super::path::PathTrie;
// use super::solver;
use super::strategy::*;
use super::*;
use crate::cli::args::{self, CliArgs, Heuristic};
use crate::cli::pipeline::OutputWriter;
use crate::interp::config::RunnerConfig;

pub struct Executor<'prog, 'io> {
    prog: &'prog Program,
    output: &'io mut OutputWriter,
    config: RunnerConfig,
    path_trie: PathTrie,
    ansr_cnt: usize,
    rng: rngs::ThreadRng,
    solver: Box<dyn solver::common::PrimSolver>,
}

impl<'prog, 'io> Executor<'prog, 'io> {
    pub fn new(
        prog: &'prog Program,
        output: &'io mut OutputWriter,
        args: &CliArgs,
    ) -> Executor<'prog, 'io> {
        let solver: Box<dyn solver::common::PrimSolver> = match args.solver {
            args::Solver::Z3 => Box::new(super::solver::smtlib::SmtLibSolver::new(
                super::solver::smtlib::SolverBackend::Z3,
            )),
            args::Solver::CVC5 => Box::new(super::solver::smtlib::SmtLibSolver::new(
                super::solver::smtlib::SolverBackend::CVC5,
            )),
            args::Solver::Encode => Box::new(super::solver::no_smt::NoSmtSolver::new()),
        };

        let config = RunnerConfig::new(args);

        Executor {
            prog,
            output,
            config,
            path_trie: PathTrie::new(),
            ansr_cnt: 0,
            rng: rand::rng(),
            solver,
        }
    }

    pub fn config_set_param(&mut self, param: &QueryParam) {
        self.config.set_param(param);
    }

    pub fn run_step_loop(&mut self, pred: Ident) -> usize {
        while !self.run_step(pred) {
            if self.ansr_cnt >= self.config.answer_limit {
                break;
            }
        }
        return self.ansr_cnt;
    }

    fn run_step(&mut self, pred: Ident) -> bool {
        let path = self.path_trie.random_unexpaned_path(&mut self.rng);
        let brch = get_branch_from_path(self.prog, pred, &path);

        if self.is_solved(&brch) {
            self.solve_answer(&brch);
            return self.path_trie.remove_trie(&path);
        }

        let res = branch_split(self.prog, &brch, self.config.heuristic);
        if res.is_empty() {
            return self.path_trie.remove_trie(&path);
        } else {
            let mut subtrie = PathTrie::new();
            for (_brch, subpath) in res {
                subtrie.insert(&subpath);
            }
            self.path_trie.expand_trie(&path, subtrie);
            return false;
        }
    }

    fn is_solved(&self, brch: &Branch) -> bool {
        if !brch.calls.is_empty() {
            return false;
        }
        for ansr in &brch.ansrs {
            if let Err((_, _)) = check_free_var(self.prog, &ansr.val, &ansr.ty) {
                return false;
            }
        }
        true
    }

    fn solve_answer(&mut self, brch: &Branch) {
        let start = std::time::Instant::now();

        if let Some(map) = self.solver.check_sat(&brch.prims) {
            let duration = start.elapsed();
            writeln!(
                self.output.answer,
                "[ANSWER]: depth = {}, solving time = {:?}",
                brch.depth, duration
            )
            .unwrap();

            let map = map
                .into_iter()
                .map(|(var, lit)| (var, Term::Lit(lit)))
                .collect();

            for Answer { par, ty, val } in &brch.ansrs {
                writeln!(
                    self.output.answer,
                    "{}: {} = {}",
                    par,
                    reinterp_type(ty),
                    reinterp_term(&val.substitute(&map))
                )
                .unwrap();
            }
            self.ansr_cnt += 1;
        }
    }
}

fn branch_split(
    prog: &Program,
    brch: &Branch,
    heur: Heuristic,
) -> Vec<(Branch, Vec<(usize, usize)>)> {
    if brch.calls.is_empty() {
        // split by free variable
        if let Some(brchs) = split_free_var(prog, brch) {
            brchs
                .into_iter()
                .enumerate()
                .map(|(idx, brch)| (brch, vec![(0, idx)]))
                .collect()
        } else {
            panic!("this branch is already solved!!")
        }
    } else {
        // split by predicate calls
        let call_idx = match heur {
            args::Heuristic::LeftBiased => brch.left_biased_strategy(),
            args::Heuristic::Interleave => brch.interleave_strategy(),
            args::Heuristic::SmallFirst => brch.small_first_strategy(),
            args::Heuristic::Hybrid => brch.hybrid_strategy(),
            args::Heuristic::LookAhead => todo!(),
            args::Heuristic::Random => todo!(),
        };
        let mut res = Vec::new();
        for &rule_idx in brch.calls[call_idx].looks.iter() {
            if let Some((brch, path)) = apply_rule_with_reduction(prog, brch, call_idx, rule_idx) {
                res.push((brch, path));
            }
        }
        res
    }
}

fn get_branch_from_path(prog: &Program, pred: Ident, path: &[(usize, usize)]) -> Branch {
    let mut brch = branch_init(prog, pred);
    for &(call_idx, rule_idx) in path {
        brch = branch_step(prog, &brch, call_idx, rule_idx);
    }
    brch
}

fn branch_init(prog: &Program, pred: Ident) -> Branch {
    // predicate for query can not be polymorphic!
    assert!(prog.preds[&pred].polys.is_empty());

    let rules = &prog.preds[&pred].rules;
    let mut call = PredCall {
        pred,
        polys: Vec::new(),
        args: prog.preds[&pred]
            .pars
            .iter()
            .map(|(par, _ty)| Term::Var(par.tag_ctx(0)))
            .collect(),
        looks: (0..rules.len()).collect(),
        depth: 0,
    };
    call.lookahead_update(rules);

    let brch = Branch {
        depth: 0,
        ansrs: prog.preds[&pred]
            .pars
            .iter()
            .map(|(par, ty)| Answer {
                par: *par,
                ty: ty.clone(),
                val: Term::Var(par.tag_ctx(0)),
            })
            .collect(),
        prims: Vec::new(),
        calls: vec![call],
    };

    brch
}

fn branch_step(prog: &Program, brch: &Branch, call_idx: usize, rule_idx: usize) -> Branch {
    if brch.calls.is_empty() {
        // free variable split
        assert_eq!(call_idx, 0);
        if let Some(brchs) = split_free_var(prog, brch) {
            brchs.into_iter().nth(rule_idx).unwrap()
        } else {
            panic!("this branch is already solved!!")
        }
    } else {
        apply_rule(prog, brch, call_idx, rule_idx).unwrap()
    }
}

fn apply_rule_with_reduction(
    prog: &Program,
    brch: &Branch,
    call_idx: usize,
    rule_idx: usize,
) -> Option<(Branch, Vec<(usize, usize)>)> {
    const MAX_REDUCTION: usize = 10;
    let mut brch = apply_rule(prog, brch, call_idx, rule_idx)?;
    let mut path = vec![(call_idx, rule_idx)];
    for _ in 1..MAX_REDUCTION {
        if let Some(call_idx) = brch.check_reduction() {
            let looks = &brch.calls[call_idx].looks;
            assert!(looks.len() <= 1);
            if looks.is_empty() {
                return None;
            } else {
                let rule_idx = brch.calls[call_idx].looks[0];
                brch = apply_rule(prog, &brch, call_idx, rule_idx)?;
                path.push((call_idx, rule_idx));
            }
        } else {
            return Some((brch, path));
        }
    }
    Some((brch, path))
}

fn apply_rule(prog: &Program, brch: &Branch, call_idx: usize, rule_idx: usize) -> Option<Branch> {
    let rules = &prog.preds[&brch.calls[call_idx].pred].rules;
    let rule_ctx = rules[rule_idx].tag_ctx(brch.depth);

    let call = &brch.calls[call_idx];
    assert_eq!(rule_ctx.head.len(), call.args.len());

    let mut unifier: Unifier<IdentCtx, LitVal, OptCons<Ident>> = Unifier::new();
    for (par, arg) in rule_ctx.head.iter().zip(call.args.iter()) {
        if unifier.unify(par, arg).is_err() {
            return None;
        }
    }

    let mut new_brch = brch.clone();
    new_brch.depth += 1;
    new_brch.remove(call_idx);

    for (prim, args) in &rule_ctx.prims {
        new_brch.prims.push((*prim, args.clone()));
    }

    if !super::progagate::propagate_unify(&mut new_brch.prims, &mut unifier) {
        return None;
    }

    for (pred, polys, args) in rule_ctx.calls.iter().rev() {
        let mut new_call = PredCall {
            pred: *pred,
            polys: polys.clone(),
            args: args.clone(),
            looks: (0..prog.preds[pred].rules.len()).collect(),
            depth: call.depth + 1,
        };

        new_call.lookahead_update(&prog.preds[pred].rules);
        new_brch.insert(call_idx, new_call);
    }

    for call in &mut new_brch.calls {
        let mut dirty_flag = false;
        for arg in &mut call.args {
            if let Some(new_arg) = unifier.subst_opt(arg) {
                *arg = new_arg;
                dirty_flag = true;
            }
        }
        // update look-ahead information if any information is propagated
        if dirty_flag {
            call.lookahead_update(&prog.preds[&call.pred].rules);
        }
    }

    for ans in &mut new_brch.ansrs {
        ans.val = unifier.subst(&ans.val);
    }

    Some(new_brch)
}

fn check_free_var(
    prog: &Program,
    val: &TermVal<IdentCtx>,
    ty: &TermType,
) -> Result<(), (IdentCtx, TermType)> {
    match (val, ty) {
        (Term::Var(_), Term::Lit(_)) => {
            Ok(()) // ignore variables with literal type
        }
        (Term::Var(var), ty) => Err((*var, ty.clone())),
        (Term::Lit(_), _ty) => Ok(()),
        (
            Term::Cons(OptCons::Some(val_cons), val_args),
            Term::Cons(OptCons::Some(ty_cons), ty_args),
        ) => {
            let cons = &prog.conss[val_cons];
            assert_eq!(cons.data_cons, *ty_cons);
            let subst: HashMap<Ident, TermType> = cons
                .polys
                .iter()
                .zip(ty_args.iter())
                .map(|(poly, arg)| (*poly, arg.clone()))
                .collect();
            let ty_args: Vec<TermType> =
                cons.pars.iter().map(|par| par.substitute(&subst)).collect();
            for (val, ty) in val_args.iter().zip(ty_args.iter()) {
                check_free_var(prog, val, ty)?;
            }
            Ok(())
        }
        (Term::Cons(OptCons::None, val_args), Term::Cons(OptCons::None, ty_args)) => {
            for (val, ty) in val_args.iter().zip(ty_args.iter()) {
                check_free_var(prog, val, ty)?;
            }
            Ok(())
        }
        _ => unreachable!(),
    }
}

fn split_free_var(prog: &Program, brch: &Branch) -> Option<Vec<Branch>> {
    for ansr in &brch.ansrs {
        if let Err((var, ty)) = check_free_var(prog, &ansr.val, &ansr.ty) {
            match ty {
                Term::Lit(_) => unreachable!(),
                Term::Var(_) => {
                    panic!("type variable at runtime!")
                }
                Term::Cons(OptCons::Some(ty_cons), _args) => {
                    let mut brchs = Vec::new();
                    for cons in &prog.datas[&ty_cons].conss {
                        let cons_val: TermVal<IdentCtx> = Term::Cons(
                            OptCons::Some(*cons),
                            prog.conss[&cons]
                                .pars
                                .iter()
                                .map(|_| Term::Var(Ident::fresh(&"_").tag_ctx(brch.depth)))
                                .collect(),
                        );
                        let mut new_brch = brch.clone();
                        new_brch.depth += 1;
                        let mut unifier = Unifier::new();
                        unifier.unify(&Term::Var(var), &cons_val).unwrap();
                        new_brch.merge(unifier);
                        brchs.push(new_brch);
                    }
                    return Some(brchs);
                }
                Term::Cons(OptCons::None, args) => {
                    let args = args
                        .iter()
                        .map(|_| Term::Var(Ident::fresh(&"_").tag_ctx(brch.depth)))
                        .collect();
                    let mut new_brch = brch.clone();
                    new_brch.depth += 1;
                    let mut unifier: Unifier<IdentCtx, _, _> = Unifier::new();
                    unifier
                        .unify(&Term::Var(var), &Term::Cons(OptCons::None, args))
                        .unwrap();
                    new_brch.merge(unifier);
                    return Some(vec![new_brch]);
                }
            }
        }
    }

    None
}
