use crate::cli::args;

use super::*;

pub trait PrimSolver {
    fn check_sat(&mut self, prims: &[(Prim, Vec<AtomVal<IdentCtx>>)]) -> bool;

    fn generate_model(
        &mut self,
        rng: &mut rngs::ThreadRng,
        prims: &[(Prim, Vec<AtomVal<IdentCtx>>)],
    ) -> HashMap<IdentCtx, LitVal>;
}

pub fn infer_type(prims: &[(Prim, Vec<AtomVal<IdentCtx>>)]) -> HashMap<IdentCtx, LitType> {
    let mut map = HashMap::new();

    for (prim, args) in prims {
        for (arg, typ) in args.iter().zip(prim.get_typ().iter()) {
            match arg {
                Term::Var(var) => {
                    if let Some(res) = map.get(var) {
                        assert_eq!(*res, *typ);
                    } else {
                        map.insert(*var, *typ);
                    }
                }
                Term::Lit(lit) => {
                    assert_eq!(lit.get_typ(), *typ);
                }
                Term::Cons(_, _) => unreachable!(),
            }
        }
    }

    map
}

pub fn new_solver<'args>(args: &'args args::CliArgs) -> Box<dyn PrimSolver + 'args> {
    let solver: Box<dyn PrimSolver> = match args.solver {
        args::Solver::Z3 | args::Solver::CVC5 => {
            Box::new(super::solver::smtlib::SmtLibSolver::new(args))
        }
        args::Solver::NoSmt => Box::new(super::solver::no_smt::NoSmtSolver::new()),
    };
    solver
}
