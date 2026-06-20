```
              ██████  ██    ██ ███████  ██████
             ██       ██    ██ ██      ██    ██
             ██   ███ ██    ██ █████   ██    ██
             ██    ██ ██    ██ ██      ██    ██
              ██████   ██████  ██       ██████


                         .-.
                        (o o)
                        | O \
                         \   \
                          `~~~'


              GPU-based Unreliable, but Fast Optics


          Developed by Michele Carla' and Andrea Bellandi,
    based on the initial work of Manu Canals and Michele Carla'
           with the support of ALBA (www.cells.es)

                https://github.com/bellaz89/gufo
```

**GUFO** is a fast accelerator optics toolkit built around GPU execution while
remaining usable on CPUs. The name stands for **GPU-based Unreliable, but Fast
Optics**.

GUFO is not intended to be a general-purpose optics program. It prioritizes
high-throughput tracking and optics workflows over maximum runtime flexibility.

## Features

- Typed Rust model for lattices, lines, and accelerator elements.
- MAD-style parser for the lattice fixtures in `optics/`.
- Interpreter bytecode shared by CPU and GPU execution paths.
- CubeCL interpreter kernel with shared-memory instruction caching.
- Default CubeCL CPU and Vulkan support, with optional CUDA, HIP, and Metal
  features.
- CLI modes for loading, compiling, tracking, optics, chromaticity, radiation
  integrals, closed orbit, RDT, and stable aperture.
- Optional Python extension module with an API compatible with the original
  Python tracking workflow for coordinate parameters.

## Requirements

- Rust 1.96 or later.
- Cargo.
- A CubeCL-supported runtime for simulations.
- Python 3.9 or later, NumPy, and maturin for the optional Python interface.

The default Rust build enables CubeCL CPU and Vulkan. CUDA, HIP, and Metal are
available through optional Cargo features.

## Install

Clone and test the Rust crate:

```bash
git clone https://github.com/bellaz89/gufo
cd gufo
cargo test
```

List available execution targets:

```bash
cargo run -- list-devices
```

Typical output:

```text
vulkan:integrated:0    AMD Radeon Graphics (RADV RENOIR)
vulkan:cpu             llvmpipe (LLVM 22.1.6, 256 bits)
cpu:0                  CubeCL CPU
```

The default simulation backend is `auto`: GUFO uses the first Vulkan device
when available, otherwise it falls back to CubeCL CPU.

## Quick Start

```bash
cargo run -- load optics/fodo.mad
cargo run -- compile optics/fodo.mad --flag linear --flag achromatic
cargo run -- compile optics/fodo.mad --instructions
cargo run -- track optics/fodo.mad --turns 10 --where -1
cargo run -- optics optics/fodo.mad
cargo run -- chromaticity optics/fodo.mad
```

Select a backend or device explicitly:

```bash
cargo run -- track optics/fodo.mad --backend cpu
cargo run -- track optics/fodo.mad --device vulkan:integrated:0
cargo run -- track optics/fodo.mad --backend cuda --device 0
```

The `list_devices` spelling is accepted as an alias for `list-devices`.

## Program Modes

- `load <path>`: parse a MAD lattice and print a compact summary. Alias:
  `lattice`.
- `compile <path>`: compile a lattice line to interpreter bytecode metadata.
  Alias: `bytecode`.
- `dump <input> <output>`: write a lattice in `mad`, `elegant`, `at`, or `opa`
  format with `--style`.
- `list-devices`: list enabled CubeCL device selectors and adapter names.
  Alias: `list_devices`.
- `track <path>`: track one or more particles and print CSV samples.
- `optics <path>`: compute periodic optics, or propagate explicit initial
  optics with `--propagate`.
- `chromaticity <path>`: compute natural and sextupole-corrected chromaticity.
- `radiation <path>`: compute radiation integrals and derived beam quantities.
- `closed-orbit <path>`: solve the one-turn closed orbit. Alias:
  `closed_orbit`.
- `rdt <path>`: compute sextupole resonance driving terms.
- `stable-aperture <path>`: track an x/y grid and report the first lost turn.
  Alias: `stable_aperture`.

Run `cargo run -- --help` or `cargo run -- <command> --help` for full command
documentation.

## Common Flags

- `--line <name>` / `-l <name>` selects a line. If omitted, GUFO uses `RING` or
  the first parsed line.
- `--double` emits 64-bit bytecode and uses double-precision tracking.
- `--flag <name>` is repeatable. Supported pass flags are `linear`, `fived`,
  `exact`, `kick`, `radiation`, `double-precision`, and `achromatic`.
- `compile --instructions` prints decoded instructions with opcode, name, kind,
  flags, aux, and arguments.
- `compile --hex` prints encoded bytecode words.
- `--collapse-linear` is available on `compile` and `track`; it collapses
  consecutive affine linear transforms into `OP_TRAN_LINEAR`.

Runtime backend flags on simulation modes:

- `--backend <auto|cpu|vulkan|cuda|hip|metal>` chooses the CubeCL backend.
- `--device <selector>` chooses a listed device, for example
  `vulkan:integrated:0`, `vulkan:discrete:0`, `vulkan:cpu`, `cpu:0`, `cuda:0`,
  or `hip:0`.

Mode-specific flags:

- `track`: `--turns`, repeatable `--where`, initial particle coordinates
  `--x`, `--px`, `--y`, `--py`, `--z`, `--dp`, and one particle source.
- `optics`: repeatable `--where`; with `--propagate`, initial optics are set by
  `--ax`, `--bx`, `--dx`, `--dpx`, `--ay`, `--by`, `--dy`, and `--dpy`.
- `closed-orbit`: `--dp`, `--iterations`, and `--step`.
- `stable-aperture`: `--turns`, `--x-min`, `--x-max`, `--x-count`, `--y-min`,
  `--y-max`, `--y-count`, `--px`, `--py`, `--z`, and `--dp`.

## Particle Sources

`track` accepts one particle source. If no source is selected, `--particles <n>`
repeats the same initial particle.

Inline particles:

```bash
cargo run -- track optics/fodo.mad \
  --particle 0.001,0,0,0,0,0 \
  --particle 0.002,0,0,0,0,0
