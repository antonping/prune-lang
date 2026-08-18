use super::common::*;
use super::*;
use easy_smt::{Context, ContextBuilder, SExpr};
use rand::seq::SliceRandom;

pub struct SmtLibSolver<'args> {
    args: &'args CliArgs,
    ctx: Context,
}

impl<'args> SmtLibSolver<'args> {
    pub fn new(args: &'args CliArgs) -> Self {
        let mut ctx_bld = ContextBuilder::new();
        match args.solver {
            args::Solver::Z3 => {
                ctx_bld.solver("z3").solver_args(["-smt2", "-in", "-v:0"]);
            }
            args::Solver::CVC5 => {
                ctx_bld
                    .solver("cvc5")
                    .solver_args(["--quiet", "--lang=smt2", "--incremental"]);
            }
            args::Solver::NoSmt => unreachable!(),
        }

        // ctx_bld.replay_file(Some(std::fs::File::create("replay.smt2").unwrap()));
        let mut ctx = ctx_bld.build().unwrap();
        ctx.push().unwrap(); // push an empty context for reset
        SmtLibSolver { ctx, args }
    }

    fn declare_vars(&mut self, ty_map: &HashMap<IdentCtx, LitType>) -> HashMap<IdentCtx, SExpr> {
        let sexp_map: HashMap<IdentCtx, SExpr> = ty_map
            .iter()
            .map(|(var, typ)| {
                let sort = match typ {
                    LitType::TyInt => self
                        .ctx
                        .bit_vec_sort(self.ctx.numeral(self.args.int_rep.get_width())),
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
                        Prim::IAdd => self.ctx.bvadd(arg1, arg2),
                        Prim::ISub => self.ctx.bvsub(arg1, arg2),
                        Prim::IMul => self.ctx.bvmul(arg1, arg2),
                        Prim::IDiv => self.ctx.bvsdiv(arg1, arg2),
                        Prim::IRem => self.ctx.bvsrem(arg1, arg2),
                        _ => unreachable!(),
                    };
                    self.ctx.assert(self.ctx.eq(res, arg3)).unwrap();
                }
                (Prim::INeg, &[arg1, arg2]) => {
                    let res = self.ctx.bvneg(arg1);
                    self.ctx.assert(self.ctx.eq(res, arg2)).unwrap();
                }
                (Prim::ICmp(cmp), &[arg1, arg2, arg3]) => {
                    let res = match cmp {
                        Compare::Lt => self.ctx.bvslt(arg1, arg2),
                        Compare::Le => self.ctx.bvsle(arg1, arg2),
                        Compare::Eq => self.ctx.eq(arg1, arg2),
                        Compare::Ge => self.ctx.bvsge(arg1, arg2),
                        Compare::Gt => self.ctx.bvsgt(arg1, arg2),
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
            Term::Lit(LitVal::Int(x)) => match self.args.int_rep {
                args::IntRep::BV8 => self.ctx.binary(8, i8::try_from(*x).unwrap()),
                args::IntRep::BV16 => self.ctx.binary(16, i16::try_from(*x).unwrap()),
                args::IntRep::BV32 => self.ctx.binary(32, *x),
            },
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
        // println!("sexpr: {}", self.ctx.display(sexpr));

        match self.args.int_rep {
            args::IntRep::BV8 => {
                if let Some(res) = self.ctx.get_u8(sexpr) {
                    return Some(LitVal::Int(res.cast_signed() as i32));
                }
            }
            args::IntRep::BV16 => {
                if let Some(res) = self.ctx.get_u16(sexpr) {
                    return Some(LitVal::Int(res.cast_signed() as i32));
                }
            }
            args::IntRep::BV32 => {
                if let Some(res) = self.ctx.get_u32(sexpr) {
                    return Some(LitVal::Int(res.cast_signed() as i32));
                }
            }
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

        None
    }
}

impl<'args> common::PrimSolver for SmtLibSolver<'args> {
    fn check_sat(&mut self, prims: &[(Prim, Vec<AtomVal<IdentCtx>>)]) -> bool {
        // fast path for empty solver query
        if prims.is_empty() {
            return true;
        }

        // reset solver state
        self.ctx.pop().unwrap();
        self.ctx.push().unwrap();

        let ty_map: HashMap<IdentCtx, LitType> = infer_type(prims);
        let sexp_map = self.declare_vars(&ty_map);
        self.add_constraints(&sexp_map, prims);

        let res = self.ctx.check().unwrap();
        match res {
            easy_smt::Response::Sat => true,
            easy_smt::Response::Unsat => false,
            easy_smt::Response::Unknown => panic!("SMT solver returns `Unknown`!"),
        }
    }

    fn generate_model(
        &mut self,
        rng: &mut rngs::ThreadRng,
        prims: &[(Prim, Vec<AtomVal<IdentCtx>>)],
    ) -> HashMap<IdentCtx, LitVal> {
        // fast path for empty solver query
        if prims.is_empty() {
            return HashMap::new();
        }

        // reset solver state
        self.ctx.pop().unwrap();
        self.ctx.push().unwrap();

        let ty_map: HashMap<IdentCtx, LitType> = infer_type(prims);
        let sexp_map = self.declare_vars(&ty_map);
        self.add_constraints(&sexp_map, prims);
        assert_eq!(self.ctx.check().unwrap(), easy_smt::Response::Sat);

        let mut bits_pool: Vec<(IdentCtx, Option<i32>)> = Vec::new();
        for (var, ty) in ty_map.iter() {
            match ty {
                LitType::TyInt => {
                    for i in 0..self.args.int_rep.get_width() {
                        bits_pool.push((*var, Some(i as i32)));
                    }
                }
                LitType::TyFloat => todo!(),
                LitType::TyBool => {
                    bits_pool.push((*var, None));
                }
                LitType::TyChar => todo!(),
            }
        }
        bits_pool.shuffle(rng);

        while !bits_pool.is_empty() {
            let (var, idx) = bits_pool.pop().unwrap();
            let (eq0, eq1) = match idx {
                Some(idx) => (
                    self.ctx.eq(
                        self.ctx.extract(idx, idx, sexp_map[&var]),
                        self.ctx.binary(1, 0),
                    ),
                    self.ctx.eq(
                        self.ctx.extract(idx, idx, sexp_map[&var]),
                        self.ctx.binary(1, 1),
                    ),
                ),
                None => (
                    self.ctx.eq(sexp_map[&var], self.ctx.false_()),
                    self.ctx.eq(sexp_map[&var], self.ctx.true_()),
                ),
            };

            let (eq_try, eq_backup) = if rng.random_bool(0.5) {
                (eq0, eq1)
            } else {
                (eq1, eq0)
            };

            let res = self.ctx.check_assuming(vec![eq_try]).unwrap();
            match res {
                easy_smt::Response::Sat => {
                    self.ctx.assert(eq_try).unwrap();
                    assert_eq!(self.ctx.check().unwrap(), easy_smt::Response::Sat);
                }
                easy_smt::Response::Unsat => {
                    self.ctx.assert(eq_backup).unwrap();
                    assert_eq!(self.ctx.check().unwrap(), easy_smt::Response::Sat);
                }
                easy_smt::Response::Unknown => panic!("SMT solver returns `Unknown`!"),
            }
        }

        assert_eq!(self.ctx.check().unwrap(), easy_smt::Response::Sat);
        let vars: Vec<IdentCtx> = ty_map.keys().copied().collect();
        vars.iter()
            .cloned()
            .zip(
                self.ctx
                    .get_value(vars.iter().map(|var| sexp_map[var]).collect())
                    .unwrap()
                    .iter()
                    .map(|(_var, val)| self.sexp_to_lit_val(*val).unwrap()),
            )
            .collect()
    }
}
