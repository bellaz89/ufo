# Repository Guidelines

## Project Structure & Module Organization

This branch is a pure Rust crate. `src/` contains the Rust library, including the MAD parser, typed lattice model, bytecode compiler, CubeCL interpreter, and CubeCL host wrapper. `optics/` stores MAD lattice fixtures used by tests. `tests/` contains Rust integration tests.

## Build, Test, and Development Commands

- `cargo test`: build the Rust crate with default CubeCL CPU/Vulkan support and run unit/integration tests.
- `cargo test --no-default-features`: run parser and bytecode tests without CubeCL.
- `cargo run -- list-devices`: list CubeCL devices across enabled CPU, Vulkan, CUDA, HIP, and Metal backends.
- `cargo run -- load optics/fodo.mad`: parse a MAD lattice and print a compact summary.
- `cargo run -- compile optics/fodo.mad --flag linear --flag achromatic`: compile a line to interpreter bytecode metadata.
- `cargo test --no-default-features --features cubecl-cpu --test cubecl`: run CubeCL interpreter tests on the CPU backend.

## Coding Style & Naming Conventions

Use Rust 2024 edition, `cargo fmt`, and typed domain structures instead of dynamic maps where possible. Keep public optics names close to the established UFO vocabulary (`Line`, `Lattice`, element labels, flags such as `FIVED`). Keep the CubeCL interpreter synchronized with the Rust bytecode layout, and avoid adding new JIT source-generation paths.

## Testing Guidelines

Rust tests live in `tests/` and module-local `#[cfg(test)]` blocks. Use small MAD examples from `optics/` or add focused fixtures there. Keep bytecode tests exact at the byte/word level. GPU-specific CubeCL tests should be opt-in when they require a configured device.

## Commit & Pull Request Guidelines

Recent history uses short imperative subjects, often with a `feat:` prefix, for example `feat: added diagnostics`; avoid `WIP` for review-ready work. Keep commits focused and describe behavior, not just files changed.

Pull requests should include a concise description, commands run, CubeCL device/backend details when relevant, and any numerical or visual output differences. Link related issues and include screenshots only for documentation or plot/image changes.

## Security & Configuration Tips

Do not commit `target/`, generated caches, or machine-specific runtime settings. UFO places runtime caches under `~/.ufo/cache` by default; set `UFO_CACHE_DIR` to use another location during validation.
