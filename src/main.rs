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
    /// List available OpenCL devices.
    #[command(alias = "list_devices")]
    ListDevices {
        /// Return only one device id, matching the old list_devices(device) API.
        device: Option<usize>,
    },
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
    /// Build the bundled interpreter kernel for the first OpenCL device.
    #[cfg(feature = "opencl")]
    BuildInterpreter {
        /// Extra OpenCL build options.
        #[arg(long, default_value = "-cl-fast-relaxed-math -cl-mad-enable")]
        options: String,
    },
    /// Track particles through a lattice line with the interpreter backend.
    #[cfg(feature = "opencl")]
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
        /// Emit 64-bit bytecode and double-precision OpenCL kernel.
        #[arg(long)]
        double: bool,
        /// Local interpreter instruction cache size in words.
        #[arg(long)]
        local_instruction_words: Option<usize>,
        /// Pass flags, repeatable: --flag linear --flag achromatic.
        #[arg(long = "flag", value_enum)]
        flags: Vec<FlagArg>,
        /// Collapse consecutive affine linear transforms into OP_TRAN_LINEAR.
        #[arg(long)]
        collapse_linear: bool,
    },
    /// Compute periodic optics functions for a lattice line.
    #[cfg(feature = "opencl")]
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
        /// Emit 64-bit bytecode and double-precision OpenCL kernel.
        #[arg(long)]
        double: bool,
        /// Local interpreter instruction cache size in words.
        #[arg(long)]
        local_instruction_words: Option<usize>,
        /// Pass flags, repeatable: --flag linear --flag achromatic.
        #[arg(long = "flag", value_enum)]
        flags: Vec<FlagArg>,
    },
    /// Compute natural and sextupole-corrected chromaticity.
    #[cfg(feature = "opencl")]
    Chromaticity {
        /// MAD lattice file path.
        path: PathBuf,
        /// Line to analyze. Defaults to RING, or the first parsed line.
        #[arg(short, long)]
        line: Option<String>,
        /// Emit 64-bit bytecode and double-precision OpenCL kernel.
        #[arg(long)]
        double: bool,
        /// Local interpreter instruction cache size in words.
        #[arg(long)]
        local_instruction_words: Option<usize>,
        /// Pass flags, repeatable: --flag linear --flag achromatic.
        #[arg(long = "flag", value_enum)]
        flags: Vec<FlagArg>,
    },
    /// Compute radiation integrals and derived beam quantities.
    #[cfg(feature = "opencl")]
    Radiation {
        /// MAD lattice file path.
        path: PathBuf,
        /// Line to analyze. Defaults to RING, or the first parsed line.
        #[arg(short, long)]
        line: Option<String>,
        /// Emit 64-bit bytecode and double-precision OpenCL kernel.
        #[arg(long)]
        double: bool,
        /// Local interpreter instruction cache size in words.
        #[arg(long)]
        local_instruction_words: Option<usize>,
        /// Pass flags, repeatable: --flag linear --flag achromatic.
        #[arg(long = "flag", value_enum)]
        flags: Vec<FlagArg>,
    },
    /// Find the one-turn closed orbit.
    #[cfg(feature = "opencl")]
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
        /// Emit 64-bit bytecode and double-precision OpenCL kernel.
        #[arg(long)]
        double: bool,
        /// Local interpreter instruction cache size in words.
        #[arg(long)]
        local_instruction_words: Option<usize>,
        /// Pass flags, repeatable: --flag linear --flag achromatic.
        #[arg(long = "flag", value_enum)]
        flags: Vec<FlagArg>,
    },
    /// Compute sextupole resonance driving terms.
    #[cfg(feature = "opencl")]
    Rdt {
        /// MAD lattice file path.
        path: PathBuf,
        /// Line to analyze. Defaults to RING, or the first parsed line.
        #[arg(short, long)]
        line: Option<String>,
        /// Emit 64-bit bytecode and double-precision OpenCL kernel.
        #[arg(long)]
        double: bool,
        /// Local interpreter instruction cache size in words.
        #[arg(long)]
        local_instruction_words: Option<usize>,
        /// Pass flags, repeatable: --flag linear --flag achromatic.
        #[arg(long = "flag", value_enum)]
        flags: Vec<FlagArg>,
    },
    /// Track an x/y grid and report first lost turn per particle.
    #[cfg(feature = "opencl")]
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
        /// Emit 64-bit bytecode and double-precision OpenCL kernel.
        #[arg(long)]
        double: bool,
        /// Local interpreter instruction cache size in words.
        #[arg(long)]
        local_instruction_words: Option<usize>,
        /// Pass flags, repeatable: --flag linear --flag achromatic.
        #[arg(long = "flag", value_enum)]
        flags: Vec<FlagArg>,
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
        Command::ListDevices { device } => list_devices(device),
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
        #[cfg(feature = "opencl")]
        Command::BuildInterpreter { options } => {
            ufo::opencl::build_interpreter_for_first_device(&options)?;
            println!("interpreter.cl build ok");
            Ok(())
        }
        #[cfg(feature = "opencl")]
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
            local_instruction_words,
            flags,
            collapse_linear,
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
            local_instruction_words,
            flags,
            collapse_linear,
        }),
        #[cfg(feature = "opencl")]
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
            local_instruction_words,
            flags,
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
            local_instruction_words,
            flags,
        }),
        #[cfg(feature = "opencl")]
        Command::Chromaticity {
            path,
            line,
            double,
            local_instruction_words,
            flags,
        } => chromaticity(ChromaticityArgs {
            path,
            line,
            double,
            local_instruction_words,
            flags,
        }),
        #[cfg(feature = "opencl")]
        Command::Radiation {
            path,
            line,
            double,
            local_instruction_words,
            flags,
        } => radiation(ChromaticityArgs {
            path,
            line,
            double,
            local_instruction_words,
            flags,
        }),
        #[cfg(feature = "opencl")]
        Command::ClosedOrbit {
            path,
            line,
            dp,
            iterations,
            step,
            double,
            local_instruction_words,
            flags,
        } => closed_orbit(ClosedOrbitArgs {
            path,
            line,
            dp,
            iterations,
            step,
            double,
            local_instruction_words,
            flags,
        }),
        #[cfg(feature = "opencl")]
        Command::Rdt {
            path,
            line,
            double,
            local_instruction_words,
            flags,
        } => rdt(ChromaticityArgs {
            path,
            line,
            double,
            local_instruction_words,
            flags,
        }),
        #[cfg(feature = "opencl")]
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
            local_instruction_words,
            flags,
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
            local_instruction_words,
            flags,
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

