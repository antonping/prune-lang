use super::common::*;
use super::*;

use easy_smt::{Context, ContextBuilder, SExpr};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SolverBackend {
    Z3,
    CVC5,
}

pub struct SmtLibSolver {
    ctx: Context,
}

impl SmtLibSolver {
    pub fn new(backend: SolverBackend) -> Self {
        let mut ctx_bld = ContextBuilder::new();
        match backend {
            SolverBackend::Z3 => {
                ctx_bld.solver("z3").solver_args(["-smt2", "-in", "-v:0"]);
            }
            SolverBackend::CVC5 => {
                ctx_bld
                    .solver("cvc5")
                    .solver_args(["--quiet", "--lang=smt2", "--incremental"]);
            }
        }

        // ctx_bld.replay_file(Some(std::fs::File::create("replay.smt2").unwrap()));
        let mut ctx = ctx_bld.build().unwrap();
        ctx.set_logic("QF_NIA").unwrap();
        match backend {
            SolverBackend::Z3 => {
                ctx.set_option(":timeout", ctx.numeral(1000)).unwrap();
            }
            SolverBackend::CVC5 => {
                ctx.set_option(":tlimit-per", ctx.numeral(1000)).unwrap();
            }
        }

        // push an empty context for reset
        ctx.push().unwrap();

        SmtLibSolver { ctx }
    }

    fn declare_vars(&mut self, ty_map: &HashMap<IdentCtx, LitType>) -> HashMap<IdentCtx, SExpr> {
        let sexp_map: HashMap<IdentCtx, SExpr> = ty_map
            .iter()
            .map(|(var, typ)| {
                let sort = match typ {
                    LitType::TyInt => self.ctx.int_sort(),
                    LitType::TyFloat => self.ctx.real_sort(),
                    LitType::TyBool => self.ctx.bool_sort(),
                    LitType::TyChar => todo!(),
                };
                let sexp = self.ctx.declare_const(format!("{var:?}"), sort).unwrap();
                (*var, sexp)
            })
            .collect();

        sexp_map
    }

    fn add_constraints(
        &mut self,
        sexp_map: &HashMap<IdentCtx, SExpr>,
        prims: &[(Prim, Vec<AtomVal<IdentCtx>>)],
    ) {
        for (prim, args) in prims {
            let args: Vec<SExpr> = args
                .iter()
                .map(|arg| self.atom_to_sexp(arg, sexp_map))
                .collect();

            match (prim, &args[..]) {
                (
                    Prim::IAdd | Prim::ISub | Prim::IMul | Prim::IDiv | Prim::IRem,
                    &[arg1, arg2, arg3],
                ) => {
                    let res = match prim {
                        Prim::IAdd => self.ctx.plus(arg1, arg2),
                        Prim::ISub => self.ctx.sub(arg1, arg2),
                        Prim::IMul => self.ctx.times(arg1, arg2),
                        Prim::IDiv => self.ctx.div(arg1, arg2),
                        Prim::IRem => self.ctx.rem(arg1, arg2),
                        _ => unreachable!(),
                    };
                    self.ctx.assert(self.ctx.eq(res, arg3)).unwrap();
                }
                (Prim::INeg, &[arg1, arg2]) => {
                    let res = self.ctx.negate(arg1);
                    self.ctx.assert(self.ctx.eq(res, arg2)).unwrap();
                }
                (Prim::ICmp(cmp), &[arg1, arg2, arg3]) => {
                    let res = match cmp {
                        Compare::Lt => self.ctx.lt(arg1, arg2),
                        Compare::Le => self.ctx.lte(arg1, arg2),
                        Compare::Eq => self.ctx.eq(arg1, arg2),
                        Compare::Ge => self.ctx.gte(arg1, arg2),
                        Compare::Gt => self.ctx.gt(arg1, arg2),
                        Compare::Ne => self.ctx.not(self.ctx.eq(arg1, arg2)),
                    };
                    self.ctx.assert(self.ctx.eq(res, arg3)).unwrap();
                }
                (Prim::BAnd | Prim::BOr, &[arg1, arg2, arg3]) => {
                    let res = match prim {
                        Prim::BAnd => self.ctx.and(arg1, arg2),
                        Prim::BOr => self.ctx.or(arg1, arg2),
                        _ => unreachable!(),
                    };
                    self.ctx.assert(self.ctx.eq(res, arg3)).unwrap();
                }
                (Prim::BNot, &[arg1, arg2]) => {
                    let res = self.ctx.not(arg1);
                    self.ctx.assert(self.ctx.eq(res, arg2)).unwrap();
                }
                _ => {
                    panic!("wrong arity of primitives!");
                }
            }
        }
    }

