# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Implement size-based search algorithm as generator.
- Add integer representation parameter for SMT solver.
- Support randomized model generation in SMT solving.
- Add more CLI parameters for new generator architecture.
- Support Bitwuzla SMT solver backend.

### Changed

- Change int and float literals from 64-bit to 32-bit.
- Reimplement partial answer completion algorithm.
- Change SMT solving entirely to bit-vector theory.

### Removed

- Remove IDDFS search algorithm and related parameters.
- Remove deprecated look-ahead branching heuristic.
- Remove deprecated reduction flag parameter.

## [0.2.4] - 2026-07-17

### Added

- Add reduction flag to CLI parameters.
- Add builtin implementation for primitives.
- Add type information to answer variables.
- Implement branch splitting by free variables.
- Improve readability after solving builtin primitives.
- Add encoded primitives as an alternative solver backend.
- Add new benchmarks for test generation.
- Introduce hybrid branching heuristic.

### Changed

- Refactor benchmark tests.

## [0.2.3] - 2026-06-11

### Added

- Add type introduction rules in type checker.
- Add snapshot testing for diagnostic messages.
- Add more test cases for semantic analyzer.
- Refactor query runner with reduction pass.
- Introduce small-first branching heuristic.

### Changed

- Improve error message quality in type checker.
- Refactor CLI interface implementation.
- Refactor benchmark tests.
- Improve look-ahead branching heuristic.

### Removed

- Remove structural-recursive branching heuristic.

### Fixed

- Fix an incorrect span bug in parser.

## [0.2.2] - 2026-04-26

### Added

- Add searching parameters as CLI arguments.
- Add benchmark testing for measuring performance.

### Changed

- Reimplement example `binary_arith`.

### Fixed

- Fix a bug in the parser that causes unwrapping an error.

## [0.2.1] - 2026-03-26

### Added

- Add a new example `avl_tree_arith_gen`.
- Add a new example `ternary_arith`.

### Changed

- Reimplement example `avl_tree_gen`.
- Reimplement example `binary_arith`.
- Use randomized rule application order in IDDFS.
- Change the default values for CLI parameters.

## [0.2.0] - 2026-03-02

### Added

- Support generic type for datatypes and functions.
- Support file dump in output directory.
- Implement look-ahead branching heuristic.
- Support interactive debug mode.

### Changed

- Rewrite benchmark examples.
- Change CLI arguments for program outputs.
- Change default right-hand side for guard syntax.

### Removed

- Remove deprecated conflict-driven heuristic.

## [0.1.3] - 2026-02-04

### Added

- Implement structural-recursive branching heuristic.

### Changed

- Change to new logic interpreter for better performance.
- Replace incremental SMT solving with primitive constraint propagator.

## [0.1.2] - 2026-01-06

### Changed

- Modify test benchmarks.

### Fixed

- Fix a pattern match bug in type checker.
- Fix a bug in left-biased scheduling.

## [0.1.1] - 2025-11-22

### Added

- Create file `CHANGELOG.md`

### Changed

- Modify `avl_tree` examples for better performance.

### Fixed

- Fix a bug in parser error reporting.
- Fix a bug in SMT solver configuration.
- Fix a bug in example `avl_tree_bad`.
- Fix a vibe typo in `README.md`.

## [0.1.0] - 2025-11-15

### Added

- Update `README.md`.
- Support ad-hoc syntax for unit return value.
- Support both Z3 and CVC5 SMT solver backend.
- Support phoney SMT solver backend.
- Add special guard syntax for boolean values.
- Implement CLI tool.

### Changed

- Modify conflict-driven heuristic implementation.

### Removed

- Remove deprecated predicate syntax.
