use super::config::{RunnerConfig, RunnerStats};
use super::solver;
use super::strategy::*;
use super::*;
use crate::cli::args::{self, CliArgs};
use crate::cli::pipeline::OutputWriter;

pub struct RunnerState<'prog, 'io> {
    prog: &'prog Program,
    output: &'io mut OutputWriter,
    config: RunnerConfig,
    stats: RunnerStats,
    ctx_cnt: usize,
    ansr_cnt: usize,
    rng: rngs::ThreadRng,
    stack: Vec<Branch>,
    solver: Box<dyn solver::common::PrimSolver>,
}

impl<'prog, 'io> RunnerState<'prog, 'io> {
    pub fn new(
        prog: &'prog Program,
        output: &'io mut OutputWriter,
        args: &CliArgs,
    ) -> RunnerState<'prog, 'io> {
        let solver_obj: Box<dyn solver::common::PrimSolver> = match args.solver {
            args::Solver::Z3 => Box::new(super::solver::smtlib::SmtLibSolver::new(
                super::solver::smtlib::SolverBackend::Z3,
            )),
            args::Solver::CVC5 => Box::new(super::solver::smtlib::SmtLibSolver::new(
                super::solver::smtlib::SolverBackend::CVC5,
            )),
            args::Solver::Encode => Box::new(super::solver::no_smt::NoSmtSolver::new()),
        };

        let rng = rand::rng();

        RunnerState {
            prog,
            output,
            config: RunnerConfig::new(args),
            stats: RunnerStats::new(),
            ctx_cnt: 0,
            ansr_cnt: 0,
            rng,
            stack: Vec::new(),
            solver: solver_obj,
        }
    }

    pub fn config_set_param(&mut self, param: &QueryParam) {
        self.config.set_param(param);
    }

    fn reset(&mut self) {
        self.stats.reset();
        assert!(self.stack.is_empty());
        self.ctx_cnt = 0;
    }

