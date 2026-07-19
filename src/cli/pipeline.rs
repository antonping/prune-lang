use super::args::{self, CliArgs};
use super::diagnostic::{DiagLevel, Diagnostic};
use super::*;
use crate::cli::replay::ReplayWriter;
use crate::logic::ast::Program;
use crate::{interp, logic, syntax, tych};

pub struct OutputWriter {
    pub answer: Box<dyn Write>,
    pub stat: Box<dyn Write>,
    pub prog: Box<dyn Write>,
}

impl OutputWriter {
    pub fn empty() -> OutputWriter {
        OutputWriter {
            answer: Box::new(io::empty()),
            stat: Box::new(io::empty()),
            prog: Box::new(io::empty()),
        }
    }
}

pub struct Pipeline<'a> {
    pub args: &'a CliArgs,
    pub src_path: PathBuf,
    pub src_code: String,
    pub msg_out: Box<dyn Write + 'a>,
}

impl<'a> Pipeline<'a> {
    pub fn new(args: &'a CliArgs) -> Pipeline<'a> {
        Pipeline {
            args,
            src_path: PathBuf::new(),
            src_code: String::new(),
            msg_out: Box::new(io::stderr()),
        }
    }

    fn emit_diags<D: Into<Diagnostic>>(&mut self, diags: Vec<D>) -> bool {
        let mut flag = false;
        for diag in diags.into_iter() {
            let diag = diag.into();
            if diag.level == DiagLevel::Error
                || (self.args.warn_as_err && diag.level == DiagLevel::Warn)
            {
                flag = true;
            }
            writeln!(
                self.msg_out,
                "{}",
                diag.report(self.src_code.as_str(), self.args.verbosity)
            )
            .unwrap();
        }
        flag
    }

    pub fn run_compiler_pipline(&mut self) -> Result<Program, io::Error> {
        self.read_source_code()?;

        let mut prog = self.parse_program()?;

        self.rename_pass(&mut prog)?;

        self.check_pass(&mut prog)?;

        let mut prog = self.compile_pass(&prog);

        if self.args.solver == args::Solver::Encode {
            prog.extend_builtin();
            prog.replace_builtin();
        }
        Ok(prog)
    }

    pub fn read_source_code(&mut self) -> Result<(), io::Error> {
        let src_path = PathBuf::from(&self.args.input);
        if let Ok(src_code) = std::fs::read_to_string(&src_path) {
            self.src_path = src_path;
            self.src_code = src_code;
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("file \"{src_path:?}\" doesn't exist!"),
            ))
        }
    }

    pub fn parse_program(&mut self) -> Result<syntax::ast::Program, io::Error> {
        let (prog, errs) = syntax::parser::parse_program(self.src_code.as_str());
        if self.emit_diags(errs) {
            return Err(io::Error::other("failed to parse program!"));
        }
        Ok(prog)
    }

    pub fn rename_pass(&mut self, prog: &mut syntax::ast::Program) -> Result<(), io::Error> {
        let errs = tych::rename::rename_pass(prog);
        if self.emit_diags(errs) {
            return Err(io::Error::other("failed in binding analysis pass!"));
        }
        Ok(())
    }

    pub fn check_pass(&mut self, prog: &mut syntax::ast::Program) -> Result<(), io::Error> {
        let errs = tych::check::check_pass(prog);
        if self.emit_diags(errs) {
            return Err(io::Error::other("failed in type checking pass!"));
        }
        Ok(())
    }

    pub fn compile_pass(&mut self, prog: &syntax::ast::Program) -> logic::ast::Program {
        let mut prog = logic::compile::compile_pass(prog);

        logic::elaborate::elaborate_pass(&mut prog);

        prog
    }

    pub fn run_backend(&self, prog: &logic::ast::Program) -> Result<Vec<usize>, io::Error> {
        let mut output = create_output_writer(self.args, &self.src_path)?;
        writeln!(output.prog, "{prog}").unwrap();

        let mut res_vec = Vec::new();
        let mut runner = interp::executor::Executor::new(prog, &mut output, self.args);
        for query_decl in &prog.querys {
            for param in &query_decl.params {
                runner.config_set_param(param);
            }
            let res = runner.run_step_loop(query_decl.entry);
            res_vec.push(res);
        }
        Ok(res_vec)
    }
}

