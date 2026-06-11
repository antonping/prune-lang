use super::*;
use itertools::Itertools;
use std::fmt;

#[derive(Clone, Debug)]
pub struct Branch {
    pub depth: usize,
    pub answers: Vec<(Ident, TermVal<IdentCtx>)>,
    pub prims: Vec<(Prim, Vec<AtomVal<IdentCtx>>)>,
    pub calls: Vec<PredCall>,
}

impl fmt::Display for Branch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "##### depth: = {} #####", self.depth)?;

        for (par, val) in &self.answers {
            writeln!(f, "{par} = {val}")?;
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
            answers: pars
                .iter()
                .map(|par| (*par, Term::Var(par.tag_ctx(0))))
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

        for (_par, val) in &mut self.answers {
            *val = unifier.subst(val);
        }
    }

    pub fn insert(&mut self, call_idx: usize, call: PredCall) {
        self.calls.insert(call_idx, call);
    }

    pub fn remove(&mut self, call_idx: usize) -> PredCall {
        self.calls.remove(call_idx)
    }

    pub fn random_strategy(&mut self, rng: &mut rand::rngs::ThreadRng) -> usize {
        assert!(!self.calls.is_empty());
        rng.random_range(0..self.calls.len())
    }

    pub fn left_biased_strategy(&mut self) -> usize {
        assert!(!self.calls.is_empty());
        0
    }

    pub fn interleave_strategy(&mut self) -> usize {
        (0..self.calls.len())
            .min_by_key(|idx| self.calls[*idx].depth)
            .unwrap()
    }

    pub fn small_first_strategy(&mut self) -> usize {
        (0..self.calls.len())
            .min_by_key(|idx| {
                let call = &self.calls[*idx];
                call.looks.len() * 1000 + call.depth
            })
            .unwrap()
    }

    pub fn check_reduction(&self) -> Option<usize> {
        (0..self.calls.len()).find(|idx| self.calls[*idx].looks.len() <= 1)
    }
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct PredCall {
    pub pred: Ident,
    pub polys: Vec<TermType>,
    pub args: Vec<TermVal<IdentCtx>>,
    pub looks: Vec<usize>,
    pub depth: usize,
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

// input: a vector of positive integer [a, b, ..., z]
// output: solution of equation x^-a + x^-b + ... + x^-z = 1
pub fn tau_function(vec: &Vec<usize>) -> f32 {
    if vec.len() <= 1 {
        return 1.0;
    }

    // Newton's method for finding numerical solutions
    let mut x: f32 = 1.0;
    for _ in 0..100 {
        // g(x) = sum(x^(-w_i)) - 1
        let mut g: f32 = -1.0;
        for &w in vec {
            g += x.powi(-(w as i32));
        }

        // g'(x) = sum(-w_i * x^(-w_i - 1))
        let mut gp: f32 = 0.0;
        for &w in vec {
            gp -= (w as f32) * x.powi(-(w as i32) - 1);
        }

        let step = g / gp;
        let nx = x - step;

        if step.abs() < 1e-4 {
            return nx;
        } else {
            x = nx;
        }
    }
    panic!("Newton's method fails to converge after 100 iterations!")
}

#[test]
fn test_tau_function() {
    assert_eq!(tau_function(&vec![]), 1.0);

    assert_eq!(tau_function(&vec![1]), 1.0);

    assert_eq!(tau_function(&vec![2]), 1.0);

    let x = tau_function(&vec![1, 1]);
    assert!((x - 2.0).abs() < 1e-4);

    let x = tau_function(&vec![1, 1, 1]);
    assert!((x - 3.0).abs() < 1e-4);

    let x = tau_function(&vec![2, 2]);
    assert!((x - 1.41421356).abs() < 1e-4);

    let x = tau_function(&vec![1, 2]);
    assert!((x - 1.61803399).abs() < 1e-4);

    let x = tau_function(&vec![1, 2, 3]);
    assert!((x - 1.83928676).abs() < 1e-4);

    let x = tau_function(&vec![5, 5]);
    assert!((x - 1.14869835).abs() < 1e-4);
}
