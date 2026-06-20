use std::{path::PathBuf, process::ExitCode};

use clap::{Parser, Subcommand, ValueEnum};
#[cfg(feature = "cubecl")]
use rand::{SeedableRng, rngs::StdRng};
#[cfg(feature = "cubecl")]
use rand_distr::{Distribution, Normal};
use ufo::{PassFlags, Result};

#[derive(Parser, Debug)]
#[command(name = "ufo")]
#[command(about = "Unreliable but Fast Optics code")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Load a MAD lattice and print a compact summary.
    #[command(alias = "lattice")]
    Load {
        /// MAD lattice file path.
        path: PathBuf,
        /// Line to inspect. Defaults to RING, or the first parsed line.
        #[arg(short, long)]
        line: Option<String>,
    },
    /// Compile a lattice line to interpreter bytecode and print metadata.
    #[command(alias = "bytecode")]
    Compile {
        /// MAD lattice file path.
        path: PathBuf,
        /// Line to compile. Defaults to RING, or the first parsed line.
        #[arg(short, long)]
        line: Option<String>,
        /// Emit 64-bit bytecode words.
        #[arg(long)]
        double: bool,
        /// Pass flags, repeatable: --flag linear --flag achromatic.
        #[arg(long = "flag", value_enum)]
        flags: Vec<FlagArg>,
        /// Print encoded words as hexadecimal.
        #[arg(long)]
        hex: bool,
        /// Print decoded instructions with opcode, name, flags, aux, and arguments.
        #[arg(long)]
        instructions: bool,
        /// Collapse consecutive affine linear transforms into OP_TRAN_LINEAR.
        #[arg(long)]
        collapse_linear: bool,
    },
    /// Dump a lattice to a MAD file.
    Dump {
        /// Input MAD lattice file path.
        input: PathBuf,
        /// Output MAD lattice file path.
        output: PathBuf,
        /// Output format.
        #[arg(long, value_enum, default_value = "mad")]
        style: DumpStyleArg,
    },
    /// List available CubeCL devices.
    #[cfg(feature = "cubecl")]
    #[command(alias = "list_devices")]
    ListDevices,
    /// Track particles through a lattice line with the interpreter backend.
    #[cfg(feature = "cubecl")]
    Track {
        /// MAD lattice file path.
        path: PathBuf,
        /// Line to track. Defaults to RING, or the first parsed line.
        #[arg(short, long)]
        line: Option<String>,
        /// Number of turns.
        #[arg(long, default_value_t = 1)]
        turns: u32,
        /// Number of particles for repeated or random generation.
        #[arg(long, default_value_t = 1)]
        particles: usize,
        /// Inline particle, repeatable. Format: x,px,y,py,z,dp.
        #[arg(long = "particle")]
        particle: Vec<String>,
        /// CSV particle table. Header columns may include x, px, y, py, z, dp.
        #[arg(long = "particles-file")]
        particles_file: Option<PathBuf>,
        /// Randomly generate particles around the initial coordinates.
        #[arg(long)]
        random: bool,
        /// Random generator seed.
        #[arg(long)]
        seed: Option<u64>,
        /// Horizontal position standard deviation for --random.
        #[arg(long, default_value_t = 0.0)]
        x_std: f64,
        /// Horizontal momentum standard deviation for --random.
        #[arg(long, default_value_t = 0.0)]
        px_std: f64,
        /// Vertical position standard deviation for --random.
        #[arg(long, default_value_t = 0.0)]
        y_std: f64,
        /// Vertical momentum standard deviation for --random.
        #[arg(long, default_value_t = 0.0)]
        py_std: f64,
        /// Longitudinal position standard deviation for --random.
        #[arg(long, default_value_t = 0.0)]
        z_std: f64,
        /// Relative momentum deviation standard deviation for --random.
        #[arg(long, default_value_t = 0.0)]
        dp_std: f64,
        /// Grid axis, repeatable. Format: x=min:max:count or x:min:max:count.
        #[arg(long = "grid", allow_hyphen_values = true)]
        grid: Vec<String>,
        /// Observation position, repeatable. Use -1 for end of turn.
        #[arg(long = "where", default_value = "-1", allow_hyphen_values = true)]
        where_: Vec<f64>,
        /// Initial horizontal position.
        #[arg(long, default_value_t = 0.0)]
        x: f64,
        /// Initial horizontal momentum.
        #[arg(long, default_value_t = 0.0)]
        px: f64,
        /// Initial vertical position.
        #[arg(long, default_value_t = 0.0)]
        y: f64,
        /// Initial vertical momentum.
        #[arg(long, default_value_t = 0.0)]
        py: f64,
        /// Initial longitudinal position.
        #[arg(long, default_value_t = 0.0)]
        z: f64,
        /// Initial relative momentum deviation.
        #[arg(long, default_value_t = 0.0)]
        dp: f64,
        /// Emit 64-bit bytecode and use double-precision CubeCL tracking.
        #[arg(long)]
        double: bool,
        /// Pass flags, repeatable: --flag linear --flag achromatic.
        #[arg(long = "flag", value_enum)]
        flags: Vec<FlagArg>,
        /// Collapse consecutive affine linear transforms into OP_TRAN_LINEAR.
        #[arg(long)]
        collapse_linear: bool,
        /// CubeCL backend to use. Defaults to first Vulkan device if present, then CPU.
        #[arg(long, value_enum, default_value = "auto")]
        backend: BackendArg,
        /// Device selector from `list-devices`, for example `vulkan:discrete:0`.
        #[arg(long)]
        device: Option<String>,
    },
    /// Compute periodic optics functions for a lattice line.
    #[cfg(feature = "cubecl")]
    Optics {
        /// MAD lattice file path.
        path: PathBuf,
        /// Line to analyze. Defaults to RING, or the first parsed line.
        #[arg(short, long)]
        line: Option<String>,
        /// Observation position, repeatable. Use -1 for end of turn.
        #[arg(long = "where", default_value = "-1", allow_hyphen_values = true)]
        where_: Vec<f64>,
        /// Propagate from explicit initial optics instead of solving periodic optics.
        #[arg(long)]
        propagate: bool,
        /// Initial horizontal alpha for --propagate.
        #[arg(long, default_value_t = 0.0)]
        ax: f64,
        /// Initial vertical alpha for --propagate.
        #[arg(long, default_value_t = 0.0)]
        ay: f64,
        /// Initial horizontal beta for --propagate.
        #[arg(long, default_value_t = 1.0)]
        bx: f64,
        /// Initial vertical beta for --propagate.
        #[arg(long, default_value_t = 1.0)]
        by: f64,
        /// Initial horizontal dispersion for --propagate.
        #[arg(long, default_value_t = 0.0)]
        dx: f64,
        /// Initial vertical dispersion for --propagate.
        #[arg(long, default_value_t = 0.0)]
        dy: f64,
        /// Initial horizontal dispersion slope for --propagate.
        #[arg(long, default_value_t = 0.0)]
        dpx: f64,
        /// Initial vertical dispersion slope for --propagate.
        #[arg(long, default_value_t = 0.0)]
        dpy: f64,
        /// Emit 64-bit bytecode and use double-precision CubeCL tracking.
        #[arg(long)]
        double: bool,
        /// Pass flags, repeatable: --flag linear --flag achromatic.
        #[arg(long = "flag", value_enum)]
        flags: Vec<FlagArg>,
        /// CubeCL backend to use. Defaults to first Vulkan device if present, then CPU.
        #[arg(long, value_enum, default_value = "auto")]
        backend: BackendArg,
        /// Device selector from `list-devices`, for example `vulkan:discrete:0`.
        #[arg(long)]
        device: Option<String>,
    },
    /// Compute natural and sextupole-corrected chromaticity.
    #[cfg(feature = "cubecl")]
    Chromaticity {
        /// MAD lattice file path.
        path: PathBuf,
        /// Line to analyze. Defaults to RING, or the first parsed line.
        #[arg(short, long)]
        line: Option<String>,
        /// Emit 64-bit bytecode and use double-precision CubeCL tracking.
        #[arg(long)]
        double: bool,
        /// Pass flags, repeatable: --flag linear --flag achromatic.
        #[arg(long = "flag", value_enum)]
        flags: Vec<FlagArg>,
        /// CubeCL backend to use. Defaults to first Vulkan device if present, then CPU.
        #[arg(long, value_enum, default_value = "auto")]
        backend: BackendArg,
        /// Device selector from `list-devices`, for example `vulkan:discrete:0`.
        #[arg(long)]
        device: Option<String>,
    },
    /// Compute radiation integrals and derived beam quantities.
    #[cfg(feature = "cubecl")]
    Radiation {
        /// MAD lattice file path.
        path: PathBuf,
        /// Line to analyze. Defaults to RING, or the first parsed line.
        #[arg(short, long)]
        line: Option<String>,
        /// Emit 64-bit bytecode and use double-precision CubeCL tracking.
        #[arg(long)]
        double: bool,
        /// Pass flags, repeatable: --flag linear --flag achromatic.
        #[arg(long = "flag", value_enum)]
        flags: Vec<FlagArg>,
        /// CubeCL backend to use. Defaults to first Vulkan device if present, then CPU.
        #[arg(long, value_enum, default_value = "auto")]
        backend: BackendArg,
        /// Device selector from `list-devices`, for example `vulkan:discrete:0`.
        #[arg(long)]
        device: Option<String>,
    },
    /// Find the one-turn closed orbit.
    #[cfg(feature = "cubecl")]
    #[command(alias = "closed_orbit")]
    ClosedOrbit {
        /// MAD lattice file path.
        path: PathBuf,
        /// Line to analyze. Defaults to RING, or the first parsed line.
        #[arg(short, long)]
        line: Option<String>,
        /// Fixed relative momentum deviation.
        #[arg(long, default_value_t = 0.0)]
        dp: f64,
        /// Maximum Nelder-Mead iterations.
        #[arg(long, default_value_t = 200)]
        iterations: usize,
        /// Initial simplex coordinate step.
        #[arg(long, default_value_t = 1.0e-4)]
        step: f64,
        /// Emit 64-bit bytecode and use double-precision CubeCL tracking.
        #[arg(long)]
        double: bool,
        /// Pass flags, repeatable: --flag linear --flag achromatic.
        #[arg(long = "flag", value_enum)]
        flags: Vec<FlagArg>,
        /// CubeCL backend to use. Defaults to first Vulkan device if present, then CPU.
        #[arg(long, value_enum, default_value = "auto")]
        backend: BackendArg,
        /// Device selector from `list-devices`, for example `vulkan:discrete:0`.
        #[arg(long)]
        device: Option<String>,
    },
    /// Compute sextupole resonance driving terms.
    #[cfg(feature = "cubecl")]
    Rdt {
        /// MAD lattice file path.
        path: PathBuf,
        /// Line to analyze. Defaults to RING, or the first parsed line.
        #[arg(short, long)]
        line: Option<String>,
        /// Emit 64-bit bytecode and use double-precision CubeCL tracking.
        #[arg(long)]
        double: bool,
        /// Pass flags, repeatable: --flag linear --flag achromatic.
        #[arg(long = "flag", value_enum)]
        flags: Vec<FlagArg>,
        /// CubeCL backend to use. Defaults to first Vulkan device if present, then CPU.
        #[arg(long, value_enum, default_value = "auto")]
        backend: BackendArg,
        /// Device selector from `list-devices`, for example `vulkan:discrete:0`.
        #[arg(long)]
        device: Option<String>,
    },
    /// Track an x/y grid and report first lost turn per particle.
    #[cfg(feature = "cubecl")]
    #[command(alias = "stable_aperture")]
    StableAperture {
        /// MAD lattice file path.
        path: PathBuf,
        /// Line to track. Defaults to RING, or the first parsed line.
        #[arg(short, long)]
        line: Option<String>,
        /// Number of turns.
        #[arg(long, default_value_t = 1000)]
        turns: u32,
        /// Horizontal grid minimum.
        #[arg(long, default_value_t = -0.01)]
        x_min: f64,
        /// Horizontal grid maximum.
        #[arg(long, default_value_t = 0.01)]
        x_max: f64,
        /// Horizontal grid point count.
        #[arg(long, default_value_t = 11)]
        x_count: usize,
        /// Vertical grid minimum.
        #[arg(long, default_value_t = -0.01)]
        y_min: f64,
        /// Vertical grid maximum.
        #[arg(long, default_value_t = 0.01)]
        y_max: f64,
        /// Vertical grid point count.
        #[arg(long, default_value_t = 11)]
        y_count: usize,
        /// Initial horizontal momentum.
        #[arg(long, default_value_t = 0.0)]
        px: f64,
        /// Initial vertical momentum.
        #[arg(long, default_value_t = 0.0)]
        py: f64,
        /// Initial longitudinal position.
        #[arg(long, default_value_t = 0.0)]
        z: f64,
        /// Initial relative momentum deviation.
        #[arg(long, default_value_t = 0.0)]
        dp: f64,
        /// Emit 64-bit bytecode and use double-precision CubeCL tracking.
        #[arg(long)]
        double: bool,
        /// Pass flags, repeatable: --flag linear --flag achromatic.
        #[arg(long = "flag", value_enum)]
        flags: Vec<FlagArg>,
        /// CubeCL backend to use. Defaults to first Vulkan device if present, then CPU.
        #[arg(long, value_enum, default_value = "auto")]
        backend: BackendArg,
        /// Device selector from `list-devices`, for example `vulkan:discrete:0`.
        #[arg(long)]
        device: Option<String>,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum FlagArg {
    Linear,
    Fived,
    Exact,
    Kick,
    Radiation,
    DoublePrecision,
    Achromatic,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum DumpStyleArg {
    Mad,
    Elegant,
    At,
    Opa,
}

#[cfg(feature = "cubecl")]
#[derive(Clone, Copy, Debug, ValueEnum)]
enum BackendArg {
    Auto,
    Cpu,
    Vulkan,
    Cuda,
    Hip,
    Metal,
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<()> {
    match cli.command {
        #[cfg(feature = "cubecl")]
        Command::ListDevices => list_devices(),
        Command::Load { path, line } => load(path, line),
        Command::Compile {
            path,
            line,
            double,
            flags,
            hex,
            instructions,
            collapse_linear,
        } => compile(
            path,
            line,
            double,
            flags,
            hex,
            instructions,
            collapse_linear,
        ),
        Command::Dump {
            input,
            output,
            style,
        } => dump(input, output, style),
        #[cfg(feature = "cubecl")]
        Command::Track {
            path,
            line,
            turns,
            particles,
            particle,
            particles_file,
            random,
            seed,
            x_std,
            px_std,
            y_std,
            py_std,
            z_std,
            dp_std,
            grid,
            where_,
            x,
            px,
            y,
            py,
            z,
            dp,
            double,
            flags,
            collapse_linear,
            backend,
            device,
        } => track(TrackArgs {
            path,
            line,
            turns,
            particles,
            particle,
            particles_file,
            random,
            seed,
            x_std,
            px_std,
            y_std,
            py_std,
            z_std,
            dp_std,
            grid,
            where_,
            x,
            px,
            y,
            py,
            z,
            dp,
            double,
            flags,
            collapse_linear,
            backend,
            device,
        }),
        #[cfg(feature = "cubecl")]
        Command::Optics {
            path,
            line,
            where_,
            propagate,
            ax,
            ay,
            bx,
            by,
            dx,
            dy,
            dpx,
            dpy,
            double,
            flags,
            backend,
            device,
        } => optics(OpticsArgs {
            path,
            line,
            where_,
            propagate,
            ax,
            ay,
            bx,
            by,
            dx,
            dy,
            dpx,
            dpy,
            double,
            flags,
            backend,
            device,
        }),
        #[cfg(feature = "cubecl")]
        Command::Chromaticity {
            path,
            line,
            double,
            flags,
            backend,
            device,
        } => chromaticity(ChromaticityArgs {
            path,
            line,
            double,
            flags,
            backend,
            device,
        }),
        #[cfg(feature = "cubecl")]
        Command::Radiation {
            path,
            line,
            double,
            flags,
            backend,
            device,
        } => radiation(ChromaticityArgs {
            path,
            line,
            double,
            flags,
            backend,
            device,
        }),
        #[cfg(feature = "cubecl")]
        Command::ClosedOrbit {
            path,
            line,
            dp,
            iterations,
            step,
            double,
            flags,
            backend,
            device,
        } => closed_orbit(ClosedOrbitArgs {
            path,
            line,
            dp,
            iterations,
            step,
            double,
            flags,
            backend,
            device,
        }),
        #[cfg(feature = "cubecl")]
        Command::Rdt {
            path,
            line,
            double,
            flags,
            backend,
            device,
        } => rdt(ChromaticityArgs {
            path,
            line,
            double,
            flags,
            backend,
            device,
        }),
        #[cfg(feature = "cubecl")]
        Command::StableAperture {
            path,
            line,
            turns,
            x_min,
            x_max,
            x_count,
            y_min,
            y_max,
            y_count,
            px,
            py,
            z,
            dp,
            double,
            flags,
            backend,
            device,
        } => stable_aperture(StableApertureArgs {
            path,
            line,
            turns,
            x_min,
            x_max,
            x_count,
            y_min,
            y_max,
            y_count,
            px,
            py,
            z,
            dp,
            double,
            flags,
            backend,
            device,
        }),
    }
}

fn dump(input: PathBuf, output: PathBuf, style: DumpStyleArg) -> Result<()> {
    let lattice = ufo::load_mad_file(&input)?;
    ufo::dump_lattice_file(&lattice, &output, style.into())?;
    println!("wrote: {}", output.display());
    Ok(())
}

impl From<DumpStyleArg> for ufo::DumpStyle {
    fn from(value: DumpStyleArg) -> Self {
        match value {
            DumpStyleArg::Mad => Self::Mad,
            DumpStyleArg::Elegant => Self::Elegant,
            DumpStyleArg::At => Self::At,
            DumpStyleArg::Opa => Self::Opa,
        }
    }
}

#[cfg(feature = "cubecl")]
impl From<BackendArg> for ufo::cubecl::CubeClBackend {
    fn from(value: BackendArg) -> Self {
        match value {
            BackendArg::Auto => Self::Auto,
            BackendArg::Cpu => Self::Cpu,
            BackendArg::Vulkan => Self::Vulkan,
            BackendArg::Cuda => Self::Cuda,
            BackendArg::Hip => Self::Hip,
            BackendArg::Metal => Self::Metal,
        }
    }
}

#[cfg(feature = "cubecl")]
struct TrackArgs {
    path: PathBuf,
    line: Option<String>,
    turns: u32,
    particles: usize,
    particle: Vec<String>,
    particles_file: Option<PathBuf>,
    random: bool,
    seed: Option<u64>,
    x_std: f64,
    px_std: f64,
    y_std: f64,
    py_std: f64,
    z_std: f64,
    dp_std: f64,
    grid: Vec<String>,
    where_: Vec<f64>,
    x: f64,
    px: f64,
    y: f64,
    py: f64,
    z: f64,
    dp: f64,
    double: bool,
    flags: Vec<FlagArg>,
    collapse_linear: bool,
    backend: BackendArg,
    device: Option<String>,
}

#[cfg(feature = "cubecl")]
struct OpticsArgs {
    path: PathBuf,
    line: Option<String>,
    where_: Vec<f64>,
    propagate: bool,
    ax: f64,
    ay: f64,
    bx: f64,
    by: f64,
    dx: f64,
    dy: f64,
    dpx: f64,
    dpy: f64,
    double: bool,
    flags: Vec<FlagArg>,
    backend: BackendArg,
    device: Option<String>,
}

#[cfg(feature = "cubecl")]
struct ChromaticityArgs {
    path: PathBuf,
    line: Option<String>,
    double: bool,
    flags: Vec<FlagArg>,
    backend: BackendArg,
    device: Option<String>,
}

#[cfg(feature = "cubecl")]
struct ClosedOrbitArgs {
    path: PathBuf,
    line: Option<String>,
    dp: f64,
    iterations: usize,
    step: f64,
    double: bool,
    flags: Vec<FlagArg>,
    backend: BackendArg,
    device: Option<String>,
}

#[cfg(feature = "cubecl")]
struct StableApertureArgs {
    path: PathBuf,
    line: Option<String>,
    turns: u32,
    x_min: f64,
    x_max: f64,
    x_count: usize,
    y_min: f64,
    y_max: f64,
    y_count: usize,
    px: f64,
    py: f64,
    z: f64,
    dp: f64,
    double: bool,
    flags: Vec<FlagArg>,
    backend: BackendArg,
    device: Option<String>,
}

#[cfg(feature = "cubecl")]
fn list_devices() -> Result<()> {
    init_cubecl_cache()?;
    for device in ufo::cubecl::list_devices() {
        if let Some(name) = device.name {
            println!("{}\t{}", device.selector, name);
        } else {
            println!("{}", device.selector);
        }
    }
    Ok(())
}

#[cfg(feature = "cubecl")]
fn run_options(
    backend: BackendArg,
    device: Option<String>,
    turns: u32,
) -> Result<ufo::cubecl::CubeClTrackRunOptions> {
    init_cubecl_cache()?;
    let mut backend = ufo::cubecl::CubeClBackend::from(backend);
    if backend == ufo::cubecl::CubeClBackend::Auto {
        if let Some(device_backend) = device
            .as_deref()
            .and_then(infer_backend_from_device_selector)
        {
            backend = device_backend;
        }
    }
    let device = parse_device_selector(backend, device)?;
    Ok(ufo::cubecl::CubeClTrackRunOptions {
        turns,
        backend,
        device,
    })
}

#[cfg(feature = "cubecl")]
fn parse_device_selector(
    backend: ufo::cubecl::CubeClBackend,
    device: Option<String>,
) -> Result<ufo::cubecl::CubeClDevice> {
    let Some(raw) = device else {
        return Ok(ufo::cubecl::CubeClDevice::Default);
    };
    let mut selector = raw.trim();
    for (prefix, parsed_backend) in [
        ("cpu:", ufo::cubecl::CubeClBackend::Cpu),
        ("vulkan:", ufo::cubecl::CubeClBackend::Vulkan),
        ("cuda:", ufo::cubecl::CubeClBackend::Cuda),
        ("hip:", ufo::cubecl::CubeClBackend::Hip),
        ("metal:", ufo::cubecl::CubeClBackend::Metal),
    ] {
        if let Some(stripped) = selector.strip_prefix(prefix) {
            selector = stripped;
            if backend != ufo::cubecl::CubeClBackend::Auto && backend != parsed_backend {
                return Err(ufo::UfoError::Parse(format!(
                    "device selector `{raw}` does not match backend {:?}",
                    backend
                )));
            }
            break;
        }
    }

    let device = if selector.eq_ignore_ascii_case("default") {
        ufo::cubecl::CubeClDevice::Default
    } else if selector.eq_ignore_ascii_case("cpu")
        || selector == "0" && backend == ufo::cubecl::CubeClBackend::Cpu
    {
        ufo::cubecl::CubeClDevice::WgpuCpu
    } else if let Some(index) = selector.strip_prefix("discrete:") {
        ufo::cubecl::CubeClDevice::DiscreteGpu(parse_device_index(index, &raw)?)
    } else if let Some(index) = selector.strip_prefix("integrated:") {
        ufo::cubecl::CubeClDevice::IntegratedGpu(parse_device_index(index, &raw)?)
    } else if let Some(index) = selector.strip_prefix("virtual:") {
        ufo::cubecl::CubeClDevice::VirtualGpu(parse_device_index(index, &raw)?)
    } else if let Ok(index) = selector.parse::<usize>() {
        match backend {
            ufo::cubecl::CubeClBackend::Vulkan | ufo::cubecl::CubeClBackend::Metal => {
                ufo::cubecl::CubeClDevice::DiscreteGpu(index)
            }
            _ => ufo::cubecl::CubeClDevice::Index(index),
        }
    } else {
        return Err(ufo::UfoError::Parse(format!(
            "invalid device selector `{raw}`"
        )));
    };
    Ok(device)
}

#[cfg(feature = "cubecl")]
fn infer_backend_from_device_selector(selector: &str) -> Option<ufo::cubecl::CubeClBackend> {
    if selector.starts_with("cpu:") {
        Some(ufo::cubecl::CubeClBackend::Cpu)
    } else if selector.starts_with("vulkan:") {
        Some(ufo::cubecl::CubeClBackend::Vulkan)
    } else if selector.starts_with("cuda:") {
        Some(ufo::cubecl::CubeClBackend::Cuda)
    } else if selector.starts_with("hip:") {
        Some(ufo::cubecl::CubeClBackend::Hip)
    } else if selector.starts_with("metal:") {
        Some(ufo::cubecl::CubeClBackend::Metal)
    } else {
        None
    }
}

#[cfg(feature = "cubecl")]
fn parse_device_index(value: &str, raw: &str) -> Result<usize> {
    value
        .parse::<usize>()
        .map_err(|_| ufo::UfoError::Parse(format!("invalid device selector `{raw}`")))
}

#[cfg(feature = "cubecl")]
fn init_cubecl_cache() -> Result<()> {
    let base = std::env::var_os("UFO_CACHE_DIR")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".ufo")))
        .unwrap_or_else(|| PathBuf::from(".ufo"));
    let cache = base.join("cache");
    let cuda_cache = cache.join("cuda");
    let hip_cache = cache.join("hip");
    std::fs::create_dir_all(&cuda_cache).map_err(|source| ufo::UfoError::WriteFile {
        path: cuda_cache.clone(),
        source,
    })?;
    std::fs::create_dir_all(&hip_cache).map_err(|source| ufo::UfoError::WriteFile {
        path: hip_cache.clone(),
        source,
    })?;
    set_env_if_missing("XDG_CACHE_HOME", &cache);
    set_env_if_missing("CUDA_CACHE_PATH", &cuda_cache);
    set_env_if_missing("HIP_CACHE_DIR", &hip_cache);
    set_env_if_missing("AMD_COMGR_CACHE_DIR", &hip_cache);
    Ok(())
}

