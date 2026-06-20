
```
                   ██    ██ ███████  ██████  
                   ██    ██ ██      ██    ██ 
                   ██    ██ █████   ██    ██ 
                   ██    ██ ██      ██    ██ 
                    ██████  ██       ██████ 

 
                          \__/  ^__^
                          (oo)  (oo)
                         _/--\  /--\_
                   _.--===0=0====0=0===--._
                  (________________________)
                       /  \________/  \
                      /                \
 
 
       An Unreliable, but (Undoubtedly) Fast Optics code


                  Developed by Michele Carla',
    based on the initial work of Manu Canals and Michele Carla'
           with the support of ALBA (www.cells.es)

                https://github.com/mcarla/ufo
                


```

...***UFO*** is a fast accelerator optics toolkit designed with GPU in mind, nevertheless it gets along well with CPUs too.
UFO is not meant to be a general purpose tool, instead it aims to performance at expenses of flexibility and ease of use...

## Rust Crate

UFO is now a Rust crate. The implementation includes:

- a typed lattice and element model,
- a `pest`-based parser for the existing MAD fixture dialect,
- interpreter bytecode encoding shared by CPU and GPU backends,
- a CubeCL interpreter kernel with shared-memory instruction caching,
- a CubeCL runtime wrapper for CPU and Vulkan by default, with optional CUDA,
  HIP, and Metal features,
- CLI commands for loading, compiling, tracking, optics, chromaticity,
  radiation integrals, closed orbit, RDT, and stable aperture workflows.

## Requirements

The following packages are required to run UFO:

- Rust 1.96 or later
- Cargo
- A CubeCL-supported runtime for simulations. The default build enables CPU and
  Vulkan; CUDA, HIP, and Metal can be enabled with Cargo features.


## Install

The latest development version of UFO can be retrieved from github and installed with:

```
git clone https://github.com/mcarla/ufo
cd ufo
cargo test
```


## Getting started

A list of available CubeCL targets can be obtained with:

```
cargo run -- list-devices
```

Useful CLI commands:

```
cargo run -- load optics/fodo.mad
cargo run -- compile optics/fodo.mad --flag linear --flag achromatic
cargo run -- track optics/fodo.mad --turns 10 --where -1
cargo run -- optics optics/fodo.mad
cargo run -- chromaticity optics/fodo.mad
```

The output should resemble:

```
vulkan:integrated:0    AMD Radeon Graphics (RADV RENOIR)
vulkan:cpu             llvmpipe (LLVM 22.1.6, 256 bits)
cpu:0                  CubeCL CPU
```

The default simulation backend is the first Vulkan device if one is available;
otherwise UFO falls back to the CubeCL CPU runtime. Runtime commands accept
explicit backend and device selectors:

```
cargo run -- track optics/fodo.mad --backend cpu
cargo run -- track optics/fodo.mad --device vulkan:discrete:0
cargo run -- track optics/fodo.mad --backend cuda --device 0
```

The `list_devices` alias is also accepted for `list-devices`.

Compiled runtime caches are placed under `~/.ufo/cache` by default. Set
`UFO_CACHE_DIR` to choose another base directory; existing runtime cache
environment variables are left unchanged.

Optional backend feature examples:

```
cargo run --no-default-features --features cubecl-cpu -- track optics/fodo.mad
cargo run --no-default-features --features cubecl-vulkan -- list-devices
cargo check --no-default-features --features cubecl-cuda
cargo check --no-default-features --features cubecl-hip
cargo check --no-default-features --features cubecl-metal
```

## Documentation

Run `cargo run -- --help` or `cargo run -- <command> --help` for command
documentation.
