use super::*;

pub struct NoSmtSolver;

impl NoSmtSolver {
    pub fn new() -> Self {
        NoSmtSolver
    }
}

impl Default for NoSmtSolver {
    fn default() -> Self {
        Self::new()
    }
}

impl common::PrimSolver for NoSmtSolver {
    fn check_sat(&mut self, prims: &[(Prim, Vec<AtomVal<IdentCtx>>)]) -> bool {
        if prims.is_empty() {
            true
        } else {
            panic!("no solver for unsolved primitives!")
        }
    }

    fn generate_model(
        &mut self,
        _rng: &mut rngs::ThreadRng,
        prims: &[(Prim, Vec<AtomVal<IdentCtx>>)],
    ) -> HashMap<IdentCtx, LitVal> {
        if prims.is_empty() {
            HashMap::new()
        } else {
            panic!("no solver for unsolved primitives!")
        }
    }
}