    fn init_stack(&mut self, pred: Ident) {
        // predicate for query can not be polymorphic!
        assert!(self.prog.preds[&pred].polys.is_empty());

        self.ctx_cnt = 0;
        let rules = &self.prog.preds[&pred].rules;
        let mut call = PredCall {
            pred,
            polys: Vec::new(),
            args: self.prog.preds[&pred]
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
            ansrs: self.prog.preds[&pred]
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

        self.stack.push(brch);
    }

    fn run_dfs_with_depth(&mut self, depth_start: usize, depth_end: usize) {
        while let Some(mut brch) = self.stack.pop() {
            if self.config.debug_mode {
                println!("{brch}");

                // pause to wait for any input
                let mut s = String::new();
                std::io::stdin().read_line(&mut s).unwrap();
            }

            if self.ansr_cnt >= self.config.answer_limit {
                break;
            }
            if brch.depth + brch.calls.len() > depth_end {
                continue;
            }

            if brch.calls.is_empty() {
                if let Some(brchs) = self.split_free_var(&brch) {
                    for brch in brchs {
                        self.stack.push(brch);
                    }
                } else {
                    if brch.depth >= depth_start && brch.depth <= depth_end {
                        self.solve_answer(&brch);
                    }
                }
            } else {
                self.run_branch_step(&mut brch);
            }
        }
    }

    fn split_free_var(&mut self, brch: &Branch) -> Option<Vec<Branch>> {
        let mut free_vars: Vec<(IdentCtx, TermType)> = Vec::new();
        for ansr in &brch.ansrs {
            self.collect_free_vars(&ansr.val, &ansr.ty, &mut free_vars);
        }
        free_vars.sort_by_key(|(id, _)| *id);
        free_vars.dedup_by_key(|(id, _)| *id);
        free_vars.retain(|(_, ty)| !matches!(ty, Term::Lit(_)));
        if free_vars.is_empty() {
            return None;
        }

        let (var, ty) = &free_vars[self.rng.random_range(0..free_vars.len())];
        match ty {
            Term::Lit(_) => unreachable!(),
            Term::Var(_) => {
                panic!("type variable at runtime!")
            }
            Term::Cons(OptCons::Some(ty_cons), _args) => {
                let mut brchs = Vec::new();
                let data = self.prog.datas.values().find(|d| d.name == *ty_cons)?;
                for cons in &data.cons {
                    self.ctx_cnt += 1;
                    let cons_val: TermVal<IdentCtx> = Term::Cons(
                        OptCons::Some(cons.name),
                        cons.flds
                            .iter()
                            .map(|_| Term::Var(Ident::fresh(&"_").tag_ctx(self.ctx_cnt)))
                            .collect(),
                    );
                    let mut new_brch = brch.clone();
                    new_brch.depth += 5;
                    let mut unifier = Unifier::new();
                    unifier.unify(&Term::Var(*var), &cons_val).unwrap();
                    new_brch.merge(unifier);
                    brchs.push(new_brch);
                }
                Some(brchs)
            }
            Term::Cons(OptCons::None, args) => {
                self.ctx_cnt += 1;
                let mut new_brch = brch.clone();
                new_brch.depth += 5;
                let args = args
                    .iter()
                    .map(|_| Term::Var(Ident::fresh(&"_").tag_ctx(self.ctx_cnt)))
                    .collect();
                let mut unifier: Unifier<IdentCtx, _, _> = Unifier::new();
                unifier
                    .unify(&Term::Var(*var), &Term::Cons(OptCons::None, args))
                    .unwrap();
                new_brch.merge(unifier);
                Some(vec![new_brch])
            }
        }
    }

    fn collect_free_vars(
        &self,
        val: &TermVal<IdentCtx>,
        ty: &TermType,
        out: &mut Vec<(IdentCtx, TermType)>,
    ) {
        match (val, ty) {
            (Term::Var(var), ty) => {
                out.push((*var, ty.clone()));
            }
            (Term::Lit(_), _ty) => {
                // do nothing
            }
            (
                Term::Cons(OptCons::Some(val_cons), val_args),
                Term::Cons(OptCons::Some(ty_cons), ty_args),
            ) => {
                let data = &self.prog.datas[ty_cons];
                let cons = data.cons.iter().find(|con| con.name == *val_cons).unwrap();
                let subst: HashMap<Ident, TermType> = data
                    .polys
                    .iter()
                    .zip(ty_args.iter())
                    .map(|(poly, arg)| (*poly, arg.clone()))
                    .collect();
                let ty_args: Vec<TermType> =
                    cons.flds.iter().map(|fld| fld.substitute(&subst)).collect();
                for (val, ty) in val_args.iter().zip(ty_args.iter()) {
                    self.collect_free_vars(val, ty, out);
                }
            }
            (Term::Cons(OptCons::None, val_args), Term::Cons(OptCons::None, ty_args)) => {
                for (val, ty) in val_args.iter().zip(ty_args.iter()) {
                    self.collect_free_vars(val, ty, out);
                }
            }
            _ => unreachable!(),
        }
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

    fn run_branch_step(&mut self, brch: &mut Branch) {
        let call_idx = match self.config.heuristic {
            args::Heuristic::LeftBiased => brch.left_biased_strategy(),
            args::Heuristic::Interleave => brch.interleave_strategy(),
            args::Heuristic::SmallFirst => brch.small_first_strategy(),
            args::Heuristic::Hybrid => brch.hybrid_strategy(),
            args::Heuristic::LookAhead => {
                // lookahead heuristic can't work without reductions!
                assert!(self.config.reduction);
                self.lookahead_choose(brch)
            }
            args::Heuristic::Random => brch.random_strategy(&mut self.rng),
        };

        use rand::seq::SliceRandom;
        let mut looks = brch.calls[call_idx].looks.clone();
        looks.shuffle(&mut self.rng);

        self.stats.step();

        if self.config.reduction {
            for &rule_idx in looks.iter().rev() {
                if let Some((brch, _steps)) =
                    self.apply_rule_with_reduction(brch, call_idx, rule_idx)
                {
                    self.stack.push(brch);
                }
            }
        } else {
            for &rule_idx in looks.iter().rev() {
                if let Some(brch) = self.apply_rule(brch, call_idx, rule_idx) {
                    self.stack.push(brch);
                }
            }
        }
    }

    fn lookahead_choose(&mut self, brch: &Branch) -> usize {
        assert!(!brch.calls.is_empty());
        let mut best_score: f32 = f32::MAX;
        let mut best_idx: usize = 0;

        let mut calls: Vec<usize> = (0..brch.calls.len()).collect();
        calls.sort_by_key(|call| brch.calls[*call].looks.len());

        for call_idx in calls.into_iter() {
            self.stats.step_la();

            let mut vec = Vec::new();
            for rule_idx in brch.calls[call_idx].looks.iter().rev() {
                if let Some((new_brch, steps)) =
                    self.apply_rule_with_reduction(brch, call_idx, *rule_idx)
                    && !new_brch.calls.is_empty()
                {
                    vec.push(steps);
                }
            }
            let tau = tau_function(&vec);
            if tau < 1.2 {
                return call_idx;
            }
            let score = tau + (brch.calls[call_idx].depth as f32) * (0.001_f32);
            if score < best_score {
                best_score = score;
                best_idx = call_idx;
            }
        }
        // println!("best_score = {}, best_idx = {}", best_score, best_idx);
        best_idx
    }

    fn apply_rule_with_reduction(
        &mut self,
        brch: &Branch,
        call_idx: usize,
        rule_idx: usize,
    ) -> Option<(Branch, usize)> {
        const MAX_REDUCTION: usize = 10;
        let mut brch = self.apply_rule(brch, call_idx, rule_idx)?;
        for steps in 1..MAX_REDUCTION {
            if let Some(call_idx) = brch.check_reduction() {
                let looks = &brch.calls[call_idx].looks;
                assert!(looks.len() <= 1);
                if looks.is_empty() {
                    return Some((brch, steps));
                } else {
                    brch = self.apply_rule(&brch, call_idx, brch.calls[call_idx].looks[0])?;
                }
            } else {
                return Some((brch, steps));
            }
        }
        Some((brch, MAX_REDUCTION))
    }

    fn apply_rule(&mut self, brch: &Branch, call_idx: usize, rule_idx: usize) -> Option<Branch> {
        let rules = &self.prog.preds[&brch.calls[call_idx].pred].rules;
        self.ctx_cnt += 1;
        let rule_ctx = rules[rule_idx].tag_ctx(self.ctx_cnt);

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
                looks: (0..self.prog.preds[pred].rules.len()).collect(),
                depth: call.depth + 1,
            };

            new_call.lookahead_update(&self.prog.preds[pred].rules);
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
                call.lookahead_update(&self.prog.preds[&call.pred].rules);
            }
        }

        for ans in &mut new_brch.ansrs {
            ans.val = unifier.subst(&ans.val);
        }

        Some(new_brch)
    }

    pub fn run_iddfs_loop(&mut self, entry: Ident) -> usize {
        for depth_limit in
            (self.config.depth_step..=self.config.depth_limit).step_by(self.config.depth_step)
        {
            writeln!(
                self.output.stat,
                "[RUN]: try depth = {}... (found answer: {})",
                depth_limit, self.ansr_cnt
            )
            .unwrap();

            self.reset();
            self.init_stack(entry);
            self.run_dfs_with_depth(depth_limit - self.config.depth_step + 1, depth_limit);

            let stat_res = self.stats.print_stat();
            writeln!(self.output.stat, "{stat_res}").unwrap();

            if self.ansr_cnt >= self.config.answer_limit {
                return self.ansr_cnt;
            }
        }
        self.ansr_cnt
    }
}
