mod common;
use common::{run_diag_test_fail, run_diag_test_succ};

#[test]
fn test_diag_cond_branch_succ() {
    run_diag_test_succ("diag_tests/type/cond_branch_succ");
}
#[test]
fn test_diag_cond_branch_fail() {
    run_diag_test_fail("diag_tests/type/cond_branch_fail");
}
#[test]
fn test_diag_cons_arg_succ() {
    run_diag_test_succ("diag_tests/type/cons_arg_succ");
}
#[test]
fn test_diag_cons_arg_fail1() {
    run_diag_test_fail("diag_tests/type/cons_arg_fail1");
}
#[test]
fn test_diag_cons_arg_fail2() {
    run_diag_test_fail("diag_tests/type/cons_arg_fail2");
}
#[test]
fn test_diag_func_arg_succ() {
    run_diag_test_succ("diag_tests/type/func_arg_succ");
}
#[test]
fn test_diag_func_arg_fail1() {
    run_diag_test_fail("diag_tests/type/func_arg_fail1");
}
#[test]
fn test_diag_func_arg_fail2() {
    run_diag_test_fail("diag_tests/type/func_arg_fail2");
}
#[test]
fn test_diag_guard_type_succ() {
    run_diag_test_succ("diag_tests/type/guard_type_succ");
}
#[test]
fn test_diag_guard_type_fail() {
    run_diag_test_fail("diag_tests/type/guard_type_fail");
}
#[test]
fn test_diag_if_branch_succ() {
    run_diag_test_succ("diag_tests/type/if_branch_succ");
}
#[test]
fn test_diag_if_branch_fail() {
    run_diag_test_fail("diag_tests/type/if_branch_fail");
}
#[test]
fn test_diag_if_cond_succ() {
    run_diag_test_succ("diag_tests/type/if_cond_succ");
}
#[test]
fn test_diag_if_cond_fail() {
    run_diag_test_fail("diag_tests/type/if_cond_fail");
}
#[test]
fn test_diag_let_infer_succ() {
    run_diag_test_succ("diag_tests/type/let_infer_succ");
}
#[test]
fn test_diag_let_infer_fail() {
    run_diag_test_fail("diag_tests/type/let_infer_fail");
}
#[test]
fn test_diag_let_patn_succ() {
    run_diag_test_succ("diag_tests/type/let_patn_succ");
}
#[test]
fn test_diag_let_patn_fail() {
    run_diag_test_fail("diag_tests/type/let_patn_fail");
}
#[test]
fn test_diag_match_branch_succ() {
    run_diag_test_succ("diag_tests/type/match_branch_succ");
}
#[test]
fn test_diag_match_branch_fail() {
    run_diag_test_fail("diag_tests/type/match_branch_fail");
}
#[test]
fn test_diag_poly_cons_succ() {
    run_diag_test_succ("diag_tests/type/poly_cons_succ");
}
#[test]
fn test_diag_poly_cons_fail() {
    run_diag_test_fail("diag_tests/type/poly_cons_fail");
}
#[test]
fn test_diag_poly_func_succ() {
    run_diag_test_succ("diag_tests/type/poly_func_succ");
}
#[test]
fn test_diag_poly_func_fail() {
    run_diag_test_fail("diag_tests/type/poly_func_fail");
}
