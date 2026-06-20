use std::{path::PathBuf, process::ExitCode};

use clap::{Parser, Subcommand, ValueEnum};
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
        /// Number of particles with identical initial coordinates.
        #[arg(long, default_value_t = 1)]
        particles: usize,
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
            collapse_linear,
        } => compile(path, line, double, flags, hex, collapse_linear),
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
        println!("{}", device.selector);
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

#[cfg(feature = "cubecl")]
fn track(args: TrackArgs) -> Result<()> {
    let lattice = ufo::load_mad_file(&args.path)?;
    let line_name = select_line_name(&lattice, args.line)
        .ok_or_else(|| ufo::UfoError::UnknownReference("no line found".to_string()))?;
    let line = lattice.line(&line_name)?;
    let flags = args
        .flags
        .into_iter()
        .fold(PassFlags::empty(), |acc, flag| acc | flag.into());
    let initial = ufo::Particle {
        x: args.x,
        px: args.px,
        y: args.y,
        py: args.py,
        z: args.z,
        dp: args.dp,
        ..ufo::Particle::default()
    };
    let particles = vec![initial; args.particles];
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
    println!("particles: {}", args.particles);
    println!("samples: {}", track.samples.len());
    println!("sample,particle,x,px,y,py,z,dp,passed_elements,alive");
    for (sample, chunk) in track.samples.chunks(args.particles).enumerate() {
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
