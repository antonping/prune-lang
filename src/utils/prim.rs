use std::fmt;
use std::str::FromStr;

use super::lit::LitType;

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub enum Compare {
    Lt,
    Le,
    Eq,
    Ge,
    Gt,
    Ne,
}

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub enum Prim {
    /// integer arithmetics
    IAdd,
    ISub,
    IMul,
    IDiv,
    IRem,
    INeg,

    /// float-point arithmetics
    // FAdd,
    // FSub,
    // FMul,
    // FDiv,
    // FNeg,

    /// comparision
    ICmp(Compare),

    /// boolean operation
    BAnd,
    BOr,
    BNot,
}

impl Prim {
    pub fn get_typ(&self) -> Vec<LitType> {
        match self {
            Prim::IAdd | Prim::ISub | Prim::IMul | Prim::IDiv | Prim::IRem => {
                vec![LitType::TyInt, LitType::TyInt, LitType::TyInt]
            }
            Prim::INeg => {
                vec![LitType::TyInt, LitType::TyInt]
            }
            Prim::ICmp(_) => {
                vec![LitType::TyInt, LitType::TyInt, LitType::TyBool]
            }
            Prim::BAnd | Prim::BOr => {
                vec![LitType::TyBool, LitType::TyBool, LitType::TyBool]
            }
            Prim::BNot => {
                vec![LitType::TyBool, LitType::TyBool]
            }
        }
    }

    pub fn get_prior(&self) -> u8 {
        match self {
            Prim::IAdd => 3,
            Prim::ISub => 3,
            Prim::IMul => 4,
            Prim::IDiv => 4,
            Prim::IRem => 4,
            Prim::INeg => 0,
            Prim::BAnd => 1,
            Prim::BOr => 1,
            Prim::BNot => 0,
            Prim::ICmp(_) => 2,
        }
    }
}

impl fmt::Display for Prim {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Prim::IAdd => "iadd".fmt(f),
            Prim::ISub => "isub".fmt(f),
            Prim::IMul => "imul".fmt(f),
            Prim::IDiv => "idiv".fmt(f),
            Prim::IRem => "irem".fmt(f),
            Prim::INeg => "ineg".fmt(f),
            Prim::ICmp(Compare::Lt) => "icmplt".fmt(f),
            Prim::ICmp(Compare::Le) => "icmple".fmt(f),
            Prim::ICmp(Compare::Eq) => "icmpeq".fmt(f),
            Prim::ICmp(Compare::Ge) => "icmpge".fmt(f),
            Prim::ICmp(Compare::Gt) => "icmpgt".fmt(f),
            Prim::ICmp(Compare::Ne) => "icmpne".fmt(f),
            Prim::BAnd => "band".fmt(f),
            Prim::BOr => "bor".fmt(f),
            Prim::BNot => "bnot".fmt(f),
        }
    }
}

impl FromStr for Prim {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "iadd" => Ok(Prim::IAdd),
            "isub" => Ok(Prim::ISub),
            "imul" => Ok(Prim::IMul),
            "idiv" => Ok(Prim::IDiv),
            "irem" => Ok(Prim::IRem),
            "ineg" => Ok(Prim::INeg),
            "icmplt" => Ok(Prim::ICmp(Compare::Lt)),
            "icmple" => Ok(Prim::ICmp(Compare::Le)),
            "icmpeq" => Ok(Prim::ICmp(Compare::Eq)),
            "icmpge" => Ok(Prim::ICmp(Compare::Ge)),
            "icmpgt" => Ok(Prim::ICmp(Compare::Gt)),
            "icmpne" => Ok(Prim::ICmp(Compare::Ne)),
            "band" => Ok(Prim::BAnd),
            "bor" => Ok(Prim::BOr),
            "bnot" => Ok(Prim::BNot),
            _ => Err(()),
        }
    }
}
