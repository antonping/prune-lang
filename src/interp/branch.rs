use super::*;
use itertools::Itertools;
use std::fmt;

#[derive(Clone, Debug)]
pub struct Branch {
    pub depth: usize,
    pub ansrs: Vec<Answer>,
    pub prims: Vec<(Prim, Vec<AtomVal<IdentCtx>>)>,
    pub calls: Vec<PredCall>,
}

#[derive(Clone, Debug)]
pub struct Answer {
    pub par: Ident,
    pub ty: TermType,
    pub val: TermVal<IdentCtx>,
}

#[derive(Clone, Debug)]
pub struct PredCall {
    pub pred: Ident,
    pub polys: Vec<TermType>,
    pub args: Vec<TermVal<IdentCtx>>,
    pub looks: Vec<usize>,
    pub depth: usize,
}

impl fmt::Display for Branch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "##### depth: = {} #####", self.depth)?;

        for ansr in &self.ansrs {
            writeln!(f, "{ansr}")?;
        }

        for (prim, args) in &self.prims {
            let args = args.iter().format(", ");
            writeln!(f, "{prim:?}({args})")?;
        }

        for call in &self.calls {
            writeln!(f, "{call}")?;
        }

        Ok(())
    }
}

impl fmt::Display for Answer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {} = {}", self.par, self.ty, self.val)
    }
}

impl fmt::Display for PredCall {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let args = self.args.iter().format(", ");
        if self.polys.is_empty() {
            write!(f, "{}({})", self.pred, args)
        } else {
            let polys = self.polys.iter().format(", ");
            write!(f, "{}[{}]({})", self.pred, polys, args)
        }
    }
}

impl Branch {
    pub fn new(pred: Ident, pars: Vec<Ident>, rule_cnt: usize) -> Branch {
        let call = PredCall {
            pred,
            polys: Vec::new(),
            args: pars.iter().map(|par| Term::Var(par.tag_ctx(0))).collect(),
            looks: (0..rule_cnt).collect(),
            depth: 0,
        };

        Branch {
            depth: 0,
            ansrs: pars
                .iter()
                .map(|par| Answer {
                    par: *par,
                    ty: Term::Lit(LitType::TyBool),
                    val: Term::Var(par.tag_ctx(0)),
                })
                .collect(),
            prims: Vec::new(),
            calls: vec![call],
        }
    }

    pub fn merge(&mut self, unifier: Unifier<IdentCtx, LitVal, OptCons<Ident>>) {
        for call in &mut self.calls {
            for arg in &mut call.args {
                *arg = unifier.subst(arg);
            }
        }

        for ans in &mut self.ansrs {
            ans.val = unifier.subst(&ans.val);
        }
    }

    pub fn insert(&mut self, call_idx: usize, call: PredCall) {
        self.calls.insert(call_idx, call);
    }

    pub fn remove(&mut self, call_idx: usize) -> PredCall {
        self.calls.remove(call_idx)
    }

    pub fn random_strategy(&self, rng: &mut rand::rngs::ThreadRng) -> usize {
        assert!(!self.calls.is_empty());
        rng.random_range(0..self.calls.len())
    }

    pub fn left_biased_strategy(&self) -> usize {
        assert!(!self.calls.is_empty());
        0
    }

    pub fn interleave_strategy(&self) -> usize {
        (0..self.calls.len())
            .min_by_key(|idx| self.calls[*idx].depth)
            .unwrap()
    }

    pub fn small_first_strategy(&self) -> usize {
        (0..self.calls.len())
            .min_by_key(|idx| {
                let call = &self.calls[*idx];
                call.looks.len() * 1000 + call.depth
            })
            .unwrap()
    }

    pub fn hybrid_strategy(&self) -> usize {
        (0..self.calls.len())
            .min_by_key(|idx| {
                let call = &self.calls[*idx];
                call.looks.len() * 2 + call.depth
            })
            .unwrap()
    }

    pub fn check_reduction(&self) -> Option<usize> {
        (0..self.calls.len()).find(|idx| self.calls[*idx].looks.len() <= 1)
    }
}

