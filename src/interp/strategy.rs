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
            history: History::new(),
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

    pub fn clear_history(&mut self) {
        for call in &mut self.calls {
            call.history.clear();
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

    pub fn naive_strategy(&mut self, n: usize) -> usize {
        assert!(!self.calls.is_empty());

        let idx = self
            .calls
            .iter()
            .position(|call| call.history.naive_strategy_pred(n));

        if let Some(idx) = idx {
            idx
        } else {
            self.clear_history();
            self.naive_strategy(n)
        }
    }

    pub fn struct_recur_strategy(&mut self) -> usize {
        assert!(!self.calls.is_empty());

        let idx = self.calls.iter().position(|call| {
            call.history
                .struct_recur_strategy_pred(call.pred, &call.args)
        });

        if let Some(idx) = idx {
            idx
        } else {
            self.clear_history();
            self.struct_recur_strategy()
        }
    }

    pub fn small_first_strategy(&mut self) -> usize {
        let mut vec = Vec::new();

        for call_idx in 0..self.calls.len() {
            let call = &self.calls[call_idx];
            vec.push(1000 * call.looks.len() + call.history.len());
        }

        let (idx, _) = vec.iter().enumerate().min_by_key(|(_idx, br)| *br).unwrap();
        idx
    }

    pub fn check_reduction(&self) -> Option<usize> {
        for call_idx in 0..self.calls.len() {
            if self.calls[call_idx].looks.len() <= 1 {
                return Some(call_idx);
            }
        }
        None
    }
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct PredCall {
    pub pred: Ident,
    pub polys: Vec<TermType>,
    pub args: Vec<TermVal<IdentCtx>>,
    pub looks: Vec<usize>,
    pub history: History,
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

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct HistoryNode {
    pred: Ident,
    args_size: Vec<usize>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct History(Vec<HistoryNode>);

impl History {
    pub fn new() -> History {
        History(Vec::new())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn push(&mut self, pred: Ident, args_size: Vec<usize>) {
        self.0.push(HistoryNode { pred, args_size });
    }

    pub fn left_biased_strategy_pred(&self) -> bool {
        true
    }

    pub fn naive_strategy_pred(&self, n: usize) -> bool {
        self.0.len() < n
    }

    pub fn struct_recur_strategy_pred(&self, pred: Ident, args: &[TermVal<IdentCtx>]) -> bool {
        let args_size: Vec<usize> = args.iter().map(|arg| arg.height()).collect();

        for node in &self.0 {
            if node.pred == pred
                && node
                    .args_size
                    .iter()
                    .zip(args_size.iter())
                    .all(|(arg0, arg)| arg0 <= arg)
            {
                return false;
            }
        }

        true
    }
}

impl Default for History {
    fn default() -> Self {
        Self::new()
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
