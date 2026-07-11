use super::*;

use crate::syntax;

const SRC: &'static str = r#"
datatype %Bool where
| %F
| %T
end

function %band(a: %Bool, b: %Bool) -> %Bool
begin
    match (a, b) with
    | (%F, %F) => %F
    | (%F, %T) => %F
    | (%T, %F) => %F
    | (%T, %T) => %T
    end
end

function %bor(a: %Bool, b: %Bool) -> %Bool
begin
    match (a, b) with
    | (%F, %F) => %F
    | (%F, %T) => %T
    | (%T, %F) => %T
    | (%T, %T) => %T
    end
end

function %bnot(a: %Bool) -> %Bool
begin
    match a with
    | %F => %T
    | %T => %F
    end
end

datatype %Bit where
| %O
| %I
end

datatype %BitList where
| %Nil
| %Cons(%Bit, %BitList)
end

datatype %Int where
| %Pos(%BitList)
| %Zero
| %Neg(%BitList)
end

// adder (ci, x, y) -> (z, co)
function %full_adder_1(ci: %Bit, x: %Bit, y: %Bit) -> (%Bit, %Bit) 
begin
    match (ci, x, y) with
    | (%O, %O, %O) => (%O, %O)
    | (%I, %O, %O) => (%I, %O)
    | (%O, %I, %O) => (%I, %O)
    | (%I, %I, %O) => (%O, %I)
    | (%O, %O, %I) => (%I, %O)
    | (%I, %O, %I) => (%O, %I)
    | (%O, %I, %I) => (%O, %I)
    | (%I, %I, %I) => (%I, %I)
    end
end

function %list_add_1(xs: %BitList) -> %BitList
begin
    match xs with
    | %Cons(%O, tail) => %Cons(%I, tail)
    | %Cons(%I, tail) => %Cons(%O, %list_add_1(tail))
    | %Nil => %Cons(%O, %Nil)
    end
end

function %full_adder_n(ci: %Bit, xs: %BitList, ys: %BitList) -> %BitList
begin
    match (xs, ys) with
    | (%Nil, %Nil) => %Cons(ci, %Nil)
    | (%Cons(x_head, x_tail), %Nil) => 
        let (z, co) = %full_adder_1(ci, x_head, %I);
        match co with
        | %O => %Cons(z, x_tail)
        | %I => %Cons(z, %list_add_1(x_tail))
        end
    | (%Nil, %Cons(y_head, y_tail)) => 
        let (z, co) = %full_adder_1(ci, %I, y_head);
        match co with
        | %O => %Cons(z, y_tail)
        | %I => %Cons(z, %list_add_1(y_tail))
        end
    | (%Cons(x_head, x_tail), %Cons(y_head, y_tail)) =>
        let (z, co) = %full_adder_1(ci, x_head, y_head);
        %Cons(z, %full_adder_n(co, x_tail, y_tail))
    end
end

function %list_add(xs: %BitList, ys: %BitList) -> %BitList
begin
    %full_adder_n(%O, xs, ys)
end

function %list_mul(xs: %BitList, ys: %BitList) -> %BitList
begin
    match (xs, ys) with
    | (%Nil, %Nil) => %Nil
    | (%Cons(_, _), %Nil) => xs
    | (%Nil, %Cons(_, _)) => ys
    | (%Cons(x_head, x_tail), %Cons(y_head, y_tail)) =>
        let new_tail = %list_mul(x_tail, y_tail);
        match (x_head, y_head) with
        // (2m)(2n) = 4mn
        | (%O, %O) => %Cons(%O, %Cons(%O, new_tail))
        // (2m)(2n + 1) = 4mn + 2m
        | (%O, %I) => %Cons(%O, %list_add(x_tail, %Cons(%O, new_tail)))
        // (2m + 1)(2n) = 4mn + 2n
        | (%I, %O) => %Cons(%O, %list_add(y_tail, %Cons(%O, new_tail)))
        // (2m + 1)(2n + 1) = 4mn + 2(m + n) + 1
        | (%I, %I) => %Cons(%I, %list_add(%list_add(x_tail, y_tail), %Cons(%O, new_tail)))
        end
    end
