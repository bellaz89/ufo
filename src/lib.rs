pub mod beam;
pub mod bytecode;
#[cfg(feature = "opencl")]
pub mod chroma;
#[cfg(feature = "opencl")]
pub mod closed_orbit;
pub mod compiler;
pub mod constants;
pub mod element;
pub mod error;
pub mod export;
pub mod flags;
pub mod instruction;
pub mod lattice;
pub mod mad;
#[cfg(feature = "opencl")]
pub mod opencl;
#[cfg(feature = "opencl")]
pub mod optics;
pub mod particle;
#[cfg(feature = "opencl")]
pub mod radiation;
#[cfg(feature = "opencl")]
pub mod rdt;
#[cfg(feature = "opencl")]
pub mod stable_aperture;
#[cfg(feature = "opencl")]
pub mod track;

pub use beam::*;
pub use bytecode::Bytecode;
#[cfg(feature = "opencl")]
pub use chroma::*;
#[cfg(feature = "opencl")]
pub use closed_orbit::*;
pub use compiler::{TrackCompileOptions, TrackingBytecode, compile_line, compile_tracking_line};
pub use constants::*;
pub use element::*;
pub use error::{Result, UfoError};
pub use export::{DumpStyle, dump_lattice_file, dump_mad_file, to_lattice_string, to_mad_string};
pub use flags::*;
pub use instruction::*;
pub use lattice::*;
pub use mad::load_mad_file;
#[cfg(feature = "opencl")]
pub use optics::*;
pub use particle::*;
#[cfg(feature = "opencl")]
pub use radiation::*;
#[cfg(feature = "opencl")]
pub use rdt::*;
#[cfg(feature = "opencl")]
pub use stable_aperture::*;
#[cfg(feature = "opencl")]
pub use track::*;

pub const DEFAULT_QUADRUPOLE_SLICES: u32 = 8;
pub const DEFAULT_BEND_SLICES: u32 = 8;
pub const DEFAULT_SEXTUPOLE_SLICES: u32 = 4;
pub const DEFAULT_OCTUPOLE_SLICES: u32 = 4;