#[cfg(feature = "cubecl")]
fn set_env_if_missing(name: &str, value: &std::path::Path) {
    if std::env::var_os(name).is_none() {
        unsafe {
            std::env::set_var(name, value);
        }
    }
}

fn load(path: PathBuf, line: Option<String>) -> Result<()> {
    let lattice = ufo::load_mad_file(&path)?;
    println!("file: {}", path.display());
    println!("elements: {}", lattice.elements.len());
    println!("lines: {}", lattice.lines.len());
    if let Some(line_name) = select_line_name(&lattice, line) {
        let line = lattice.line(&line_name)?;
        let flat = line.flatten(&lattice)?;
        println!("line: {}", line.label);
        println!("line_elements: {}", flat.len());
        println!("line_length: {:.12}", line.length(&lattice)?);
        println!("line_angle: {:.12}", line.angle(&lattice)?);
    }
    Ok(())
}

fn compile(
    path: PathBuf,
    line: Option<String>,
    double: bool,
    flags: Vec<FlagArg>,
    hex: bool,
    instructions: bool,
    collapse_linear: bool,
) -> Result<()> {
    let lattice = ufo::load_mad_file(&path)?;
    let line_name = select_line_name(&lattice, line)
        .ok_or_else(|| ufo::UfoError::UnknownReference("no line found".to_string()))?;
    let line = lattice.line(&line_name)?;
    let flags = flags
        .into_iter()
        .fold(PassFlags::empty(), |acc, flag| acc | flag.into());
    let bytecode = ufo::compile_line(
        &lattice,
        line,
        &ufo::compiler::CompileOptions {
            flags,
            turns: 1,
            is_64bit: double,
            collapse_linear,
        },
    )?;

    println!("line: {line_name}");
    println!("instructions: {}", bytecode.instructions().len());
    println!("words: {}", bytecode.word_count());
    println!("bytes: {}", bytecode.emit_bytes()?.len());

    if instructions {
        println!("index,opcode,name,kind,flags,aux,args");
        for (idx, instruction) in bytecode.instructions().iter().enumerate() {
            println!(
                "{idx},{},{},{},{},{},{}",
                instruction.op,
                instruction.name,
                instruction.kind,
                format_pass_flags(instruction.flags),
                instruction.aux,
                format_args(&instruction.args)
            );
        }
    }

    if hex {
        if double {
            for word in bytecode.emit_u64_words()? {
                println!("{word:016x}");
            }
        } else {
            for word in bytecode.emit_u32_words()? {
                println!("{word:08x}");
            }
        }
    }
    Ok(())
}

