use super::*;

use crate::utils::prim::Prim;
use crate::utils::unify::*;

#[derive(Clone, Debug)]
struct FuncTyScm {
    polys: Vec<Ident>,
    pars: Vec<TermType>,
    res: TermType,
}

#[derive(Clone, Debug)]
struct ConsTyScm {
    polys: Vec<Ident>,
    flds: Vec<TermType>,
    res: TermType,
}

#[allow(unused)]
#[derive(Clone, Debug)]
struct DataTyScm {
    polys: Vec<Ident>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CheckError {
    UnifyFailed {
        typ1: TermType,
        typ2: TermType,
        span: Span,
    },
    OccurCheckFailed {
        var: Ident,
        typ: TermType,
        span: Span,
    },
    UnifyVecDiffLen {
        vec1: Vec<TermType>,
        vec2: Vec<TermType>,
        span: Span,
    },
    TypeArityMismatch {
        actual: usize,
        expected: usize,
        span: Span,
    },
}

use crate::cli::diagnostic::Diagnostic;
impl From<CheckError> for Diagnostic {
    fn from(val: CheckError) -> Self {
        match val {
            CheckError::UnifyFailed {typ1, typ2, span } => {
                Diagnostic::error("cannot match type!".to_string()).line_span(
                    span.clone(),
                    format!("the expression here has type {typ1}, but expected {typ2}."),
                )
            }
            CheckError::OccurCheckFailed { var, typ, span } => {
                Diagnostic::error("occurrence check failed!".to_string()).line_span(
                    span.clone(),
                    format!("failed to unify the variable {var} with type {typ}, since it occurs in its own type."),
                )
            }
            CheckError::UnifyVecDiffLen { vec1, vec2, span } => {
                Diagnostic::error("type vectors have different length!".to_string()).line_span(
                    span.clone(),
                    format!("failed to unify two vectors with lengths: {vec1:?} and {vec2:?}"),
                )
            }
            CheckError::TypeArityMismatch { actual, expected, span } => {
                Diagnostic::error("type arity mismatch!".to_string()).line_span(
                    span.clone(),
                    format!("the type constructor has arity {actual}, but expected arity {expected}."),
                )
            }
        }
    }
}

struct Checker {
    val_ctx: HashMap<Ident, TermType>,
    func_ctx: HashMap<Ident, FuncTyScm>,
    cons_ctx: HashMap<Ident, ConsTyScm>,
    data_ctx: HashMap<Ident, DataTyScm>,
    unifier: Unifier<Ident, LitType, OptCons<Ident>>,
    errors: Vec<CheckError>,
}

impl Checker {
    pub fn new() -> Checker {
        Checker {
            val_ctx: HashMap::new(),
            func_ctx: HashMap::new(),
            cons_ctx: HashMap::new(),
            data_ctx: HashMap::new(),
            unifier: Unifier::new(),
            errors: Vec::new(),
        }
    }

    fn fresh(&mut self) -> TermType {
        TermType::Var(Ident::fresh(&"a"))
    }

    fn unify(&mut self, typ1: &TermType, typ2: &TermType, span: &Span) {
        match self.unifier.unify(typ1, typ2) {
            Ok(()) => {}
            Err(UnifyError::UnifyFailed(typ1, typ2)) => {
                self.errors.push(CheckError::UnifyFailed {
                    typ1,
                    typ2,
                    span: span.clone(),
                });
            }
            Err(UnifyError::OccurCheckFailed(var, typ)) => {
                self.errors.push(CheckError::OccurCheckFailed {
                    var,
                    typ,
                    span: span.clone(),
                });
            }
            Err(UnifyError::UnifyVecDiffLen(vec1, vec2)) => {
                self.errors.push(CheckError::UnifyVecDiffLen {
                    vec1,
                    vec2,
                    span: span.clone(),
                });
            }
        }
    }

    fn unify_many(&mut self, vec1: &[TermType], vec2: &[(TermType, Span)], span: &Span) {
        if vec1.len() == vec2.len() {
            for (lhs, (rhs, span)) in vec1.iter().zip(vec2.iter()) {
                self.unify(lhs, rhs, span);
            }
        } else {
            self.errors.push(CheckError::UnifyVecDiffLen {
                vec1: vec1.to_vec(),
                vec2: vec2.iter().map(|x| x.0.clone()).collect(),
                span: span.clone(),
            });
        }
    }

