use super::*;
use crate::cli::args::{self, CliArgs};

#[derive(Debug)]
pub struct RunnerConfig {
    pub answer_limit: usize,
    pub solver: args::Solver,
    pub heuristic: args::Heuristic,
    pub strategy: args::Strategy,
    pub debug_mode: bool,
}

impl RunnerConfig {
    pub fn new(args: &CliArgs) -> RunnerConfig {
        RunnerConfig {
            answer_limit: args.answer_limit,
            solver: args.solver,
            heuristic: args.heuristic,
            strategy: args.strategy,
            debug_mode: args.debug_mode,
        }
    }

    pub fn reset_default(&mut self) {
        self.answer_limit = usize::MAX;
    }

    pub fn set_param(&mut self, param: &QueryParam) {
        match param {
            QueryParam::DepthStep(_x) => {
                // deprecated
            }
            QueryParam::DepthLimit(_x) => {
                // deprecated
            }
            QueryParam::AnswerLimit(x) => {
                self.answer_limit = *x;
            }
            QueryParam::AnswerPause(_x) => {
                // deprecated
            }
        }
    }
}