fn format_pass_flags(flags: PassFlags) -> String {
    let mut names = Vec::new();
    for (flag, name) in [
        (PassFlags::LINEAR, "linear"),
        (PassFlags::FIVED, "fived"),
        (PassFlags::EXACT, "exact"),
        (PassFlags::KICK, "kick"),
        (PassFlags::RADIATION, "radiation"),
        (PassFlags::DOUBLE_PRECISION, "double-precision"),
        (PassFlags::ACHROMATIC, "achromatic"),
        (PassFlags::NO_APERTURE_CHECK, "no-aperture-check"),
    ] {
        if flags.contains(flag) {
            names.push(name);
        }
    }
    if names.is_empty() {
        "-".to_string()
    } else {
        names.join("|")
    }
}

fn format_args(args: &[f64]) -> String {
    if args.is_empty() {
        return "[]".to_string();
    }
    let values = args
        .iter()
        .map(|value| format!("{value:.12}"))
        .collect::<Vec<_>>()
        .join(";");
    format!("[{values}]")
}

#[cfg(feature = "cubecl")]
#[derive(Clone, Copy)]
struct ParticleScales {
    x: f64,
    px: f64,
    y: f64,
    py: f64,
    z: f64,
    dp: f64,
}

#[cfg(feature = "cubecl")]
#[derive(Clone, Copy)]
enum ParticleCoord {
    X,
    Px,
    Y,
    Py,
    Z,
    Dp,
}