    fn check_prim(&mut self, prim: &Prim, args: &[Expr], span: &Span) -> TermType {
        let args: Vec<_> = args
            .iter()
            .map(|arg| (self.infer_expr(arg), arg.get_span()))
            .collect();

        match prim {
            Prim::IAdd | Prim::ISub | Prim::IMul | Prim::IDiv | Prim::IRem => {
                self.unify_many(
                    &[TermType::Lit(LitType::TyInt), TermType::Lit(LitType::TyInt)],
                    &args,
                    span,
                );
                TermType::Lit(LitType::TyInt)
            }
            Prim::INeg => {
                self.unify_many(&[TermType::Lit(LitType::TyInt)], &args, span);
                TermType::Lit(LitType::TyInt)
            }
            Prim::ICmp(_) => {
                self.unify_many(
                    &[TermType::Lit(LitType::TyInt), TermType::Lit(LitType::TyInt)],
                    &args,
                    span,
                );
                TermType::Lit(LitType::TyBool)
            }
            Prim::BAnd | Prim::BOr => {
                self.unify_many(
                    &[
                        TermType::Lit(LitType::TyBool),
                        TermType::Lit(LitType::TyBool),
                    ],
                    &args,
                    span,
                );
                TermType::Lit(LitType::TyBool)
            }
            Prim::BNot => {
                self.unify_many(&[TermType::Lit(LitType::TyBool)], &args, span);
                TermType::Lit(LitType::TyBool)
            }
        }
    }

    fn infer_expr(&mut self, expr: &Expr) -> TermType {
        match expr {
            Expr::Lit { lit, span: _ } => TermType::Lit(lit.get_typ()),
            Expr::Var { var, span: _ } => self.val_ctx[&var.ident].clone(),
            Expr::Prim { prim, args, span } => self.check_prim(prim, args, span),
            Expr::Cons { cons, flds, span } => {
                // instantiate constructor type scheme
                let cons_scm = &self.cons_ctx[&cons.ident];

                let inst_map: HashMap<Ident, TermType> = cons_scm
                    .polys
                    .iter()
                    .map(|poly| (*poly, Term::Var(poly.uniquify())))
                    .collect();

                let inst_flds: Vec<_> = cons_scm
                    .flds
                    .iter()
                    .map(|fld| fld.substitute(&inst_map))
                    .collect();

                let inst_res = cons_scm.res.substitute(&inst_map);

                let flds: Vec<_> = flds
                    .iter()
                    .map(|fld| (self.infer_expr(fld), fld.get_span()))
                    .collect();

                self.unify_many(&inst_flds, &flds, span);
                inst_res
            }
            Expr::Tuple { flds, span: _ } => {
                let flds: Vec<TermType> = flds.iter().map(|fld| self.infer_expr(fld)).collect();
                TermType::Cons(OptCons::None, flds)
            }
            Expr::Match {
                expr,
                brchs,
                span: _,
            } => {
                let expr_ty = self.infer_expr(expr);
                let res = self.fresh();
                for (patn, cont) in brchs {
                    let patn_ty = self.check_patn(patn);
                    let patn_span = patn.get_span();
                    self.unify(&patn_ty, &expr_ty, &patn_span);
                    let cont_ty = self.infer_expr(cont);
                    let cont_span = cont.get_span();
                    self.unify(&res, &cont_ty, &cont_span);
                }
                res
            }
            Expr::Let {
                patn,
                expr,
                cont,
                span: _,
            } => {
                let expr_ty = self.infer_expr(expr);
                let expr_span = expr.get_span();
                let patn_ty = self.check_patn(patn);
                self.unify(&patn_ty, &expr_ty, &expr_span);
                self.infer_expr(cont)
            }
            Expr::App { func, args, span } => {
                // instantiate predicate type scheme
                let func_scm = &self.func_ctx[&func.ident];

                let inst_map: HashMap<Ident, TermType> = func_scm
                    .polys
                    .iter()
                    .map(|poly| (*poly, Term::Var(poly.uniquify())))
                    .collect();

                let inst_pars: Vec<_> = func_scm
                    .pars
                    .iter()
                    .map(|par| par.substitute(&inst_map))
                    .collect();

                let inst_res = func_scm.res.substitute(&inst_map);

                let args: Vec<_> = args
                    .iter()
                    .map(|arg| (self.infer_expr(arg), arg.get_span()))
                    .collect();

                self.unify_many(&inst_pars, &args, span);
                inst_res
            }
            Expr::Ifte {
                cond,
                then,
                els,
                span: _,
            } => {
                let cond_ty = self.infer_expr(cond);
                let cond_span = cond.get_span();
                self.unify(&cond_ty, &TermType::Lit(LitType::TyBool), &cond_span);
                let then_ty = self.infer_expr(then);
                let els_ty = self.infer_expr(els);
                let els_span = els.get_span();
                self.unify(&then_ty, &els_ty, &els_span);
                then_ty
            }
            Expr::Cond { brchs, span: _ } => {
                let res = self.fresh();
                for (cond, body) in brchs {
                    let cond_ty = self.infer_expr(cond);
                    let cond_span = cond.get_span();
                    let body_ty = self.infer_expr(body);
                    let body_span = body.get_span();
                    self.unify(&cond_ty, &TermType::Lit(LitType::TyBool), &cond_span);
                    self.unify(&body_ty, &res, &body_span);
                }
                res
            }
            Expr::Alter { brchs, span: _ } => {
                let res = self.fresh();
                for body in brchs {
                    let body_ty = self.infer_expr(body);
                    let body_span = body.get_span();
                    self.unify(&body_ty, &res, &body_span);
                }
                res
            }
            Expr::Fresh {
                vars,
                cont,
                span: _,
            } => {
                for var in vars {
                    let cell = self.fresh();
                    self.val_ctx.insert(var.ident, cell);
                }
                self.infer_expr(cont)
            }
            Expr::Guard {
                lhs,
                rhs,
                cont,
                span: _,
            } => {
                let lhs_ty = self.infer_expr(lhs);
                if let Some(rhs) = rhs {
                    let rhs_ty = self.infer_expr(rhs);
                    let rhs_span = rhs.get_span();
                    self.unify(&lhs_ty, &rhs_ty, &rhs_span);
                } else {
                    let lhs_span = lhs.get_span();
                    self.unify(
                        &lhs_ty,
                        &TermType::Cons(OptCons::None, Vec::new()),
                        &lhs_span,
                    );
                }
                self.infer_expr(cont)
            }
            Expr::Undefined { span: _ } => self.fresh(),
        }
    }