end

function %list_sub(xs: %BitList, ys: %BitList) -> %Int
begin
    fresh zs;
    alternative
    // xs + (-ys) = (+zs) ===> ys + zs = xs
    | guard xs = %full_adder_n(%O, ys, zs); %Pos(zs)
    // xs + (-ys) = (-zs) ===> (-xs) + ys = zs ===> xs + zs = ys
    | guard ys = %full_adder_n(%O, xs, zs); %Neg(zs)
    // xs + (-ys) = 0 ===> xs = ys
    | guard xs = ys; %Zero
    end
end

function %ineg(x: %Int) -> %Int
begin
    match x with
    | %Pos(xs) => %Neg(xs)
    | %Zero => %Zero
    | %Neg(xs) => %Pos(xs)
    end
end

function %iadd(x: %Int, y: %Int) -> %Int
begin
    match (x, y) with
    | (%Pos(xs), %Pos(ys)) => %Pos(%full_adder_n(%O, xs, ys))
    | (%Pos(xs), %Zero) => %Pos(xs)
    | (%Pos(xs), %Neg(ys)) => %list_sub(xs, ys)
    | (%Zero, ys) => ys
    | (%Neg(xs), %Pos(ys)) => %list_sub(ys, xs)
    | (%Neg(xs), %Zero) => %Neg(xs)
    | (%Neg(xs), %Neg(ys)) => %Neg(%full_adder_n(%O, xs, ys))
    end 
end

function %imul(x: %Int, y: %Int) -> %Int
begin
    match (x, y) with
    | (%Pos(xs), %Pos(ys)) => %Pos(%list_mul(xs, ys))
    | (%Pos(_xs), %Zero) => %Zero
    | (%Pos(xs), %Neg(ys)) => %Neg(%list_mul(xs, ys))
    | (%Zero, _) => %Zero
    | (%Neg(xs), %Pos(ys)) => %Neg(%list_mul(xs, ys))
    | (%Neg(_xs), %Zero) => %Zero
    | (%Neg(xs), %Neg(ys)) => %Pos(%list_mul(xs, ys))
    end 
end

function %isub(x: %Int, y: %Int) -> %Int
begin
    %iadd(x, %ineg(y))
end


datatype %Compare where
| %Lt
| %Eq
| %Gt
end

function %bit_cmp(x: %Bit, y: %Bit) -> %Compare
begin
    match (x, y) with
    | (%O, %O) => %Eq
    | (%O, %I) => %Lt
    | (%I, %O) => %Gt
    | (%I, %I) => %Eq
    end
end

function %list_cmp_help(st: %Compare, xs: %BitList, ys: %BitList) -> %Compare
begin
    match (xs, ys) with
    | (%Nil, %Nil) => st
    | (%Nil, %Cons(_, _)) => %Lt
    | (%Cons(_, _), %Nil) => %Gt
    | (%Cons(x_head, x_tail), %Cons(y_head, y_tail)) =>
        match %bit_cmp(x_head, y_head) with
        | %Lt => %list_cmp_help(%Lt, x_tail, y_tail)
        | %Eq => %list_cmp_help(st, x_tail, y_tail)
        | %Gt => %list_cmp_help(%Gt, x_tail, y_tail)
        end
    end
end

function %list_cmp(xs: %BitList, ys: %BitList) -> %Compare
begin
    %list_cmp_help(%Eq, xs, ys)
end