#[cfg(feature = "cubecl")]
struct GridAxis {
    coord: ParticleCoord,
    values: Vec<f64>,
}

#[cfg(feature = "cubecl")]
fn build_track_particles(args: &TrackArgs) -> Result<Vec<ufo::Particle>> {
    let base = ufo::Particle {
        x: args.x,
        px: args.px,
        y: args.y,
        py: args.py,
        z: args.z,
        dp: args.dp,
        ..ufo::Particle::default()
    };
    let source_count = (!args.particle.is_empty()) as u8
        + args.particles_file.is_some() as u8
        + args.random as u8
        + (!args.grid.is_empty()) as u8;
    if source_count > 1 {
        return Err(ufo::UfoError::Parse(
            "choose only one particle source: --particle, --particles-file, --random, or --grid"
                .to_string(),
        ));
    }
    if !args.particle.is_empty() {
        args.particle
            .iter()
            .map(|raw| parse_inline_particle(raw, base))
            .collect()
    } else if let Some(path) = &args.particles_file {
        read_particles_csv(path, base)
    } else if args.random {
        random_particles(
            base,
            args.particles,
            ParticleScales {
                x: args.x_std,
                px: args.px_std,
                y: args.y_std,
                py: args.py_std,
                z: args.z_std,
                dp: args.dp_std,
            },
            args.seed,
        )
    } else if !args.grid.is_empty() {
        grid_particles(base, &args.grid)
    } else {
        Ok(vec![base; args.particles])
    }
}