impl PredCall {
    fn try_unify_rule_head(&self, head: &[TermVal]) -> Result<(), ()> {
        assert_eq!(head.len(), self.args.len());

        let mut unifier: Unifier<IdentCtx, LitVal, OptCons<Ident>> = Unifier::new();
        for (par, arg) in head.iter().zip(self.args.iter()) {
            if unifier.unify(&par.tag_ctx(0), arg).is_err() {
                return Err(());
            }
        }

        Ok(())
    }

    pub fn lookahead_update(&mut self, rules: &[Rule]) {
        let mut new_looks = self.looks.clone();
        new_looks.retain(|look| self.try_unify_rule_head(&rules[*look].head).is_ok());
        self.looks = new_looks
    }
}

pub fn reinterp_type(typ: &TermType) -> TermType {
    match typ {
        Term::Cons(cons, args) => match cons {
            OptCons::Some(c) => match c.name.as_str() {
                "%Int" => Term::Lit(LitType::TyInt),
                "%Bool" => Term::Lit(LitType::TyBool),
                _ => Term::Cons(*cons, args.iter().map(reinterp_type).collect()),
            },
            OptCons::None => Term::Cons(*cons, args.iter().map(reinterp_type).collect()),
        },
        other => other.clone(),
    }
}

pub fn reinterp_term(term: &TermVal<IdentCtx>) -> TermVal<IdentCtx> {
    match term {
        Term::Cons(cons, args) => match cons {
            OptCons::Some(c) => match c.name.as_str() {
                "%Pos" => Term::Lit(LitVal::Int(bit_list_to_uint(&args[0]) as i32)),
                "%Zero" => Term::Lit(LitVal::Int(0)),
                "%Neg" => Term::Lit(LitVal::Int(-(bit_list_to_uint(&args[0]) as i32))),
                "%F" => Term::Lit(LitVal::Bool(false)),
                "%T" => Term::Lit(LitVal::Bool(true)),
                _ => Term::Cons(*cons, args.iter().map(reinterp_term).collect()),
            },
            OptCons::None => Term::Cons(*cons, args.iter().map(reinterp_term).collect()),
        },
        other => other.clone(),
    }
}

pub fn bit_list_to_uint(term: &TermVal<IdentCtx>) -> u64 {
    let Term::Cons(OptCons::Some(cons), args) = term else {
        panic!("invalid bit list!");
    };
    match cons.name.as_str() {
        "%Nil" => {
            assert!(args.is_empty());
            1
        }
        "%Cons" => {
            let [head, tail] = args.as_slice() else {
                panic!("not a bit list!");
            };
            let Term::Cons(OptCons::Some(bit), args) = head else {
                panic!("not a bit!");
            };
            assert!(args.is_empty());
            let bit_val = match bit.name.as_str() {
                "%O" => 0,
                "%I" => 1,
                _ => panic!("not a bit!"),
            };
            bit_list_to_uint(tail) * 2 + bit_val
        }
        _ => panic!("invalid bit list!"),
    }
}

pub fn apply_rule_with_reduction(
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

pub fn apply_rule(
    prog: &Program,
    brch: &Branch,
    call_idx: usize,
    rule_idx: usize,
) -> Option<Branch> {
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

pub fn walk_free_var(
    prog: &Program,
    val: &TermVal<IdentCtx>,
    ty: &TermType,
    map: &mut HashMap<IdentCtx, TermType>,
) {
    match (val, ty) {
        (Term::Var(var), ty) => {
            map.insert(*var, ty.clone());
        }
        (Term::Lit(lit), Term::Lit(ty)) => {
            assert_eq!(lit.get_typ(), *ty);
        }
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
                walk_free_var(prog, val, ty, map);
            }
        }
        (Term::Cons(OptCons::None, val_args), Term::Cons(OptCons::None, ty_args)) => {
            for (val, ty) in val_args.iter().zip(ty_args.iter()) {
                walk_free_var(prog, val, ty, map);
            }
        }
        _ => unreachable!(),
    }
}

pub fn branch_init(prog: &Program, pred: Ident) -> Branch {
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
