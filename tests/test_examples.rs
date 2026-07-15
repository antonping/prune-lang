mod common;
use common::{run_example, run_example_sat, run_example_unsat};

#[test]
fn test_unary_arith() {
    run_example("arith/unary_arith.pr");
}
#[test]
fn test_binary_arith() {
    run_example("arith/binary_arith.pr");
}
#[test]
fn test_ternary_arith() {
    run_example("arith/ternary_arith.pr");
}
#[test]
fn test_binary_vec() {
    run_example("arith/binary_vec.pr");
}
#[test]
fn test_reverse_forward() {
    run_example("basic/reverse_forward.pr");
}
#[test]
fn test_reverse_backward() {
    run_example("basic/reverse_backward.pr");
}
#[test]
fn test_concat_forward() {
    run_example("basic/concat_forward.pr");
}
#[test]
fn test_concat_backward() {
    run_example("basic/concat_backward.pr");
}
#[test]
fn test_smt_sat() {
    run_example("features/smt_sat.pr");
}
#[test]
fn test_smt_unsat() {
    run_example("features/smt_unsat.pr");
}
#[test]
fn test_tree_insert_good() {
    run_example_unsat("sym_exec/tree_insert_good.pr");
}
#[test]
fn test_tree_insert_bad() {
    run_example_sat("sym_exec/tree_insert_bad.pr");
}
#[test]
fn test_avl_tree_good() {
    run_example_unsat("sym_exec/avl_tree_good.pr");
}
#[test]
#[ignore = "this test cost too much time!"]
fn test_avl_tree_bad() {
    run_example_sat("sym_exec/avl_tree_bad.pr");
}
#[test]
fn test_avl_tree_arith_gen() {
    run_example("test_gen/avl_tree_arith_gen.pr");
}
#[test]
fn test_avl_tree_gen() {
    run_example("test_gen/avl_tree_gen.pr");
}
#[test]
fn test_balanced_tree_arith_gen() {
    run_example("test_gen/balanced_tree_arith_gen.pr");
}
#[test]
fn test_balanced_tree_gen() {
    run_example("test_gen/balanced_tree_gen.pr");
}
#[test]
fn test_sorted_list_gen() {
    run_example("test_gen/sorted_list_gen.pr");
}
#[test]
fn test_lambda_free_gen() {
    run_example("test_gen/lambda_free_gen.pr");
}
#[test]
fn test_stlc_term_gen() {
    run_example("test_gen/stlc_term_gen.pr");
}
#[test]
fn test_mini_lang_gen() {
    run_example("test_gen/mini_lang_gen.pr");
}
