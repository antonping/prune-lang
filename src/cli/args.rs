use crate::utils::lit::LitVal;

use super::*;
use clap::{Parser, ValueEnum};

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
pub enum Solver {
    Z3,
    CVC5,
    Bitwuzla,
    NoSmt,
}

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
pub enum IntRep {
    BV8,
    BV16,
    BV32,
}

impl IntRep {
    pub fn get_width(&self) -> usize {
        match self {
            IntRep::BV8 => 8,
            IntRep::BV16 => 16,
            IntRep::BV32 => 32,
        }
    }
}

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
pub enum Heuristic {
    LeftBiased,
    Interleave,
    SmallFirst,
    Hybrid,
    Random,
}

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    pub input: PathBuf,

    #[arg(long, default_value = "no-smt", value_name = "SOLVER")]
    pub solver: Solver,

    #[arg(long, default_value = "bv16", value_name = "INT_REP")]
    pub int_rep: IntRep,

    #[arg(long, default_value = "hybrid", value_name = "HEURISTIC")]
    pub heuristic: Heuristic,

    #[arg(long, default_value_t = usize::MAX, value_name = "TIME(s)")]
    pub time_limit: usize,

    #[arg(long, default_value_t = 1000, value_name = "TIME(ms)")]
    pub time_limit_per: usize,

    #[arg(long, default_value_t = usize::MAX, value_name = "INT")]
    pub answer_limit: usize,

    #[arg(long, default_value_t = usize::MAX, value_name = "INT")]
    pub depth_limit: usize,

    #[arg(long, default_value_t = 5, value_name = "INT")]
    pub depth_range: usize,

    #[arg(long, default_value_t = 5, value_name = "INT")]
    pub depth_grow: usize,

    #[arg(short, long, default_value_t = 10, value_name = "INT")]
    pub verbosity: u8,

    #[arg(long, default_value_t = false, action = clap::ArgAction::SetTrue)]
    pub dump_file: bool,

    #[arg(long, default_value_t = false, action = clap::ArgAction::SetTrue)]
    pub debug_mode: bool,

    #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
    pub show_output: bool,

    #[arg(long, default_value_t = false, action = clap::ArgAction::SetTrue)]
    pub show_stat: bool,

    #[arg(long, default_value_t = false, action = clap::ArgAction::SetTrue)]
    pub show_prog: bool,

    #[arg(long, default_value_t = false, action = clap::ArgAction::SetTrue)]
    pub warn_as_err: bool,
}

impl CliArgs {
    pub fn set_param(&mut self, name: &str, val: &LitVal) -> Result<String, String> {
        let val_str = val.to_string();
        match (name, val) {
            ("solver", LitVal::String(s)) if s.as_str() == "z3" => self.solver = Solver::Z3,
            ("solver", LitVal::String(s)) if s.as_str() == "cvc5" => self.solver = Solver::CVC5,
            ("solver", LitVal::String(s)) if s.as_str() == "bitwuzla" => {
                self.solver = Solver::Bitwuzla
            }
            ("solver", LitVal::String(s)) if s.as_str() == "no-smt" => self.solver = Solver::NoSmt,
            ("int_rep", LitVal::String(s)) if s.as_str() == "bv8" => self.int_rep = IntRep::BV8,
            ("int_rep", LitVal::String(s)) if s.as_str() == "bv16" => self.int_rep = IntRep::BV16,
            ("int_rep", LitVal::String(s)) if s.as_str() == "bv32" => self.int_rep = IntRep::BV32,
            ("heuristic", LitVal::String(s)) if s.as_str() == "left-biased" => {
                self.heuristic = Heuristic::LeftBiased;
            }
            ("heuristic", LitVal::String(s)) if s.as_str() == "interleave" => {
                self.heuristic = Heuristic::Interleave;
            }
            ("heuristic", LitVal::String(s)) if s.as_str() == "small-first" => {
                self.heuristic = Heuristic::SmallFirst;
            }
            ("heuristic", LitVal::String(s)) if s.as_str() == "hybrid" => {
                self.heuristic = Heuristic::Hybrid;
            }
            ("heuristic", LitVal::String(s)) if s.as_str() == "random" => {
                self.heuristic = Heuristic::Random;
            }
            ("time_limit", &LitVal::Int(x)) if let Ok(n) = usize::try_from(x) => {
                self.time_limit = n;
            }
            ("time_limit_per", &LitVal::Int(x)) if let Ok(n) = usize::try_from(x) => {
                self.time_limit_per = n;
            }
            ("answer_limit", &LitVal::Int(x)) if let Ok(n) = usize::try_from(x) => {
                self.answer_limit = n;
            }
            ("depth_limit", &LitVal::Int(x)) if let Ok(n) = usize::try_from(x) => {
                self.depth_limit = n;
            }
            ("depth_range", &LitVal::Int(x)) if let Ok(n) = usize::try_from(x) => {
                self.depth_range = n;
            }
            ("depth_grow", &LitVal::Int(x)) if let Ok(n) = usize::try_from(x) => {
                self.depth_grow = n;
            }
            ("verbosity", &LitVal::Int(x)) if let Ok(n) = u8::try_from(x) => {
                self.verbosity = n;
            }
            ("dump_file", &LitVal::Bool(b)) => self.dump_file = b,
            ("debug_mode", &LitVal::Bool(b)) => self.debug_mode = b,
            ("show_output", &LitVal::Bool(b)) => self.show_output = b,
            ("show_stat", &LitVal::Bool(b)) => self.show_stat = b,
            ("show_prog", &LitVal::Bool(b)) => self.show_prog = b,
            ("warn_as_err", &LitVal::Bool(b)) => self.warn_as_err = b,
            (n, v) => {
                return Err(format!(
                    "unsupported parameter `{n}` or invalid value `{v}` for this parameter!"
                ));
            }
        }
        Ok(format!("parameter `{name}` has been set to `{val_str}`."))
    }
}

pub fn parse_cli_args() -> CliArgs {
    CliArgs::parse()
}

pub fn get_test_cli_args(prog_name: PathBuf) -> CliArgs {
    CliArgs {
        input: prog_name,
        solver: Solver::Z3,
        int_rep: IntRep::BV16,
        heuristic: Heuristic::Hybrid,
        time_limit: usize::MAX,
        time_limit_per: 1000,
        answer_limit: usize::MAX,
        depth_limit: 1000,
        depth_range: 5,
        depth_grow: 5,
        verbosity: 10,
        dump_file: false,
        debug_mode: false,
        show_output: true,
        show_stat: true,
        show_prog: false,
        warn_as_err: true,
    }
}

pub fn get_bench_cli_args(
    prog_name: PathBuf,
    heuristic: Heuristic,
    answer_limit: usize,
) -> CliArgs {
    CliArgs {
        input: prog_name,
        solver: Solver::Z3,
        int_rep: IntRep::BV16,
        heuristic,
        time_limit: usize::MAX,
        time_limit_per: 1000,
        answer_limit,
        depth_limit: 1000,
        depth_range: 5,
        depth_grow: 5,
        verbosity: 10,
        dump_file: false,
        debug_mode: false,
        show_output: false,
        show_stat: false,
        show_prog: false,
        warn_as_err: true,
    }
}