#[cfg(feature = "cubecl")]
fn parse_inline_particle(raw: &str, base: ufo::Particle) -> Result<ufo::Particle> {
    let values = raw
        .split([',', ' ', '\t'])
        .filter(|value| !value.is_empty())
        .map(|value| parse_f64(value, "particle coordinate"))
        .collect::<Result<Vec<_>>>()?;
    if values.is_empty() || values.len() > 6 {
        return Err(ufo::UfoError::Parse(format!(
            "particle `{raw}` must contain 1 to 6 values: x,px,y,py,z,dp"
        )));
    }
    Ok(particle_from_values(base, &values))
}

#[cfg(feature = "cubecl")]
fn read_particles_csv(path: &PathBuf, base: ufo::Particle) -> Result<Vec<ufo::Particle>> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .trim(csv::Trim::All)
        .from_path(path)
        .map_err(|source| ufo::UfoError::ReadFile {
            path: path.clone(),
            source: std::io::Error::other(source),
        })?;
    let mut records = reader.records();
    let Some(first) = records.next() else {
        return Err(ufo::UfoError::Parse(format!(
            "particle CSV `{}` is empty",
            path.display()
        )));
    };
    let first = first.map_err(|source| ufo::UfoError::Parse(source.to_string()))?;
    let mut particles = Vec::new();
    if let Some(columns) = particle_csv_columns(&first) {
        for record in records {
            let record = record.map_err(|source| ufo::UfoError::Parse(source.to_string()))?;
            particles.push(particle_from_named_record(base, &columns, &record)?);
        }
    } else {
        particles.push(particle_from_csv_record(base, &first)?);
        for record in records {
            let record = record.map_err(|source| ufo::UfoError::Parse(source.to_string()))?;
            particles.push(particle_from_csv_record(base, &record)?);
        }
    }
    if particles.is_empty() {
        return Err(ufo::UfoError::Parse(format!(
            "particle CSV `{}` contains no particles",
            path.display()
        )));
    }
    Ok(particles)
}

#[cfg(feature = "cubecl")]
fn particle_csv_columns(header: &csv::StringRecord) -> Option<Vec<(usize, ParticleCoord)>> {
    let mut columns = Vec::new();
    for (idx, name) in header.iter().enumerate() {
        if let Some(coord) = parse_particle_coord(name) {
            columns.push((idx, coord));
        }
    }
    (!columns.is_empty()).then_some(columns)
}

#[cfg(feature = "cubecl")]
fn particle_from_named_record(
    mut particle: ufo::Particle,
    columns: &[(usize, ParticleCoord)],
    record: &csv::StringRecord,
) -> Result<ufo::Particle> {
    for (idx, coord) in columns {
        if let Some(raw) = record.get(*idx)
            && !raw.trim().is_empty()
        {
            set_particle_coord(
                &mut particle,
                *coord,
                parse_f64(raw, "CSV particle coordinate")?,
            );
        }
    }
    Ok(particle)
}

#[cfg(feature = "cubecl")]
fn particle_from_csv_record(
    base: ufo::Particle,
    record: &csv::StringRecord,
) -> Result<ufo::Particle> {
    if record.len() > 6 {
        return Err(ufo::UfoError::Parse(
            "particle CSV rows without a header may contain at most 6 columns".to_string(),
        ));
    }
    let values = record
        .iter()
        .map(|value| parse_f64(value, "CSV particle coordinate"))
        .collect::<Result<Vec<_>>>()?;
    Ok(particle_from_values(base, &values))
}

