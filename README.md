
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
- OpenCL-compatible interpreter bytecode encoding,
- bundled OpenCL interpreter kernels under `src/kernels/`,
- an `opencl3`-based device/build wrapper,
- CLI commands for loading, compiling, tracking, optics, chromaticity,
  radiation integrals, closed orbit, RDT, and stable aperture workflows.

## Requirements

The following packages are required to run UFO:

- Rust 1.96 or later
- Cargo
- An OpenCL implementation for runtime execution and OpenCL build tests


## Install

The latest development version of UFO can be retrieved from github and installed with:

```
git clone https://github.com/mcarla/ufo
cd ufo
cargo test
```


## Getting started

At least one properly configured OpenCL back-end is required to run any simulation,
a list of the available OpenCL back-ends can be obtained with:
```
cargo run -- list-devices
```

Useful CLI commands:

```
cargo run -- load optics/fodo.mad
cargo run -- compile optics/fodo.mad --flag linear --flag achromatic
cargo run -- build-interpreter
```

For compatibility with the old Python API naming, `list_devices` is accepted as
an alias for `list-devices`.

The output should resemble:

```
0:   Quadro P600
1:   pthread-Intel(R) Core(TM) i5-8400 CPU @ 2.80GHz
```

In this example two back-ends are available: 0 is an Nvidia Quadro GPU, while 1 is an Intel i5 CPU.
Tracking and analysis commands accept MAD lattice files directly. For example:

```
cargo run -- track optics/fodo.mad --turns 10 --where -1
cargo run -- optics optics/fodo.mad
cargo run -- chromaticity optics/fodo.mad
```

## Documentation

Run `cargo run -- --help` or `cargo run -- <command> --help` for command
documentation.
