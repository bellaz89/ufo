# Repository Guidelines

## Project Structure & Module Organization

This branch is being converted to a pure Rust crate. `src/` contains the Rust library, including the MAD parser, typed lattice model, bytecode compiler, and OpenCL host wrapper. `src/kernels/` contains bundled OpenCL interpreter kernels. `optics/` stores MAD lattice fixtures used by tests. `tests/` contains Rust integration tests; the older Python tests remain temporarily as migration references.

## Build, Test, and Development Commands

- `cargo test`: build the Rust crate with default OpenCL support and run unit/integration tests.
- `cargo test --no-default-features`: run parser and bytecode tests without linking OpenCL.
- `cargo run -- list-devices`: list OpenCL devices through the Rust OpenCL wrapper.
- `cargo run -- load optics/fodo.mad`: parse a MAD lattice and print a compact summary.
- `cargo run -- compile optics/fodo.mad --flag linear --flag achromatic`: compile a line to interpreter bytecode metadata.
- `UFO_RUN_OPENCL_TESTS=1 cargo test builds_interpreter_when_opencl_tests_are_enabled`: build `interpreter.cl` through the first available OpenCL device.

## Coding Style & Naming Conventions

Use Rust 2024 edition, `cargo fmt`, and typed domain structures instead of dynamic maps where possible. Keep public optics names close to the established UFO vocabulary (`Line`, `Lattice`, element labels, flags such as `FIVED`). Keep OpenCL kernels in `src/kernels/` synchronized with the Rust bytecode layout, and avoid adding new JIT source-generation paths.

## Testing Guidelines

Rust tests live in `tests/` and module-local `#[cfg(test)]` blocks. Use small MAD examples from `optics/` or add focused fixtures there. Keep bytecode tests exact at the byte/word level. OpenCL tests should be opt-in when they require a configured device or writable POCL cache directories.

## Commit & Pull Request Guidelines

Recent history uses short imperative subjects, often with a `feat:` prefix, for example `feat: added diagnostics`; avoid `WIP` for review-ready work. Keep commits focused and describe behavior, not just files changed.

Pull requests should include a concise description, commands run, OpenCL device/backend details when relevant, and any numerical or visual output differences. Link related issues and include screenshots only for documentation or plot/image changes.

## Security & Configuration Tips

Do not commit local virtual environments, `target/`, generated caches, or machine-specific OpenCL settings. For POCL, prefer writable temp/cache directories under `/tmp` during local validation.