#[cfg(feature = "cubecl")]
fn particle_from_values(mut particle: ufo::Particle, values: &[f64]) -> ufo::Particle {
    for (idx, value) in values.iter().enumerate() {
        match idx {
            0 => particle.x = *value,
            1 => particle.px = *value,
            2 => particle.y = *value,
            3 => particle.py = *value,
            4 => particle.z = *value,
            5 => particle.dp = *value,
            _ => unreachable!(),
        }
    }
    particle
}

#[cfg(feature = "cubecl")]
fn random_particles(
    base: ufo::Particle,
    count: usize,
    stds: ParticleScales,
    seed: Option<u64>,
) -> Result<Vec<ufo::Particle>> {
    if count == 0 {
        return Err(ufo::UfoError::Parse(
            "particles must be greater than zero".to_string(),
        ));
    }
    for (name, std) in [
        ("x-std", stds.x),
        ("px-std", stds.px),
        ("y-std", stds.y),
        ("py-std", stds.py),
        ("z-std", stds.z),
        ("dp-std", stds.dp),
    ] {
        if std < 0.0 {
            return Err(ufo::UfoError::Parse(format!("{name} must be non-negative")));
        }
    }
    let mut rng = match seed {
        Some(seed) => StdRng::seed_from_u64(seed),
        None => StdRng::from_entropy(),
    };
    (0..count)
        .map(|_| {
            Ok(ufo::Particle {
                x: sample_normal(base.x, stds.x, &mut rng)?,
                px: sample_normal(base.px, stds.px, &mut rng)?,
                y: sample_normal(base.y, stds.y, &mut rng)?,
                py: sample_normal(base.py, stds.py, &mut rng)?,
                z: sample_normal(base.z, stds.z, &mut rng)?,
                dp: sample_normal(base.dp, stds.dp, &mut rng)?,
                ..ufo::Particle::default()
            })
        })
        .collect()
}

#[cfg(feature = "cubecl")]
fn sample_normal(mean: f64, std: f64, rng: &mut StdRng) -> Result<f64> {
    if std == 0.0 {
        Ok(mean)
    } else {
        let normal = Normal::new(mean, std).map_err(|error| {
            ufo::UfoError::Parse(format!("invalid normal distribution: {error}"))
        })?;
        Ok(normal.sample(rng))
    }
}

#[cfg(feature = "cubecl")]
fn grid_particles(base: ufo::Particle, raw_axes: &[String]) -> Result<Vec<ufo::Particle>> {
    let axes = raw_axes
        .iter()
        .map(|raw| parse_grid_axis(raw))
        .collect::<Result<Vec<_>>>()?;
    let mut particles = vec![base];
    for axis in axes {
        let mut next = Vec::with_capacity(particles.len() * axis.values.len());
        for particle in &particles {
            for value in &axis.values {
                let mut particle = *particle;
                set_particle_coord(&mut particle, axis.coord, *value);
                next.push(particle);
            }
        }
        particles = next;
    }
    Ok(particles)
}

#[cfg(feature = "cubecl")]
fn parse_grid_axis(raw: &str) -> Result<GridAxis> {
    let (coord, range) = raw
        .split_once('=')
        .or_else(|| raw.split_once(':'))
        .ok_or_else(|| {
            ufo::UfoError::Parse(format!(
                "grid `{raw}` must have format x=min:max:count or x:min:max:count"
            ))
        })?;
    let coord = parse_particle_coord(coord)
        .ok_or_else(|| ufo::UfoError::Parse(format!("unknown grid coordinate `{coord}`")))?;
    let parts = range.split(':').collect::<Vec<_>>();
    if parts.len() != 3 {
        return Err(ufo::UfoError::Parse(format!(
            "grid `{raw}` must have min, max, and count"
        )));
    }
    let min = parse_f64(parts[0], "grid minimum")?;
    let max = parse_f64(parts[1], "grid maximum")?;
    let count = parts[2]
        .parse::<usize>()
        .map_err(|_| ufo::UfoError::Parse(format!("invalid grid count `{}`", parts[2])))?;
    if count == 0 {
        return Err(ufo::UfoError::Parse(
            "grid count must be greater than zero".to_string(),
        ));
    }
    Ok(GridAxis {
        coord,
        values: linspace(min, max, count),
    })
}

#[cfg(feature = "cubecl")]
fn parse_particle_coord(raw: &str) -> Option<ParticleCoord> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "x" => Some(ParticleCoord::X),
        "px" => Some(ParticleCoord::Px),
        "y" => Some(ParticleCoord::Y),
        "py" => Some(ParticleCoord::Py),
        "z" => Some(ParticleCoord::Z),
        "dp" => Some(ParticleCoord::Dp),
        _ => None,
    }
}

#[cfg(feature = "cubecl")]
fn set_particle_coord(particle: &mut ufo::Particle, coord: ParticleCoord, value: f64) {
    match coord {
        ParticleCoord::X => particle.x = value,
        ParticleCoord::Px => particle.px = value,
        ParticleCoord::Y => particle.y = value,
        ParticleCoord::Py => particle.py = value,
        ParticleCoord::Z => particle.z = value,
        ParticleCoord::Dp => particle.dp = value,
    }
}

#[cfg(feature = "cubecl")]
fn parse_f64(raw: &str, context: &str) -> Result<f64> {
    raw.trim()
        .parse::<f64>()
        .map_err(|_| ufo::UfoError::Parse(format!("invalid {context} `{raw}`")))
}

#[cfg(feature = "cubecl")]
fn track(args: TrackArgs) -> Result<()> {
    let particles = build_track_particles(&args)?;
    let particle_count = particles.len();
    let lattice = ufo::load_mad_file(&args.path)?;
    let line_name = select_line_name(&lattice, args.line)
        .ok_or_else(|| ufo::UfoError::UnknownReference("no line found".to_string()))?;
    let line = lattice.line(&line_name)?;
    let flags = args
        .flags
        .into_iter()
        .fold(PassFlags::empty(), |acc, flag| acc | flag.into());
    let mut track = ufo::Track::new(
        &lattice,
        line,
        particles,
        ufo::TrackOptions {
            flags,
            turns: args.turns,
            where_: args.where_,
            is_64bit: args.double,
            collapse_linear: args.collapse_linear,
        },
    )?;
    track.run_with_options(&run_options(args.backend, args.device, args.turns)?)?;

    println!("line: {line_name}");
    println!("turns: {}", args.turns);
    println!("particles: {}", particle_count);
    println!("samples: {}", track.samples.len());
    println!("sample,particle,x,px,y,py,z,dp,passed_elements,alive");
    for (sample, chunk) in track.samples.chunks(particle_count).enumerate() {
        for (particle, value) in chunk.iter().enumerate() {
            println!(
                "{sample},{particle},{:.12},{:.12},{:.12},{:.12},{:.12},{:.12},{},{}",
                value.x,
                value.px,
                value.y,
                value.py,
                value.z,
                value.dp,
                value.passed_elements,
                value.alive
            );
        }
    }
    Ok(())
}