function %int_cmp(x: %Int, y: %Int) -> %Compare
begin
    match (x, y) with
    | (%Zero, %Zero) => %Eq
    | (%Zero, %Pos(_)) => %Lt
    | (%Zero, %Neg(_)) => %Gt
    | (%Pos(_), %Zero) => %Gt
    | (%Pos(xs), %Pos(ys)) => %list_cmp(xs, ys)
    | (%Pos(_), %Neg(_)) => %Gt
    | (%Neg(_), %Zero) => %Lt
    | (%Neg(_), %Pos(_)) => %Lt
    | (%Neg(xs), %Neg(ys)) =>
        match %list_cmp(xs, ys) with
        | %Lt => %Gt
        | %Eq => %Eq
        | %Gt => %Lt
        end
    end
end

function %icmpeq(x: %Int, y: %Int) -> %Bool
begin
    match %int_cmp(x, y) with
    | %Lt => %F
    | %Eq => %T
    | %Gt => %F
    end
end

function %icmplt(x: %Int, y: %Int) -> %Bool
begin
    match %int_cmp(x, y) with
    | %Lt => %T
    | %Eq => %F
    | %Gt => %F
    end
end

function %icmple(x: %Int, y: %Int) -> %Bool
begin
    match %int_cmp(x, y) with
    | %Lt => %T
    | %Eq => %T
    | %Gt => %F
    end
end

function %icmpgt(x: %Int, y: %Int) -> %Bool
begin
    match %int_cmp(x, y) with
    | %Lt => %F
    | %Eq => %F
    | %Gt => %T
    end
end

function %icmpge(x: %Int, y: %Int) -> %Bool
begin
    match %int_cmp(x, y) with
    | %Lt => %F
    | %Eq => %T
    | %Gt => %T
    end
end

function %icmpne(x: %Int, y: %Int) -> %Bool
begin
    match %int_cmp(x, y) with
    | %Lt => %T
    | %Eq => %F
    | %Gt => %T
    end
end

"#;

pub struct BuiltinMap {
    map: HashMap<&'static str, Ident>,
}

impl BuiltinMap {
    fn new() -> BuiltinMap {
        BuiltinMap {
            map: HashMap::new(),
        }
    }

    fn replace_lit_type(&self, typ: &mut TermType) {
        match typ {
            TermType::Lit(LitType::TyInt) => {
                *typ = TermType::Cons(OptCons::Some(self.map["%Int"]), vec![]);
            }
            TermType::Lit(LitType::TyBool) => {
                *typ = TermType::Cons(OptCons::Some(self.map["%Bool"]), vec![]);
            }
            TermType::Cons(_, flds) => {
                for fld in flds {
                    self.replace_lit_type(fld);
                }
            }
            _ => {}
        }
    }

    fn uint_to_bitlist(&self, n: u64) -> TermVal {
        if n == 1 {
            return Term::Cons(OptCons::Some(self.map["%Nil"]), vec![]);
        }
        let bit = if n & 1 == 0 {
            self.map["%O"]
        } else {
            self.map["%I"]
        };
        let tail = self.uint_to_bitlist(n >> 1);
        Term::Cons(
            OptCons::Some(self.map["%Cons"]),
            vec![Term::Cons(OptCons::Some(bit), vec![]), tail],
        )
    }

    fn int_to_bitint(&self, n: i64) -> TermVal {
        if n == 0 {
            return Term::Cons(OptCons::Some(self.map["%Zero"]), vec![]);
        }
        let (sign_ident, abs_n) = if n > 0 {
            (self.map["%Pos"], n as u64)
        } else {
            (self.map["%Neg"], n.unsigned_abs())
        };
        let bitlist = self.uint_to_bitlist(abs_n);
        Term::Cons(OptCons::Some(sign_ident), vec![bitlist])
    }

    fn replace_lit_val(&self, term: &mut TermVal) {
        match term {
            Term::Lit(LitVal::Int(n)) => {
                *term = self.int_to_bitint(*n);
            }
            Term::Lit(LitVal::Bool(true)) => {
                *term = Term::Cons(OptCons::Some(self.map["%T"]), vec![]);
            }
            Term::Lit(LitVal::Bool(false)) => {
                *term = Term::Cons(OptCons::Some(self.map["%F"]), vec![]);
            }
            Term::Cons(_, flds) => {
                for fld in flds {
                    self.replace_lit_val(fld);
                }
            }
            _ => {}
        }
    }

