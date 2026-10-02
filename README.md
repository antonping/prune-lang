# Prune Programming Language

## Language Overview

Prune is a declarative programming language for describing logic constraints and searching for values that satisfy them, primarily used as a property-based testing (PBT) data generator.
It allows users to describe complex input preconditions and sample valid test values, without writing custom generators or relying on inefficient random sampling and filtering.

The language is heavily inspired by logic programming languages such as Prolog and MiniKanren.
Unlike these languages, Prune allows you to define functions and treat them as relations between inputs and outputs.
This "functional" style improves readability and makes it easier for beginners to get started.

The search algorithm for queries is based on unification and backtracking, and the choice of sub-goal and branching order is randomized and guided by heuristics.
For arithmetic constraints, Prune calls an external SMT solver (Z3, cvc5 or Bitwuzla).
With powerful modern SMT solvers, queries with arithmetic constraints can be solved efficiently.

# Installation

## Install Prune compiler

The Prune compiler can be installed using `Cargo` (Rust's package manager).
Before running these commands, make sure you have the [Rust toolchain](https://rustup.rs/) installed.

```bash
# Install Prune compiler.
cargo install prune-lang

# Check if prune is correctly installed.
prune --version
```

You can also download a prebuilt binary from the GitHub release page, or build it yourself from source.

## (Optional) Install SMT solver

SMT solvers are used to handle arithmetic constraints over integers and floating-point numbers.
You can use the Prune compiler without installing an SMT solver, but with some limitations.
These SMT solvers are supported:

- [Z3 solver](https://github.com/Z3Prover/z3) (Recommended)
- [cvc5 solver](https://github.com/cvc5/cvc5)
- [Bitwuzla solver](https://github.com/bitwuzla/bitwuzla)

Installation guides can be found on their GitHub pages linked above.

# Quick Start

To get started, writing a constraint-based data generator takes three steps:

- Define algebraic datatypes.
- Define functions and predicates over these datatypes.
- Choose one predicate (or several) to run as a query.

Prune compiles your program into logic rules, then a logic interpreter searches for datatype values that satisfy the query.

## First example: Palindrome (without SMT solver)

In the example below, we define a list datatype and a `reverse` function over it.
The query will search for a list that stays the same after reversing (in other words, palindromes).
Since no arithmetic constraints are used, this example runs without an SMT solver.

```
datatype List[a] where
| Nil
| Cons(a, List[a])
end

function reverse(xs: List[Int]) -> List[Int]
begin
    reverse_help(xs, Nil)
end

function reverse_help(xs: List[Int], acc: List[Int]) -> List[Int]
begin
    match xs with
    | Nil => acc
    | Cons(x, ys) => reverse_help(ys, Cons(x, acc))
    end
end

function palindrome(xs: List[Int])
begin
    guard reverse(xs) = xs;
end

%param answer_limit 30;
%query palindrome;
```

To run this example, save the code as test1.pr and run the following command:

```bash
prune test1.pr
```

The output should look something like this (results vary between runs):

```
[SUCC]: cnt=1, size=3, range=(0,5), run_time=0ms, smt_time=0ms
xs: List(Int) = Nil
[SUCC]: cnt=2, size=5, range=(0,5), run_time=0ms, smt_time=0ms
xs: List(Int) = Cons(3927, Cons(3927, Nil))

......

[SUCC]: cnt=29, size=8, range=(5,10), run_time=0ms, smt_time=0ms
xs: List(Int) = Cons(-15129, Cons(17282, Cons(-26185, Cons(17282, Cons(-15129, Nil)))))
[SUCC]: cnt=30, size=6, range=(6,11), run_time=0ms, smt_time=0ms
xs: List(Int) = Cons(20083, Cons(16406, Cons(20083, Nil)))
[STOP]: Answer limit exceeded!

```

You can check that all printed answers satisfy the constraint — in other words, they are all palindromes.

## Second example: Sorted List (with SMT solver)

Before running the following example, make sure you have an external SMT solver installed (see `Installation` section).

In this example, we define a function `is_sorted` that checks whether a list is sorted in ascending order.
The query will search for sorted lists.

```
datatype List[a] where
| Nil
| Cons(a, List[a])
end

function is_sorted(xs: List[Int]) -> Bool
begin
    match xs with
    | Nil => true
    | Cons(_, Nil) => true
    | Cons(x, Cons(y, zs)) =>
        x < y && is_sorted(Cons(y, zs))
    end
end

function sorted_list(xs: List[Int])
begin
    guard is_sorted(xs) = true;
end

%param answer_limit 30;
%query sorted_list;
```

Save this code as test2.pr and run the corresponding command:

```bash
prune test2.pr --solver z3   # for Z3 solver
prune test2.pr --solver cvc5 # for cvc5 solver
prune test2.pr --solver bitwuzla # for Bitwuzla solver
```

The output looks like this:

```
[SUCC]: cnt=1, size=2, range=(0,5), run_time=1ms, smt_time=0ms
xs: List(Int) = Nil
[SUCC]: cnt=2, size=2, range=(0,5), run_time=0ms, smt_time=0ms
xs: List(Int) = Cons(-3439, Nil)

......

[SUCC]: cnt=29, size=6, range=(5,10), run_time=7ms, smt_time=29ms
xs: List(Int) = Cons(-5360, Cons(-5166, Cons(-344, Cons(16897, Cons(18918, Nil)))))
[SUCC]: cnt=30, size=6, range=(6,11), run_time=8ms, smt_time=25ms
xs: List(Int) = Cons(-5301, Cons(8530, Cons(11253, Cons(11427, Cons(29709, Nil)))))
[STOP]: Answer limit exceeded!
```

You can check that all these lists are sorted in ascending order.

# License

Licensed under the Apache License, Version 2.0.
See [LICENSE](LICENSE) or http://www.apache.org/licenses/LICENSE-2.0 for details.