# Coding Standards & Guidelines

This document outlines the strict coding standards and guidelines for the **RustPBX** project. Every contributor and agent must follow these rules.

---

## 1. Language & Documentation

- **100% English**: All documentation, code comments, variable names, function names, error messages, and log messages **MUST** be written in English.
- **No Turkish/Non-English prose** in any source file, docstring, or commit message.

---

## 2. Strict Modular Structure & File Size Limits

To prevent maintenance nightmares and keep files readable and easy to edit:

- **Maximum File Length**: No single source file should exceed **300-400 lines of code**. If a file grows beyond 400 lines, it **MUST** be refactored and split into submodules.
- **Granular Modules**: Split crates into deep submodule trees. For example:
  - `sipcore/src/parser/headers/from.rs`
  - `sipcore/src/parser/headers/to.rs`
  - `sipcore/src/parser/headers/via.rs`
  Instead of putting all header parsers in a single 2000-line `header.rs` file.
- **Single Responsibility Principle**: Each module should define at most 1-2 core types or a tight set of related helper functions.

---

## 3. Mandatory CI/Build Quality Checks

Before any code is committed or merged, the following checks **MUST** pass cleanly with zero warnings:

```bash
# 1. Format Check (Strict rustfmt formatting)
cargo fmt --all -- --check

# 2. Linter Checks (Strict Clippy warnings treated as errors)
cargo clippy --workspace --all-targets --all-features -- -D warnings

# 3. Test Suite Pass
cargo test --workspace
```

### Pre-commit Hooks & Developer Workflow

- All developers must run `cargo fmt` and `cargo clippy` during local development.
- In CI pipelines, warnings are strictly forbidden (`-D warnings`).

---

## 4. Code Hygiene & Rust Idioms

- **Zero Unused Code**: No `#[allow(dead_code)]` or unused imports in production code.
- **Explicit Error Handling**: Avoid `.unwrap()` and `.expect()` in non-test code. Use custom `enum Error` types with `thiserror`.
- **Memory Safety**: No `unsafe` blocks without explicit justification and code review.
- **Zero-Copy & Performance**: Use `bytes::Bytes` and offset slices instead of allocating heap strings during parsing.