    fn check_patn(&mut self, patn: &Pattern) -> TermType {
        match patn {
            Pattern::Lit { lit, span: _ } => TermType::Lit(lit.get_typ()),
            Pattern::Var { var, span: _ } => {
                let ty = self.fresh();
                self.val_ctx.insert(var.ident, ty.clone());
                ty
            }
            Pattern::Cons { cons, flds, span } => {
                // instantiate constructor type scheme
                let cons_scm = &self.cons_ctx[&cons.ident];

                let inst_map: HashMap<Ident, TermType> = cons_scm
                    .polys
                    .iter()
                    .map(|poly| (*poly, Term::Var(poly.uniquify())))
                    .collect();

                let inst_flds: Vec<_> = cons_scm
                    .flds
                    .iter()
                    .map(|fld| fld.substitute(&inst_map))
                    .collect();

                let inst_res = cons_scm.res.substitute(&inst_map);

                let flds: Vec<_> = flds
                    .iter()
                    .map(|fld| (self.check_patn(fld), fld.get_span()))
                    .collect();

                self.unify_many(&inst_flds, &flds, span);
                inst_res
            }
            Pattern::Tuple { flds, span: _ } => {
                let typs: Vec<TermType> = flds.iter().map(|fld| self.check_patn(fld)).collect();
                TermType::Cons(OptCons::None, typs)
            }
        }
    }

