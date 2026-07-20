use super::*;
use crate::logic;
use crate::syntax::ast;

fn compile_type(typ: &ast::Type) -> TermType {
    match typ {
        ast::Type::Lit { lit, span: _ } => Term::Lit(*lit),
        ast::Type::Var { var, span: _ } => Term::Var(var.ident),
        ast::Type::Cons {
            cons,
            flds,
            span: _,
        } => {
            let flds = flds.iter().map(compile_type).collect();
            Term::Cons(OptCons::Some(cons.ident), flds)
        }
        ast::Type::Tuple { flds, span: _ } => {
            let flds: Vec<TermType> = flds.iter().map(compile_type).collect();
            Term::Cons(OptCons::None, flds)
        }
    }
}

fn compile_data_decl(data: &ast::DataDecl) -> (DataDecl, Vec<ConsDecl>) {
    let name = data.name.ident;
    let polys: Vec<_> = data.polys.iter().map(|poly| poly.ident).collect();
    let (cons_decls, cons_names): (Vec<ConsDecl>, Vec<Ident>) = data
        .cons
        .iter()
        .map(|cons| (compile_cons_decl(name, &polys, cons), cons.name.ident))
        .unzip();
    let data_decl = DataDecl {
        name,
        polys,
        conss: cons_names,
    };
    (data_decl, cons_decls)
}

fn compile_cons_decl(data: Ident, polys: &[Ident], cons: &ast::Constructor) -> ConsDecl {
    let name = cons.name.ident;
    let inst_polys: Vec<Ident> = polys.iter().map(|poly| poly.uniquify()).collect();
    let inst_map: HashMap<Ident, TermType> = polys
        .iter()
        .copied()
        .zip(inst_polys.iter().map(|poly| Term::Var(*poly)))
        .collect();
    let pars = cons
        .flds
        .iter()
        .map(|par| compile_type(par).substitute(&inst_map))
        .collect();
    ConsDecl {
        name,
        polys: inst_polys.clone(),
        pars,
        data_cons: data,
        data_args: inst_polys.iter().map(|poly| Term::Var(*poly)).collect(),
    }
}

fn compile_query(query: &ast::QueryDecl) -> QueryDecl {
    QueryDecl {
        entry: query.entry.ident,
        params: query
            .params
            .iter()
            .map(|(param, _span)| compile_query_param(param))
            .collect(),
    }
}

fn compile_query_param(param: &ast::QueryParam) -> QueryParam {
    match param {
        ast::QueryParam::AnswerLimit(x) => QueryParam::AnswerLimit(*x),
        ast::QueryParam::TimeLimit(x) => QueryParam::TimeLimit(*x),
        ast::QueryParam::MemLimit(x) => QueryParam::MemLimit(*x),
    }
}

pub fn compile_pass(prog: &ast::Program) -> Program {
    let mut datas: HashMap<Ident, DataDecl> = HashMap::new();
    let mut conss: HashMap<Ident, ConsDecl> = HashMap::new();
    let mut preds: HashMap<Ident, PredDecl> = HashMap::new();

    for data in prog.datas.iter() {
        let (data_decl, cons_decls) = compile_data_decl(data);
        datas.insert(data_decl.name, data_decl);
        for cons_decl in cons_decls {
            conss.insert(cons_decl.name, cons_decl);
        }
    }

    let goal_pred_decls = translate::logic_translate(&prog.funcs);
    for (pred, pred_decl) in goal_pred_decls.iter() {
        let pred_decl = PredDecl {
            name: *pred,
            polys: pred_decl.polys.clone(),
            pars: pred_decl.pars.clone(),
            rules: logic::normalize::normalize_pred(pred_decl),
        };
        preds.insert(*pred, pred_decl);
    }

    let querys = prog.querys.iter().map(compile_query).collect();

    Program {
        datas,
        conss,
        preds,
        querys,
    }
}