```

CSV particles:

```bash
cargo run -- track optics/fodo.mad --particles-file bunch.csv
```

Header columns may include `x`, `px`, `y`, `py`, `z`, and `dp`. Omitted columns
keep the base values from `--x`, `--px`, `--y`, `--py`, `--z`, and `--dp`.
Headerless CSV rows are read in `x,px,y,py,z,dp` order.

Random beam:

```bash
cargo run -- track optics/fodo.mad \
  --random --particles 1000 --seed 1 \
  --x-std 1e-3 --px-std 1e-4 --y-std 1e-3 --py-std 1e-4
```

Grid beam:

```bash
cargo run -- track optics/fodo.mad \
  --grid x=-0.001:0.001:5 \
  --grid y=-0.001:0.001:5
```

For negative numeric values, prefer the equals form so the CLI does not parse
the value as a new option:

```bash
cargo run -- stable-aperture optics/fodo.mad --x-min=-0.001 --y-min=-0.001
```

## Optional Backends

```bash
cargo run --no-default-features --features cubecl-cpu -- track optics/fodo.mad
cargo run --no-default-features --features cubecl-vulkan -- list-devices
cargo check --no-default-features --features cubecl-cuda
cargo check --no-default-features --features cubecl-hip
cargo check --no-default-features --features cubecl-metal
```

## Runtime Cache

Compiled runtime caches are stored under `~/.gufo/cache` by default. Set
`GUFO_CACHE_DIR` to choose another base directory. Existing runtime cache
environment variables are left unchanged.

## Python Interface

GUFO exposes an optional PyO3 extension module that follows the original Python
workflow for lattice loading and tracking.

Create a virtual environment and install the build tools:

```bash
python -m venv .venv
source .venv/bin/activate
python -m pip install -U pip maturin numpy
```

Install GUFO into the active environment for development:

```bash
maturin develop --features python
```

Alternatively, build and install a wheel:

```bash
maturin build --features python
python -m pip install target/wheels/gufo-*.whl
```

The `python` feature enables CubeCL CPU and CubeCL WGPU/Vulkan support. Python
tracking uses the same `auto` policy as the CLI.

Smoke test:

```bash
python - <<'PY'
import gufo

lat = gufo.Lattice("optics/fodo.mad")

tr = gufo.Track(
    lat.RING,
    turns=1,
    particles=2,
    where=[-1],
    flags=gufo.FIVED,
    parameters=["x"],
)

tr.parameters[:, 0] = [0.001, 0.002]
tr.run()

print(tr.tracks.shape)   # (particles, samples, 6)
print(tr.tracks[:, 0, 0])
PY
```

The compatibility layer currently supports coordinate parameters `x`, `px`,
`y`, `py`, `z`, and `dp`, mutable NumPy `parameters`, NumPy `tracks`, lattice
line access such as `lat.RING`, and the original pass-flag constants.