    fn atom_to_sexp(&self, atom: &AtomVal<IdentCtx>, map: &HashMap<IdentCtx, SExpr>) -> SExpr {
        match atom {
            Term::Var(var) => map[var],
            Term::Lit(LitVal::Int(x)) => self.ctx.numeral(*x),
            Term::Lit(LitVal::Float(x)) => self.ctx.decimal(*x),
            Term::Lit(LitVal::Bool(x)) => {
                if *x {
                    self.ctx.true_()
                } else {
                    self.ctx.false_()
                }
            }
            Term::Lit(LitVal::Char(_x)) => todo!(),
            Term::Cons(_cons, _flds) => unreachable!(),
        }
    }

    fn sexp_to_lit_val(&self, sexpr: SExpr) -> Option<LitVal> {
        if let Some(res) = self.ctx.get_i64(sexpr) {
            return Some(LitVal::Int(res));
        }
        if let Some(res) = self.ctx.get_f64(sexpr) {
            return Some(LitVal::Float(res));
        }
        if let Some(res) = self.ctx.get_atom(sexpr) {
            match res {
                "true" => {
                    return Some(LitVal::Bool(true));
                }
                "false" => {
                    return Some(LitVal::Bool(false));
                }
                _ => {
                    return None;
                }
            }
        }

        // todo: basic type `Char``

        None
    }
}

impl common::PrimSolver for SmtLibSolver {
    fn check_sat(
        &mut self,
        prims: &[(Prim, Vec<AtomVal<IdentCtx>>)],
    ) -> Option<HashMap<IdentCtx, LitVal>> {
        // fast path for empty solver query
        if prims.is_empty() {
            return Some(HashMap::new());
        }

        // reset solver state
        self.ctx.pop().unwrap();
        self.ctx.push().unwrap();

        let ty_map: HashMap<IdentCtx, LitType> = infer_type(prims);
        let sexp_map = self.declare_vars(&ty_map);
        self.add_constraints(&sexp_map, prims);

        let check_res = self.ctx.check().unwrap();
        if check_res == easy_smt::Response::Sat {
            let vars: Vec<IdentCtx> = ty_map.keys().copied().collect();
            let res = vars
                .iter()
                .cloned()
                .zip(
                    self.ctx
                        .get_value(vars.iter().map(|var| sexp_map[var]).collect())
                        .unwrap()
                        .iter()
                        .map(|(_var, val)| self.sexp_to_lit_val(*val).unwrap()),
                )
                .collect();

            Some(res)
        } else {
            None
        }
    }

    fn generate_sat(
        &mut self,
        rng: &mut rngs::ThreadRng,
        prims: &[(Prim, Vec<AtomVal<IdentCtx>>)],
    ) -> Option<HashMap<IdentCtx, LitVal>> {
        // fast path for empty solver query
        if prims.is_empty() {
            return Some(HashMap::new());
        }

        // reset solver state
        self.ctx.pop().unwrap();
        self.ctx.push().unwrap();

        let ty_map: HashMap<IdentCtx, LitType> = infer_type(prims);
        let sexp_map = self.declare_vars(&ty_map);
        self.add_constraints(&sexp_map, prims);

        if self.ctx.check().unwrap() != easy_smt::Response::Sat {
            return None;
        }

        let mut vars: Vec<IdentCtx> = ty_map.keys().copied().collect();
        let mut count = 0;

        while !vars.is_empty() && count < 10 {
            let var = vars.remove(rng.random_range(0..vars.len()));
            let value = match ty_map[&var] {
                LitType::TyInt => self.ctx.numeral(rng.random::<i32>() as i64),
                LitType::TyFloat => todo!(),
                LitType::TyBool => {
                    if rng.random_bool(0.5) {
                        self.ctx.true_()
                    } else {
                        self.ctx.false_()
                    }
                }
                LitType::TyChar => todo!(),
            };
            let res = self
                .ctx
                .check_assuming(vec![self.ctx.eq(sexp_map[&var], value)])
                .unwrap();
            if res == easy_smt::Response::Sat {
                count = 0;
                self.ctx.assert(self.ctx.eq(sexp_map[&var], value)).unwrap();
                let check_res = self.ctx.check().unwrap();
                if check_res != easy_smt::Response::Sat {
                    return None;
                }
            } else {
                count += 1;
                vars.push(var);
            }
        }

        let check_res = self.ctx.check().unwrap();
        if check_res != easy_smt::Response::Sat {
            return None;
        }
        let vars: Vec<IdentCtx> = ty_map.keys().copied().collect();
        let res = vars
            .iter()
            .cloned()
            .zip(
                self.ctx
                    .get_value(vars.iter().map(|var| sexp_map[var]).collect())
                    .unwrap()
                    .iter()
                    .map(|(_var, val)| self.sexp_to_lit_val(*val).unwrap()),
            )
            .collect();

        Some(res)
    }
}