#[cfg(feature = "opencl")]
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
    local_instruction_words: Option<usize>,
    flags: Vec<FlagArg>,
    collapse_linear: bool,
}

#[cfg(feature = "opencl")]
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
    local_instruction_words: Option<usize>,
    flags: Vec<FlagArg>,
}

#[cfg(feature = "opencl")]
struct ChromaticityArgs {
    path: PathBuf,
    line: Option<String>,
    double: bool,
    local_instruction_words: Option<usize>,
    flags: Vec<FlagArg>,
}

#[cfg(feature = "opencl")]
struct ClosedOrbitArgs {
    path: PathBuf,
    line: Option<String>,
    dp: f64,
    iterations: usize,
    step: f64,
    double: bool,
    local_instruction_words: Option<usize>,
    flags: Vec<FlagArg>,
}

#[cfg(feature = "opencl")]
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
    local_instruction_words: Option<usize>,
    flags: Vec<FlagArg>,
}

fn list_devices(device: Option<usize>) -> Result<()> {
    #[cfg(feature = "opencl")]
    {
        let devices = ufo::opencl::list_devices()?;
        if let Some(id) = device {
            if let Some(device) = devices.iter().find(|d| d.index == id) {
                println!("{}: {}", device.index, device.name);
            } else {
                return Err(ufo::UfoError::OpenCl(format!("device {id} not found")));
            }
        } else {
            for device in devices {
                println!("{}: {}", device.index, device.name);
            }
        }
        Ok(())
    }
    #[cfg(not(feature = "opencl"))]
    {
        let _ = device;
        Err(ufo::UfoError::Parse(
            "OpenCL support is disabled; rebuild with the `opencl` feature".to_string(),
        ))
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

#[cfg(feature = "opencl")]
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
    track.run_with_options(&ufo::opencl::TrackRunOptions {
        turns: args.turns,
        local_instruction_words: args.local_instruction_words,
        ..ufo::opencl::TrackRunOptions::default()
    })?;

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

#[cfg(feature = "opencl")]
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
        run_options: ufo::opencl::TrackRunOptions {
            local_instruction_words: args.local_instruction_words,
            ..ufo::opencl::TrackRunOptions::default()
        },
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

#[cfg(feature = "opencl")]
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
            run_options: ufo::opencl::TrackRunOptions {
                local_instruction_words: args.local_instruction_words,
                ..ufo::opencl::TrackRunOptions::default()
            },
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

#[cfg(feature = "opencl")]
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
            run_options: ufo::opencl::TrackRunOptions {
                local_instruction_words: args.local_instruction_words,
                ..ufo::opencl::TrackRunOptions::default()
            },
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

#[cfg(feature = "opencl")]
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
            run_options: ufo::opencl::TrackRunOptions {
                local_instruction_words: args.local_instruction_words,
                ..ufo::opencl::TrackRunOptions::default()
            },
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

#[cfg(feature = "opencl")]
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
            run_options: ufo::opencl::TrackRunOptions {
                local_instruction_words: args.local_instruction_words,
                ..ufo::opencl::TrackRunOptions::default()
            },
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

#[cfg(feature = "opencl")]
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
            run_options: ufo::opencl::TrackRunOptions {
                local_instruction_words: args.local_instruction_words,
                ..ufo::opencl::TrackRunOptions::default()
            },
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

#[cfg(feature = "opencl")]
fn print_complex(name: &str, value: ufo::Complex) {
    println!("{name}_re: {:.12}", value.re);
    println!("{name}_im: {:.12}", value.im);
    println!("{name}_abs: {:.12}", value.abs());
}

#[cfg(feature = "opencl")]
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
