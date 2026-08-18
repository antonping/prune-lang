use super::*;
use crate::logic::ast::QueryParam;
use clap::{Parser, ValueEnum};

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
pub enum Solver {
    Z3,
    CVC5,
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

    #[arg(long, default_value_t = usize::MAX, value_name = "INT")]
    pub answer_limit: usize,

    #[arg(long, default_value_t = usize::MAX, value_name = "INT")]
    pub time_limit: usize,

    #[arg(long, default_value_t = usize::MAX, value_name = "INT")]
    pub mem_limit: usize,

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
    pub fn set_param(&mut self, param: &QueryParam) {
        match param {
            QueryParam::AnswerLimit(x) => {
                self.answer_limit = *x;
            }
            QueryParam::TimeLimit(x) => {
                self.time_limit = *x;
            }
            QueryParam::MemLimit(x) => {
                self.mem_limit = *x;
            }
        }
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
        answer_limit: usize::MAX,
        time_limit: usize::MAX,
        mem_limit: usize::MAX,
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
        answer_limit,
        time_limit: usize::MAX,
        mem_limit: usize::MAX,
        verbosity: 10,
        dump_file: false,
        debug_mode: false,
        show_output: false,
        show_stat: false,
        show_prog: false,
        warn_as_err: true,
    }
}
