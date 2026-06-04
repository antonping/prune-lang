mod common;
use common::{run_diag_test_fail, run_diag_test_succ};

#[test]
fn test_diag_unused_var_succ() {
    run_diag_test_succ("diag_tests/binding/unused_var_succ");
}
#[test]
fn test_diag_unused_var_fail() {
    run_diag_test_fail("diag_tests/binding/unused_var_fail");
}
#[test]
fn test_diag_undefined_var_succ() {
    run_diag_test_succ("diag_tests/binding/undefined_var_succ");
}
#[test]
fn test_diag_undefined_var_fail() {
    run_diag_test_fail("diag_tests/binding/undefined_var_fail");
}
#[test]
fn test_diag_dup_func_succ() {
    run_diag_test_succ("diag_tests/binding/dup_func_succ");
}
#[test]
fn test_diag_dup_func_fail() {
    run_diag_test_fail("diag_tests/binding/dup_func_fail");
}
#[test]
fn test_diag_dup_data_succ() {
    run_diag_test_succ("diag_tests/binding/dup_data_succ");
}
#[test]
fn test_diag_dup_data_fail() {
    run_diag_test_fail("diag_tests/binding/dup_data_fail");
}
#[test]
fn test_diag_dup_cons_succ() {
    run_diag_test_succ("diag_tests/binding/dup_cons_succ");
}
#[test]
fn test_diag_dup_cons_fail1() {
    run_diag_test_fail("diag_tests/binding/dup_cons_fail1");
}
#[test]
fn test_diag_dup_cons_fail2() {
    run_diag_test_fail("diag_tests/binding/dup_cons_fail2");
}
#[test]
fn test_diag_shadowing_succ() {
    run_diag_test_succ("diag_tests/binding/shadowing_succ");
}
#[test]
fn test_diag_shadowing_fail() {
    run_diag_test_fail("diag_tests/binding/shadowing_fail");
}
#[test]
fn test_diag_wildcard_succ() {
    run_diag_test_succ("diag_tests/binding/wildcard_succ");
}
#[test]
fn test_diag_wildcard_fail() {
    run_diag_test_fail("diag_tests/binding/wildcard_fail");
}
