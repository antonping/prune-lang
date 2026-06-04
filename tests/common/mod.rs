#![allow(dead_code)]

use prune_lang::cli;
use std::path::PathBuf;

const EXAMPLES_DIR: &str = "./examples";

pub fn run_diag_test_succ(path: &str) {
    let name = path.rsplit_once('/').unwrap().1;
    let mut msg_out = Vec::new();
    cli::pipeline::run_test_diag_pipeline(
        PathBuf::from(format!("{EXAMPLES_DIR}/{path}.pr")),
        &mut msg_out,
    )
    .expect("this test should compile successfully!");
    insta::assert_snapshot!(name, String::from_utf8(msg_out).unwrap());
}

pub fn run_diag_test_fail(path: &str) {
    let name = path.rsplit_once('/').unwrap().1;
    let mut msg_out = Vec::new();
    cli::pipeline::run_test_diag_pipeline(
        PathBuf::from(format!("{EXAMPLES_DIR}/{path}.pr")),
        &mut msg_out,
    )
    .expect_err("this test should fail to compile!");
    insta::assert_snapshot!(name, String::from_utf8(msg_out).unwrap());
}

pub fn run_example(path: &str) {
    cli::pipeline::run_test_pipeline(PathBuf::from(format!("{EXAMPLES_DIR}/{path}"))).unwrap();
}

pub fn run_example_unsat(path: &str) {
    let res =
        cli::pipeline::run_test_pipeline(PathBuf::from(format!("{EXAMPLES_DIR}/{path}"))).unwrap();
    assert!(res.iter().all(|p| *p == 0));
}

pub fn run_example_sat(path: &str) {
    let res =
        cli::pipeline::run_test_pipeline(PathBuf::from(format!("{EXAMPLES_DIR}/{path}"))).unwrap();
    assert!(res.iter().any(|p| *p > 0));
}
