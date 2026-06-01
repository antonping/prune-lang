use insta::*;
use std::path::PathBuf;

use prune_lang::cli;

#[test]
fn test_diag_polymorphism() {
    let mut msg_out = Vec::new();
    cli::pipeline::run_test_diag_pipeline(
        PathBuf::from("./examples/features/polymorphism.pr"),
        &mut msg_out,
    )
    .expect("this should pass without errors!");
    assert_snapshot!(String::from_utf8(msg_out).unwrap());
}

#[test]
fn test_diag_polymorphism_fail() {
    let mut msg_out = Vec::new();
    cli::pipeline::run_test_diag_pipeline(
        PathBuf::from("./examples/features/polymorphism_fail.pr"),
        &mut msg_out,
    )
    .expect_err("this should fail at type checking!");
    assert_snapshot!(String::from_utf8(msg_out).unwrap());
}