    fn atom_to_termval(&self, atom: &AtomVal) -> TermVal {
        match atom {
            Term::Var(v) => Term::Var(*v),
            Term::Lit(LitVal::Int(n)) => self.int_to_bitint(*n),
            Term::Lit(LitVal::Bool(true)) => Term::Cons(OptCons::Some(self.map["%T"]), vec![]),
            Term::Lit(LitVal::Bool(false)) => Term::Cons(OptCons::Some(self.map["%F"]), vec![]),
            Term::Lit(l) => Term::Lit(*l),
            _ => unreachable!(),
        }
    }
}

fn prim_name(prim: &Prim) -> &'static str {
    match prim {
        Prim::IAdd => "%iadd",
        Prim::ISub => "%isub",
        Prim::IMul => "%imul",
        Prim::IDiv => todo!("IDiv is not supported yet!"),
        Prim::IRem => todo!("IRem is not supported yet!"),
        Prim::INeg => "%ineg",
        Prim::ICmp(Compare::Lt) => "%icmplt",
        Prim::ICmp(Compare::Le) => "%icmple",
        Prim::ICmp(Compare::Eq) => "%icmpeq",
        Prim::ICmp(Compare::Ge) => "%icmpge",
        Prim::ICmp(Compare::Gt) => "%icmpgt",
        Prim::ICmp(Compare::Ne) => "%icmpne",
        Prim::BAnd => "%band",
        Prim::BOr => "%bor",
        Prim::BNot => "%bnot",
    }
}

impl Program {
    pub fn extend_builtin(&mut self) -> BuiltinMap {
        let (mut prog, errs) = syntax::parser::parse_program(SRC);
        assert!(errs.is_empty());

        let errs = crate::tych::rename::rename_pass(&mut prog);
        assert!(errs.is_empty());

        let errs = crate::tych::check::check_pass(&mut prog);
        assert!(errs.is_empty());

        let prog = super::compile::compile_pass(&prog);

        let mut map = BuiltinMap::new();
        for (_name, data) in &prog.datas {
            map.map.insert(data.name.name.as_str(), data.name);
            for cons in &data.cons {
                map.map.insert(cons.name.name.as_str(), cons.name);
            }
        }
        for (_name, pred) in &prog.preds {
            map.map.insert(pred.name.name.as_str(), pred.name);
        }
        self.datas.extend(prog.datas);
        self.preds.extend(prog.preds);
        map
    }

    pub fn replace_builtin(&mut self, map: &BuiltinMap) {
        for (_name, data) in &mut self.datas {
            for cons in &mut data.cons {
                for fld in &mut cons.flds {
                    map.replace_lit_type(fld);
                }
            }
        }

        for (_name, pred) in &mut self.preds {
            for (_name, typ) in &mut pred.pars {
                map.replace_lit_type(typ);
            }
            for rule in &mut pred.rules {
                for (_name, typ) in &mut rule.vars {
                    map.replace_lit_type(typ);
                }
                for head in &mut rule.head {
                    map.replace_lit_val(head);
                }
                let mut new_calls: Vec<(Ident, Vec<TermType>, Vec<TermVal>)> = Vec::new();
                for (prim, args) in rule.prims.drain(..) {
                    new_calls.push((
                        map.map[prim_name(&prim)],
                        vec![],
                        args.iter().map(|a| map.atom_to_termval(&a)).collect(),
                    ));
                }
                rule.calls.extend(new_calls);
                for (_call_pred, call_polys, call_args) in &mut rule.calls {
                    for poly in call_polys {
                        map.replace_lit_type(poly);
                    }
                    for arg in call_args {
                        map.replace_lit_val(arg);
                    }
                }
            }
        }
    }
}