#[cfg(feature = "cubecl")]
fn optics(args: OpticsArgs) -> Result<()> {
    let lattice = ufo::load_mad_file(&args.path)?;
    let line_name = select_line_name(&lattice, args.line)
        .ok_or_else(|| ufo::UfoError::UnknownReference("no line found".to_string()))?;
    let line = lattice.line(&line_name)?;
    let mut flags = args
        .flags
        .into_iter()
        .fold(PassFlags::empty(), |acc, flag| acc | flag.into());
    if flags.is_empty() {
        flags = PassFlags::LINEAR | PassFlags::ACHROMATIC;
    }
    let options = ufo::OpticsOptions {
        where_: args.where_,
        flags,
        is_64bit: args.double,
        run_options: run_options(args.backend, args.device, 1)?,
    };
    let optics = if args.propagate {
        ufo::Optics::propagate(
            &lattice,
            line,
            ufo::InitialOptics {
                ax: args.ax,
                ay: args.ay,
                bx: args.bx,
                by: args.by,
                dx: args.dx,
                dy: args.dy,
                dpx: args.dpx,
                dpy: args.dpy,
            },
            options,
        )?
    } else {
        ufo::Optics::periodic(&lattice, line, options)?
    };

    println!("line: {line_name}");
    println!("qx: {:.12}", optics.qx);
    println!("qy: {:.12}", optics.qy);
    println!("where,ax,bx,dx,dpx,mux,ay,by,dy,dpy,muy");
    for (where_, point) in optics.where_.iter().zip(&optics.points) {
        println!(
            "{:.12},{:.12},{:.12},{:.12},{:.12},{:.12},{:.12},{:.12},{:.12},{:.12},{:.12}",
            where_,
            point.ax,
            point.bx,
            point.dx,
            point.dpx,
            point.mux,
            point.ay,
            point.by,
            point.dy,
            point.dpy,
            point.muy
        );
    }
    Ok(())
}

#[cfg(feature = "cubecl")]
fn chromaticity(args: ChromaticityArgs) -> Result<()> {
    let lattice = ufo::load_mad_file(&args.path)?;
    let line_name = select_line_name(&lattice, args.line)
        .ok_or_else(|| ufo::UfoError::UnknownReference("no line found".to_string()))?;
    let line = lattice.line(&line_name)?;
    let mut flags = args
        .flags
        .into_iter()
        .fold(PassFlags::empty(), |acc, flag| acc | flag.into());
    if flags.is_empty() {
        flags = PassFlags::LINEAR | PassFlags::ACHROMATIC;
    }
    let optics = ufo::Optics::periodic(
        &lattice,
        line,
        ufo::OpticsOptions {
            where_: ufo::chromaticity_observations(&lattice, line)?,
            flags,
            is_64bit: args.double,
            run_options: run_options(args.backend, args.device, 1)?,
        },
    )?;
    let chromaticity = ufo::chromaticity(&lattice, line, &optics)?;

    println!("line: {line_name}");
    println!("natural_dqx: {:.12}", chromaticity.natural[0]);
    println!("natural_dqy: {:.12}", chromaticity.natural[1]);
    println!("dqx: {:.12}", chromaticity.total[0]);
    println!("dqy: {:.12}", chromaticity.total[1]);
    Ok(())
}

#[cfg(feature = "cubecl")]
fn radiation(args: ChromaticityArgs) -> Result<()> {
    let lattice = ufo::load_mad_file(&args.path)?;
    let line_name = select_line_name(&lattice, args.line)
        .ok_or_else(|| ufo::UfoError::UnknownReference("no line found".to_string()))?;
    let line = lattice.line(&line_name)?;
    let mut flags = args
        .flags
        .into_iter()
        .fold(PassFlags::empty(), |acc, flag| acc | flag.into());
    if flags.is_empty() {
        flags = PassFlags::LINEAR | PassFlags::ACHROMATIC;
    }
    let optics = ufo::Optics::periodic(
        &lattice,
        line,
        ufo::OpticsOptions {
            where_: ufo::radiation_observations(&lattice, line)?,
            flags,
            is_64bit: args.double,
            run_options: run_options(args.backend, args.device, 1)?,
        },
    )?;
    let radiation = ufo::emittance(&lattice, line, &optics)?;
    let beam = ufo::Beam::default();

    println!("line: {line_name}");
    println!("i1: {:.12}", radiation.i1);
    println!("i2: {:.12}", radiation.i2);
    println!("i4: {:.12}", radiation.i4);
    println!("i5: {:.12}", radiation.i5);
    println!("jx: {:.12}", radiation.jx);
    println!("jt: {:.12}", radiation.jt);
    println!("u0: {:.12}", radiation.u0(&beam));
    println!("ex: {:.12}", radiation.ex(&beam));
    Ok(())
}

#[cfg(feature = "cubecl")]
fn closed_orbit(args: ClosedOrbitArgs) -> Result<()> {
    let lattice = ufo::load_mad_file(&args.path)?;
    let line_name = select_line_name(&lattice, args.line)
        .ok_or_else(|| ufo::UfoError::UnknownReference("no line found".to_string()))?;
    let line = lattice.line(&line_name)?;
    let flags = args
        .flags
        .into_iter()
        .fold(PassFlags::empty(), |acc, flag| acc | flag.into());
    let orbit = ufo::closed_orbit(
        &lattice,
        line,
        ufo::ClosedOrbitOptions {
            flags,
            is_64bit: args.double,
            dp: args.dp,
            iterations: args.iterations,
            step: args.step,
            run_options: run_options(args.backend, args.device, 1)?,
        },
    )?;

    println!("line: {line_name}");
    println!("x: {:.12}", orbit.orbit[0]);
    println!("px: {:.12}", orbit.orbit[1]);
    println!("y: {:.12}", orbit.orbit[2]);
    println!("py: {:.12}", orbit.orbit[3]);
    println!("residual: {:.12e}", orbit.residual);
    Ok(())
}

#[cfg(feature = "cubecl")]
fn rdt(args: ChromaticityArgs) -> Result<()> {
    let lattice = ufo::load_mad_file(&args.path)?;
    let line_name = select_line_name(&lattice, args.line)
        .ok_or_else(|| ufo::UfoError::UnknownReference("no line found".to_string()))?;
    let line = lattice.line(&line_name)?;
    let mut flags = args
        .flags
        .into_iter()
        .fold(PassFlags::empty(), |acc, flag| acc | flag.into());
    if flags.is_empty() {
        flags = PassFlags::LINEAR | PassFlags::ACHROMATIC;
    }
    let optics = ufo::Optics::periodic(
        &lattice,
        line,
        ufo::OpticsOptions {
            where_: ufo::rdt_observations(&lattice, line)?,
            flags,
            is_64bit: args.double,
            run_options: run_options(args.backend, args.device, 1)?,
        },
    )?;
    let terms = ufo::rdt(&lattice, line, &optics)?;

    println!("line: {line_name}");
    print_complex("f3000", terms.f3000);
    print_complex("f1200", terms.f1200);
    print_complex("f1020", terms.f1020);
    print_complex("f0120", terms.f0120);
    print_complex("f0111", terms.f0111);
    Ok(())
}