fn create_dump_dir(src_path: &PathBuf) -> Result<PathBuf, io::Error> {
    use std::fs;

    if src_path.extension().and_then(|ext| ext.to_str()) != Some("pr") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("source file extension is not \".pr\"!: {src_path:?}"),
        ));
    }

    if !src_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("file \"{src_path:?}\" doesn't exist!"),
        ));
    }

    if !src_path.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("path \"{src_path:?}\" exists, but it is not a file!"),
        ));
    }

    let file_stem = src_path.file_stem().unwrap().to_os_string();

    let mut dir_path = src_path.clone();
    dir_path.pop();
    dir_path.push(file_stem);

    if !dir_path.exists() {
        fs::create_dir(&dir_path)?;
    } else if !dir_path.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("path \"{dir_path:?}\" exist, but it is not a directory!"),
        ));
    }

    Ok(dir_path)
}

fn create_output_writer(args: &CliArgs, src_path: &PathBuf) -> Result<OutputWriter, io::Error> {
    let mut output = OutputWriter::empty();
    if args.dump_file {
        let dir_path = create_dump_dir(src_path)?;

        let answer = File::create(dir_path.join("answer.txt"))?;
        if args.show_output {
            output.answer = Box::new(ReplayWriter::replay_stdout(answer));
        } else {
            output.answer = Box::new(answer);
        }

        let stat = File::create(dir_path.join("stat.txt"))?;
        if args.show_stat {
            output.stat = Box::new(ReplayWriter::replay_stdout(stat));
        } else {
            output.stat = Box::new(stat);
        }

        let prog = File::create(dir_path.join("prog.txt"))?;
        if args.show_prog {
            output.prog = Box::new(ReplayWriter::replay_stdout(prog));
        } else {
            output.prog = Box::new(prog);
        }
    } else {
        if args.show_output {
            output.answer = Box::new(io::stdout());
        }

        if args.show_stat {
            output.stat = Box::new(io::stdout());
        }

        if args.show_prog {
            output.prog = Box::new(io::stdout());
        }
    }
    Ok(output)
}

pub fn run_cli_pipeline() -> Result<Vec<usize>, io::Error> {
    let args = args::parse_cli_args();
    let mut pipe = Pipeline::new(&args);
    let prog = pipe.run_compiler_pipline()?;
    let res = pipe.run_backend(&prog)?;
    Ok(res)
}

pub fn run_test_pipeline(prog_name: PathBuf) -> Result<Vec<usize>, io::Error> {
    let args = args::get_test_cli_args(prog_name);
    let mut pipe = Pipeline::new(&args);
    let prog = pipe.run_compiler_pipline()?;
    let res = pipe.run_backend(&prog)?;
    Ok(res)
}

pub fn run_test_diag_pipeline(prog_name: PathBuf, msg_out: &mut Vec<u8>) -> Result<(), io::Error> {
    let args = args::get_test_cli_args(prog_name);
    let mut pipe = Pipeline::new(&args);
    pipe.msg_out = Box::new(msg_out);
    let _prog = pipe.run_compiler_pipline()?;
    Ok(())
}

pub fn run_bench_pipeline(
    prog_name: PathBuf,
    heuristic: args::Heuristic,
    reduction: bool,
    depth_limit: usize,
) -> Result<Vec<usize>, io::Error> {
    let args = args::get_bench_cli_args(prog_name, heuristic, reduction, depth_limit);
    let mut pipe = Pipeline::new(&args);
    let prog = pipe.run_compiler_pipline()?;
    let res = pipe.run_backend(&prog)?;
    Ok(res)
}
