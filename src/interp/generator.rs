use rand::seq::SliceRandom;

use super::branch::*;
use super::*;
use crate::cli::args::{self, CliArgs};
use crate::cli::pipeline::OutputWriter;
use crate::interp;

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
            size += 0.1;
            self.run_sized(
                query_decl.entry,
                (size.floor() as usize, size.floor() as usize + 5),
            );

            if self.ansr_cnt >= self.config.answer_limit {
                writeln!(self.output.answer, "[STOP]: Answer limit exceeded!").unwrap();
                return self.ansr_cnt;
            }
            let time = self.config.start_time.elapsed().as_secs_f32();
            if time > self.config.time_limit as f32 {
                writeln!(self.output.answer, "[STOP]: Time limit exceeded!").unwrap();
                return self.ansr_cnt;
            }
            let mem = memory_stats::memory_stats().unwrap().physical_mem as f32 / 1048576.0;
            if mem > self.config.mem_limit as f32 {
                writeln!(self.output.answer, "[STOP]: Memory limit exceeded!").unwrap();
                return self.ansr_cnt;
            }
        }
    }

    fn run_sized(&mut self, pred: Ident, size: (usize, usize)) {
        let brch = branch_init(self.prog, pred);
        let mut stack = vec![brch];

        while !stack.is_empty() {
            let brch = stack.pop().unwrap();
            if brch.depth + brch.calls.len() > size.1 {
                continue;
            }

            if self.is_solved(&brch) {
                if brch.depth < size.0 {
                    continue;
                }
                if self.solve_answer(&brch) {
                    return;
                }
            } else {
                let mut brchs = self.branch_split(&brch);
                brchs.shuffle(&mut self.rng);
                for (brch, _) in brchs {
                    stack.push(brch);
                }
            }
        }
    }

    fn is_solved(&self, brch: &Branch) -> bool {
        if !brch.calls.is_empty() {
            return false;
        }
        for ansr in &brch.ansrs {
            if let Err((_, _)) = check_free_var(self.prog, &ansr.val, &ansr.ty) {
                return false;
            }
        }
        true
    }

    fn solve_answer(&mut self, brch: &Branch) -> bool {
        let smt_start = std::time::Instant::now();

        if let Some(map) = self.solver.generate_sat(&mut self.rng, &brch.prims) {
            self.ansr_cnt += 1;
            let time = self.config.start_time.elapsed().as_secs_f32();
            let mem = memory_stats::memory_stats().unwrap().physical_mem as f32 / 1048576.0;
            let smt_time = smt_start.elapsed();
            writeln!(
                self.output.answer,
                "[ANSWER]({}): depth={}, time={:.2}s, mem={:.2}MB, SMT={:.2?}",
                self.ansr_cnt, brch.depth, time, mem, smt_time
            )
            .unwrap();

            let map = map
                .into_iter()
                .map(|(var, lit)| (var, Term::Lit(lit)))
                .collect();

            for Answer { par, ty, val } in &brch.ansrs {
                writeln!(
                    self.output.answer,
                    "{}: {} = {}",
                    par,
                    reinterp_type(ty),
                    reinterp_term(&val.substitute(&map))
                )
                .unwrap();
            }

            true
        } else {
            false
        }
    }

    fn branch_split(&mut self, brch: &Branch) -> Vec<(Branch, Vec<(usize, usize)>)> {
        if brch.calls.is_empty() {
            // split by free variable
            if let Some(brchs) = split_free_var(self.prog, brch) {
                brchs
                    .into_iter()
                    .enumerate()
                    .map(|(idx, brch)| (brch, vec![(0, idx)]))
                    .collect()
            } else {
                panic!("this branch is already solved!!")
            }
        } else {
            // split by predicate calls
            let call_idx = match self.config.heuristic {
                args::Heuristic::LeftBiased => brch.left_biased_strategy(),
                args::Heuristic::Interleave => brch.interleave_strategy(),
                args::Heuristic::SmallFirst => brch.small_first_strategy(),
                args::Heuristic::Hybrid => brch.hybrid_strategy(),
                args::Heuristic::Random => brch.random_strategy(&mut self.rng),
            };
            let mut res = Vec::new();
            for &rule_idx in brch.calls[call_idx].looks.iter() {
                if let Some((brch, path)) =
                    apply_rule_with_reduction(self.prog, brch, call_idx, rule_idx)
                {
                    if self.solver.check_sat(&brch.prims).is_some() {
                        res.push((brch, path));
                    }
                }
            }
            res
        }
    }
}
