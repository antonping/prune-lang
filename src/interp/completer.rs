use super::*;
use crate::interp::branch::{Answer, walk_free_var};
use rand::seq::SliceRandom;

struct Completer<'prog, 'args, 'rng> {
    prog: &'prog Program,
    args: &'args CliArgs,
    rng: &'rng mut rand::rngs::ThreadRng,
    map: HashMap<IdentCtx, TermVal<IdentCtx>>,
}

impl<'prog, 'args, 'rng> Completer<'prog, 'args, 'rng> {
    fn new(
        prog: &'prog Program,
        args: &'args CliArgs,
        rng: &'rng mut rand::rngs::ThreadRng,
    ) -> Self {
        Completer {
            prog,
            args,
            rng,
            map: HashMap::new(),
        }
    }

    fn complete_value(&mut self, var: IdentCtx, ty: &TermType) {
        if self.map.contains_key(&var) {
            return;
        }
        loop {
            let size = self.rng.random_range(1..=50);
            if let Some(val) = self.generate_sized_value(ty, size) {
                self.map.insert(var, val);
                return;
            }
        }
    }

    fn generate_sized_value(&mut self, ty: &TermType, size: usize) -> Option<TermVal<IdentCtx>> {
        if size < 1 {
            return None;
        }
        match ty {
            Term::Lit(LitType::TyInt) => match self.args.int_rep {
                args::IntRep::BV8 => Some(Term::Lit(LitVal::Int(self.rng.random::<i8>() as i32))),
                args::IntRep::BV16 => Some(Term::Lit(LitVal::Int(self.rng.random::<i16>() as i32))),
                args::IntRep::BV32 => Some(Term::Lit(LitVal::Int(self.rng.random::<i32>()))),
            },
            Term::Lit(LitType::TyFloat) => Some(Term::Lit(LitVal::Float(self.rng.random()))),
            Term::Lit(LitType::TyBool) => Some(Term::Lit(LitVal::Bool(self.rng.random()))),
            Term::Lit(LitType::TyChar) => Some(Term::Lit(LitVal::Char(self.rng.random()))),
            Term::Cons(OptCons::None, ty_args) => {
                let sizes = self.distribute_size(ty_args.len(), size - 1)?;
                let mut args = Vec::new();
                for (ty, &size) in ty_args.iter().zip(sizes.iter()) {
                    args.push(self.generate_sized_value(ty, size)?);
                }
                Some(Term::Cons(OptCons::None, args))
            }
            Term::Cons(OptCons::Some(data_name), ty_args) => {
                let data = &self.prog.datas[&data_name].clone();
                let mut conss = data.conss.clone();
                conss.shuffle(&mut self.rng);
                for cons_id in &conss {
                    let cons = &self.prog.conss[cons_id];
                    let subst: HashMap<Ident, TermType> = cons
                        .polys
                        .iter()
                        .zip(ty_args.iter())
                        .map(|(poly, arg)| (*poly, arg.clone()))
                        .collect();

                    let inst_pars: Vec<TermType> =
                        cons.pars.iter().map(|par| par.substitute(&subst)).collect();

                    let sizes = self.distribute_size(inst_pars.len(), size - 1)?;
                    let mut args = Vec::new();
                    for (ty, &size) in inst_pars.iter().zip(sizes.iter()) {
                        args.push(self.generate_sized_value(ty, size)?);
                    }

                    return Some(Term::Cons(OptCons::Some(*cons_id), args));
                }

                None
            }
            Term::Var(var) => {
                panic!("type variable {var} at runtime!");
            }
        }
    }

    fn distribute_size(&mut self, n: usize, budget: usize) -> Option<Vec<usize>> {
        if n == 0 {
            return Some(Vec::new());
        }
        if budget < n {
            return None;
        }
        let mut sizes = vec![1usize; n];
        let mut remaining = budget - n;
        while remaining > 0 {
            let idx = self.rng.random_range(0..n);
            sizes[idx] += 1;
            remaining -= 1;
        }
        Some(sizes)
    }
}

pub fn answer_complete(
    prog: &Program,
    args: &CliArgs,
    rng: &mut rand::rngs::ThreadRng,
    ansrs: &mut Vec<Answer>,
) {
    let mut map = HashMap::new();
    for ansr in ansrs.iter() {
        walk_free_var(prog, &ansr.val, &ansr.ty, &mut map);
    }
    // println!("map: {:?}", map);

    let mut completer = Completer::new(prog, args, rng);
    for (var, ty) in map.into_iter() {
        completer.complete_value(var, &ty);
    }

    for ansr in ansrs.iter_mut() {
        ansr.val = ansr.val.substitute(&completer.map);
    }
}
