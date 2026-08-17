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

pub fn new_solver(args: &args::CliArgs) -> Box<dyn PrimSolver> {
    let solver = match args.solver {
        args::Solver::Z3 => Some(super::solver::smtlib::Solver::Z3),
        args::Solver::CVC5 => Some(super::solver::smtlib::Solver::CVC5),
        args::Solver::NoSmt => None,
    };

    let int_rep = match args.int_rep {
        args::IntRep::Num => super::solver::smtlib::IntRep::Num,
        args::IntRep::BV8 => super::solver::smtlib::IntRep::BV8,
        args::IntRep::BV16 => super::solver::smtlib::IntRep::BV16,
        args::IntRep::BV32 => super::solver::smtlib::IntRep::BV32,
    };

    let solver: Box<dyn solver::common::PrimSolver> = match solver {
        Some(solver) => Box::new(super::solver::smtlib::SmtLibSolver::new(solver, int_rep)),
        None => Box::new(super::solver::no_smt::NoSmtSolver::new()),
    };

    solver
}