    fn check_type(&mut self, typ: &Type) -> TermType {
        match typ {
            Type::Lit { lit, span: _ } => Term::Lit(*lit),
            Type::Var { var, span: _ } => Term::Var(var.ident),
            Type::Cons {
                cons,
                flds,
                span: _,
            } => {
                let flds: Vec<_> = flds.iter().map(|fld| self.check_type(fld)).collect();
                let data_scm = &self.data_ctx[&cons.ident];
                if flds.len() != data_scm.polys.len() {
                    self.errors.push(CheckError::TypeArityMismatch {
                        actual: flds.len(),
                        expected: data_scm.polys.len(),
                        span: typ.get_span(),
                    });
                }
                Term::Cons(OptCons::Some(cons.ident), flds)
            }
            Type::Tuple { flds, span: _ } => {
                let flds: Vec<TermType> = flds.iter().map(|fld| self.check_type(fld)).collect();
                Term::Cons(OptCons::None, flds)
            }
        }
    }

    fn scan_data_ty_scm(&mut self, data_decl: &DataDecl) {
        for poly in &data_decl.polys {
            self.unifier.fresh(poly.ident);
        }
        let data_scm = DataTyScm {
            polys: data_decl.polys.iter().map(|poly| poly.ident).collect(),
        };
        self.data_ctx.insert(data_decl.name.ident, data_scm);
    }

    fn scan_cons_ty_scm(&mut self, data_decl: &DataDecl) {
        let res = TermType::Cons(
            OptCons::Some(data_decl.name.ident),
            data_decl
                .polys
                .iter()
                .map(|poly| TermType::Var(poly.ident))
                .collect(),
        );

        for cons in &data_decl.cons {
            let flds = cons.flds.iter().map(|fld| self.check_type(fld)).collect();
            let cons_typ = ConsTyScm {
                polys: data_decl.polys.iter().map(|poly| poly.ident).collect(),
                flds,
                res: res.clone(),
            };
            self.cons_ctx.insert(cons.name.ident, cons_typ);
        }
    }

    fn scan_func_ty_scm(&mut self, func_decl: &FuncDecl) {
        for poly in &func_decl.polys {
            self.unifier.fresh(poly.ident);
        }

        let polys = func_decl.polys.iter().map(|poly| poly.ident).collect();
        let pars = func_decl
            .pars
            .iter()
            .map(|(_par, typ)| self.check_type(typ))
            .collect();

        let res = self.check_type(&func_decl.res);
        let func_scm = FuncTyScm { polys, pars, res };
        self.func_ctx.insert(func_decl.name.ident, func_scm);
    }

    fn check_func_decl(&mut self, func_decl: &FuncDecl) {
        let func_scm = self.func_ctx[&func_decl.name.ident].clone();
        for ((par, _), par_ty) in func_decl.pars.iter().zip(func_scm.pars.iter()) {
            self.val_ctx.insert(par.ident, par_ty.clone());
        }
        let body_ty = self.infer_expr(&func_decl.body);
        let body_span = func_decl.body.get_span();
        self.unify(&func_scm.res, &body_ty, &body_span);
    }

    fn check_prog(&mut self, prog: &Program) {
        for data_decl in &prog.datas {
            self.scan_data_ty_scm(data_decl);
        }

        for data_decl in &prog.datas {
            self.scan_cons_ty_scm(data_decl);
        }

        for func_decl in &prog.funcs {
            self.scan_func_ty_scm(func_decl);
        }

        for func_decl in &prog.funcs {
            self.check_func_decl(func_decl);
        }
    }
}

pub fn check_pass(prog: &Program) -> Vec<CheckError> {
    let mut pass = Checker::new();
    pass.check_prog(prog);
    let mut errors = std::mem::take(&mut pass.errors);
    for err in &mut errors {
        match err {
            CheckError::UnifyFailed {
                typ1,
                typ2,
                span: _,
            } => {
                *typ1 = pass.unifier.subst(typ1);
                *typ2 = pass.unifier.subst(typ2);
            }
            CheckError::OccurCheckFailed {
                var: _,
                typ,
                span: _,
            } => {
                *typ = pass.unifier.subst(typ);
            }
            CheckError::UnifyVecDiffLen {
                vec1,
                vec2,
                span: _,
            } => {
                *vec1 = vec1.iter().map(|t| pass.unifier.subst(t)).collect();
                *vec2 = vec2.iter().map(|t| pass.unifier.subst(t)).collect();
            }
            CheckError::TypeArityMismatch {
                actual: _,
                expected: _,
                span: _,
            } => {
                // do nothing
            }
        }
    }
    errors
}
