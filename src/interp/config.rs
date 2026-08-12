use super::*;
use crate::cli::args::{self, CliArgs};

#[derive(Debug, Clone)]
pub struct ExecConfig {
    pub answer_limit: usize,
    pub time_limit: usize,
    pub mem_limit: usize,
    pub solver: args::Solver,
    pub heuristic: args::Heuristic,
    pub debug_mode: bool,
    pub start_time: std::time::Instant,
}

impl ExecConfig {
    pub fn new(args: &CliArgs) -> ExecConfig {
        ExecConfig {
            answer_limit: args.answer_limit,
            time_limit: args.time_limit,
            mem_limit: args.mem_limit,
            solver: args.solver,
            heuristic: args.heuristic,
            debug_mode: args.debug_mode,
            start_time: std::time::Instant::now(),
        }
    }

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
