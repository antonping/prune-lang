use rand::seq::SliceRandom;

use super::branch::*;
use super::*;
use crate::cli::args::{self, CliArgs};
use crate::cli::pipeline::OutputWriter;
use crate::interp;

enum GenResult {
    Success {
        brch: Branch,
        br_time: usize,
        smt_time: usize,
    },
    Exhausted {
        time: usize,
    },
    Timeout,
}

pub struct Generator<'prog, 'io> {
    prog: &'prog Program,
    output: &'io mut OutputWriter,
    config: interp::config::ExecConfig,
    ansr_cnt: usize,
    rng: rngs::ThreadRng,
    solver: Box<dyn solver::common::PrimSolver>,
}

impl<'prog, 'io> Generator<'prog, 'io> {
    pub fn new(
        prog: &'prog Program,
        output: &'io mut OutputWriter,
        args: &CliArgs,
    ) -> Generator<'prog, 'io> {
        let solver = interp::solver::common::new_solver(args);
        let config = interp::config::ExecConfig::new(args);
        Generator {
            prog,
            output,
            config,
            ansr_cnt: 0,
            rng: rand::rng(),
            solver,
        }
    }

    pub fn run_loop(&mut self, query_decl: &QueryDecl) -> usize {
        for param in &query_decl.params {
            self.config.set_param(param);
        }

        let mut size: f32 = 1.0;

        loop {
            size += 0.2;

            let low_size = size.floor() as usize;
            let high_size = size.floor() as usize + 5;
            let res = self.run_sized(query_decl.entry, low_size, high_size);

            match res {
                GenResult::Success {
                    brch,
                    br_time,
                    smt_time,
                } => {
                    writeln!(
                        self.output.answer,
                        "[ANSWER]({}): depth={}, range=({},{}), br_time={:.2}ms, smt_time={:.2}ms",
                        self.ansr_cnt, brch.depth, low_size, high_size, br_time, smt_time
                    )
                    .unwrap();
                    for Answer { par, ty, val } in &brch.ansrs {
                        writeln!(
                            self.output.answer,
                            "{}: {} = {}",
                            par,
                            reinterp_type(ty),
                            reinterp_term(val)
                        )
                        .unwrap();
                    }
                }
                GenResult::Exhausted { time } => {
                    writeln!(
                        self.output.answer,
                        "[FAIL]: Search exhausted at range ({}, {}) in {}ms!",
                        low_size, high_size, time,
                    )
                    .unwrap();
                }
                GenResult::Timeout => {
                    writeln!(
                        self.output.answer,
                        "[FAIL]: Search timeout at range ({}, {})!",
                        low_size, high_size,
                    )
                    .unwrap();
                }
            }

            if self.ansr_cnt >= self.config.answer_limit {
                writeln!(self.output.answer, "[STOP]: Answer limit exceeded!").unwrap();
                break;
            }
            let time = self.config.start_time.elapsed().as_secs_f32();
            if time > self.config.time_limit as f32 {
                writeln!(self.output.answer, "[STOP]: Time limit exceeded!").unwrap();
                break;
            }
        }

        self.ansr_cnt
    }

    fn run_sized(&mut self, pred: Ident, size_low: usize, size_high: usize) -> GenResult {
        let brch = branch_init(self.prog, pred);
        let mut stack = vec![brch];

        let time_start = std::time::Instant::now();
        while !stack.is_empty() {
            let time = time_start.elapsed().as_millis() as usize;
            if time > 1000 {
                return GenResult::Timeout;
            }

            let mut brch = stack.pop().unwrap();
            if brch.depth + brch.calls.len() > size_high {
                continue;
            }

            if brch.calls.is_empty() {
                if brch.depth < size_low {
                    continue;
                }
                assert!(self.solver.check_sat(&brch.prims));
                let smt_time = self.solve_smt_constraints(&mut brch);
                self.ansr_cnt += 1;
                interp::completer::answer_complete(self.prog, &mut self.rng, &mut brch.ansrs);
                return GenResult::Success {
                    brch,
                    br_time: time,
                    smt_time,
                };
            } else {
                let mut brchs = self.branch_split(&brch);
                brchs.shuffle(&mut self.rng);
                for brch in brchs {
                    stack.push(brch);
                }
            }
        }

        let time = time_start.elapsed().as_millis() as usize;
        return GenResult::Exhausted { time };
    }

    fn solve_smt_constraints(&mut self, brch: &mut Branch) -> usize {
        let smt_start = std::time::Instant::now();
        let map = self.solver.generate_model(&mut self.rng, &brch.prims);
        let smt_time = smt_start.elapsed().as_millis() as usize;
        let map = map
            .into_iter()
            .map(|(var, lit)| (var, Term::Lit(lit)))
            .collect();
        for ansr in brch.ansrs.iter_mut() {
            ansr.val = ansr.val.substitute(&map);
        }
        smt_time
    }

    fn branch_split(&mut self, brch: &Branch) -> Vec<Branch> {
        let call_idx = match self.config.heuristic {
            args::Heuristic::LeftBiased => brch.left_biased_strategy(),
            args::Heuristic::Interleave => brch.interleave_strategy(),
            args::Heuristic::SmallFirst => brch.small_first_strategy(),
            args::Heuristic::Hybrid => brch.hybrid_strategy(),
            args::Heuristic::Random => brch.random_strategy(&mut self.rng),
        };
        let mut res = Vec::new();
        for &rule_idx in brch.calls[call_idx].looks.iter() {
            if let Some(brch) = apply_rule(self.prog, brch, call_idx, rule_idx) {
                if self.solver.check_sat(&brch.prims) {
                    res.push(brch);
                }
            }
        }
        res
    }
}