#[cfg(feature = "cubecl")]
fn stable_aperture(args: StableApertureArgs) -> Result<()> {
    if args.x_count == 0 || args.y_count == 0 {
        return Err(ufo::UfoError::Parse(
            "x-count and y-count must be greater than zero".to_string(),
        ));
    }
    let lattice = ufo::load_mad_file(&args.path)?;
    let line_name = select_line_name(&lattice, args.line)
        .ok_or_else(|| ufo::UfoError::UnknownReference("no line found".to_string()))?;
    let line = lattice.line(&line_name)?;
    let flags = args
        .flags
        .into_iter()
        .fold(PassFlags::empty(), |acc, flag| acc | flag.into());
    let xs = linspace(args.x_min, args.x_max, args.x_count);
    let ys = linspace(args.y_min, args.y_max, args.y_count);
    let mut particles = Vec::with_capacity(xs.len() * ys.len());
    for y in &ys {
        for x in &xs {
            particles.push(ufo::Particle {
                x: *x,
                px: args.px,
                y: *y,
                py: args.py,
                z: args.z,
                dp: args.dp,
                ..ufo::Particle::default()
            });
        }
    }
    let result = ufo::stable_aperture(
        &lattice,
        line,
        particles,
        ufo::StableApertureOptions {
            flags,
            turns: args.turns,
            is_64bit: args.double,
            run_options: run_options(args.backend, args.device, args.turns)?,
        },
    )?;

    println!("line: {line_name}");
    println!("x,y,lost_turn");
    for (row, y) in ys.iter().enumerate() {
        for (col, x) in xs.iter().enumerate() {
            let idx = row * xs.len() + col;
            println!("{:.12},{:.12},{}", x, y, result.lost_turns[idx]);
        }
    }
    Ok(())
}

#[cfg(feature = "cubecl")]
fn print_complex(name: &str, value: ufo::Complex) {
    println!("{name}_re: {:.12}", value.re);
    println!("{name}_im: {:.12}", value.im);
    println!("{name}_abs: {:.12}", value.abs());
}

#[cfg(feature = "cubecl")]
fn linspace(min: f64, max: f64, count: usize) -> Vec<f64> {
    if count == 1 {
        return vec![min];
    }
    let step = (max - min) / (count - 1) as f64;
    (0..count).map(|idx| min + idx as f64 * step).collect()
}

fn select_line_name(lattice: &ufo::Lattice, requested: Option<String>) -> Option<String> {
    requested.or_else(|| {
        lattice
            .lines
            .contains_key("RING")
            .then(|| "RING".to_string())
            .or_else(|| lattice.lines.keys().next().cloned())
    })
}

impl From<FlagArg> for PassFlags {
    fn from(value: FlagArg) -> Self {
        match value {
            FlagArg::Linear => PassFlags::LINEAR,
            FlagArg::Fived => PassFlags::FIVED,
            FlagArg::Exact => PassFlags::EXACT,
            FlagArg::Kick => PassFlags::KICK,
            FlagArg::Radiation => PassFlags::RADIATION,
            FlagArg::DoublePrecision => PassFlags::DOUBLE_PRECISION,
            FlagArg::Achromatic => PassFlags::ACHROMATIC,
        }
    }
}

#[cfg(all(test, feature = "cubecl"))]
mod tests {
    use super::*;

    fn track_args() -> TrackArgs {
        TrackArgs {
            path: PathBuf::from("optics/fodo.mad"),
            line: None,
            turns: 1,
            particles: 1,
            particle: Vec::new(),
            particles_file: None,
            random: false,
            seed: None,
            x_std: 0.0,
            px_std: 0.0,
            y_std: 0.0,
            py_std: 0.0,
            z_std: 0.0,
            dp_std: 0.0,
            grid: Vec::new(),
            where_: vec![-1.0],
            x: 1.0,
            px: 2.0,
            y: 3.0,
            py: 4.0,
            z: 5.0,
            dp: 6.0,
            double: false,
            flags: Vec::new(),
            collapse_linear: false,
            backend: BackendArg::Cpu,
            device: None,
        }
    }

    #[test]
    fn inline_particles_override_base_coordinates() {
        let mut args = track_args();
        args.particle = vec!["0.1,0.2,0.3,0.4,0.5,0.6".to_string(), "7,8".to_string()];

        let particles = build_track_particles(&args).unwrap();

        assert_eq!(particles.len(), 2);
        assert_eq!(particles[0].x, 0.1);
        assert_eq!(particles[0].dp, 0.6);
        assert_eq!(particles[1].x, 7.0);
        assert_eq!(particles[1].px, 8.0);
        assert_eq!(particles[1].y, args.y);
    }

    #[test]
    fn csv_particles_support_named_columns_and_base_defaults() {
        let path = std::env::temp_dir().join(format!("ufo-particles-{}.csv", std::process::id()));
        std::fs::write(&path, "x,px,dp\n0.1,0.2,0.3\n0.4,0.5,0.6\n").unwrap();
        let mut args = track_args();
        args.particles_file = Some(path.clone());

        let particles = build_track_particles(&args).unwrap();

        assert_eq!(particles.len(), 2);
        assert_eq!(particles[0].x, 0.1);
        assert_eq!(particles[0].px, 0.2);
        assert_eq!(particles[0].y, args.y);
        assert_eq!(particles[1].dp, 0.6);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn random_particles_are_seeded_and_centered_on_base() {
        let mut args = track_args();
        args.random = true;
        args.seed = Some(7);
        args.particles = 3;
        args.x_std = 1.0e-3;

        let first = build_track_particles(&args).unwrap();
        let second = build_track_particles(&args).unwrap();

        assert_eq!(first, second);
        assert_eq!(first.len(), 3);
        assert_ne!(first[0].x, args.x);
        assert_eq!(first[0].px, args.px);
    }

    #[test]
    fn grid_particles_build_cartesian_product() {
        let mut args = track_args();
        args.grid = vec!["x=0:1:2".to_string(), "y=-1:1:3".to_string()];

        let particles = build_track_particles(&args).unwrap();

        assert_eq!(particles.len(), 6);
        assert_eq!(
            particles
                .iter()
                .map(|particle| particle.x)
                .collect::<Vec<_>>(),
            vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0]
        );
        assert_eq!(
            particles
                .iter()
                .map(|particle| particle.y)
                .collect::<Vec<_>>(),
            vec![-1.0, 0.0, 1.0, -1.0, 0.0, 1.0]
        );
    }

    #[test]
    fn particle_sources_are_mutually_exclusive() {
        let mut args = track_args();
        args.random = true;
        args.particle = vec!["1,2,3,4,5,6".to_string()];

        let error = build_track_particles(&args).unwrap_err().to_string();

        assert!(error.contains("choose only one particle source"));
    }
}
