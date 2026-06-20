#[cfg(feature = "cubecl")]
use std::{env, path::PathBuf, time::Instant};

#[cfg(not(feature = "cubecl"))]
use gufo::Result;
#[cfg(feature = "cubecl")]
use gufo::{Lattice, Particle, PassFlags, Result};

#[cfg(feature = "cubecl")]
#[derive(Clone, Debug)]
struct Args {
    path: PathBuf,
    line: Option<String>,
    turns: u32,
    particles: usize,
    runs: usize,
    warmups: usize,
    collapse_linear: bool,
    flags: PassFlags,
}

#[cfg(feature = "cubecl")]
impl Default for Args {
    fn default() -> Self {
        Self {
            path: PathBuf::from("optics/fodo.mad"),
            line: None,
            turns: 100,
            particles: 1024,
            runs: 5,
            warmups: 1,
            collapse_linear: false,
            flags: PassFlags::empty(),
        }
    }
}

#[cfg(feature = "cubecl")]
fn main() -> Result<()> {
    let args = parse_args()?;
    println!("implementation,phase,run,ms,particles,turns,samples");

    for run in 0..args.warmups {
        run_once(&args, "warmup", run)?;
    }
    for run in 0..args.runs {
        run_once(&args, "measure", run)?;
    }
    Ok(())
}

#[cfg(not(feature = "cubecl"))]
fn main() -> Result<()> {
    Err(gufo::UfoError::Parse(
        "bench_track requires the `cubecl` feature".to_string(),
    ))
}

#[cfg(feature = "cubecl")]
fn run_once(args: &Args, phase: &str, run: usize) -> Result<()> {
    let start = Instant::now();
    let lattice = gufo::load_mad_file(&args.path)?;
    let line_name = select_line_name(&lattice, args.line.as_deref())
        .ok_or_else(|| gufo::UfoError::UnknownReference("no line found".to_string()))?;
    let line = lattice.line(&line_name)?;
    let tracking = gufo::compile_tracking_line(
        &lattice,
        line,
        &gufo::TrackCompileOptions {
            flags: args.flags,
            turns: args.turns,
            is_64bit: args.flags.contains(PassFlags::DOUBLE_PRECISION),
            where_: vec![-1.0],
            collapse_linear: args.collapse_linear,
        },
    )?;
    let compile_ms = start.elapsed().as_secs_f64() * 1.0e3;
    println!(
        "rust,compile_{phase},{run},{compile_ms:.6},{},{},{}",
        args.particles, args.turns, tracking.dumps_per_turn
    );

    let particles = vec![
        Particle {
            x: 0.001,
            ..Particle::default()
        };
        args.particles
    ];
    let start = Instant::now();
    let output = gufo::cubecl::track_with_options(
        &tracking,
        &particles,
        &gufo::cubecl::CubeClTrackRunOptions {
            turns: args.turns,
            ..gufo::cubecl::CubeClTrackRunOptions::default()
        },
    )?;
    let track_ms = start.elapsed().as_secs_f64() * 1.0e3;
    println!(
        "rust,track_{phase},{run},{track_ms:.6},{},{},{}",
        args.particles,
        args.turns,
        output.len()
    );
    Ok(())
}

#[cfg(feature = "cubecl")]
fn parse_args() -> Result<Args> {
    let mut args = Args::default();
    let mut iter = env::args().skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--path" => args.path = PathBuf::from(value(&mut iter, "--path")?),
            "--line" => args.line = Some(value(&mut iter, "--line")?),
            "--turns" => args.turns = parse_value(&mut iter, "--turns")?,
            "--particles" => args.particles = parse_value(&mut iter, "--particles")?,
            "--runs" => args.runs = parse_value(&mut iter, "--runs")?,
            "--warmups" => args.warmups = parse_value(&mut iter, "--warmups")?,
            "--collapse-linear" => args.collapse_linear = true,
            "--flag" => args.flags |= parse_flag(&value(&mut iter, "--flag")?)?,
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            other => {
                return Err(gufo::UfoError::Parse(format!(
                    "unknown benchmark argument `{other}`"
                )));
            }
        }
    }
    Ok(args)
}

#[cfg(feature = "cubecl")]
fn value(iter: &mut impl Iterator<Item = String>, name: &str) -> Result<String> {
    iter.next()
        .ok_or_else(|| gufo::UfoError::Parse(format!("missing value for `{name}`")))
}

#[cfg(feature = "cubecl")]
fn parse_value<T>(iter: &mut impl Iterator<Item = String>, name: &str) -> Result<T>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    value(iter, name)?
        .parse::<T>()
        .map_err(|error| gufo::UfoError::Parse(format!("invalid value for `{name}`: {error}")))
}

#[cfg(feature = "cubecl")]
fn parse_flag(value: &str) -> Result<PassFlags> {
    match value.to_ascii_lowercase().as_str() {
        "linear" => Ok(PassFlags::LINEAR),
        "fived" => Ok(PassFlags::FIVED),
        "exact" => Ok(PassFlags::EXACT),
        "kick" => Ok(PassFlags::KICK),
        "radiation" => Ok(PassFlags::RADIATION),
        "double" | "double-precision" => Ok(PassFlags::DOUBLE_PRECISION),
        "achromatic" => Ok(PassFlags::ACHROMATIC),
        other => Err(gufo::UfoError::Parse(format!(
            "unknown benchmark flag `{other}`"
        ))),
    }
}

#[cfg(feature = "cubecl")]
fn select_line_name(lattice: &Lattice, requested: Option<&str>) -> Option<String> {
    if let Some(requested) = requested {
        return Some(requested.to_string());
    }
    if lattice.lines.contains_key("RING") {
        return Some("RING".to_string());
    }
    lattice.lines.keys().next().cloned()
}

#[cfg(feature = "cubecl")]
fn print_help() {
    println!(
        "Usage: cargo run --release --example bench_track -- [options]\n\
\n\
Options:\n\
  --path <mad>          MAD file path [default: optics/fodo.mad]\n\
  --line <name>         Line name [default: RING or first line]\n\
  --turns <n>           Turns per tracking run [default: 100]\n\
  --particles <n>       Particles per tracking run [default: 1024]\n\
  --runs <n>            Measured runs [default: 5]\n\
  --warmups <n>         Warmup runs [default: 1]\n\
  --collapse-linear     Collapse affine linear bytecode\n\
  --flag <name>         Repeatable pass flag: linear, fived, exact, kick,\n\
                        radiation, double-precision, achromatic"
    );
}
