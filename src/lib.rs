pub mod beam;
pub mod bytecode;
#[cfg(feature = "cubecl")]
pub mod chroma;
#[cfg(feature = "cubecl")]
pub mod closed_orbit;
pub mod compiler;
pub mod constants;
#[cfg(feature = "cubecl")]
pub mod cubecl;
pub mod element;
pub mod error;
pub mod export;
pub mod flags;
pub mod instruction;
pub mod lattice;
pub mod mad;
#[cfg(feature = "cubecl")]
pub mod optics;
pub mod particle;
#[cfg(feature = "python")]
mod python;
#[cfg(feature = "cubecl")]
pub mod radiation;
#[cfg(feature = "cubecl")]
pub mod rdt;
#[cfg(feature = "cubecl")]
pub mod stable_aperture;
#[cfg(feature = "cubecl")]
pub mod track;

pub use beam::*;
pub use bytecode::Bytecode;
#[cfg(feature = "cubecl")]
pub use chroma::*;
#[cfg(feature = "cubecl")]
pub use closed_orbit::*;
pub use compiler::{TrackCompileOptions, TrackingBytecode, compile_line, compile_tracking_line};
pub use constants::*;
#[cfg(feature = "cubecl")]
pub use cubecl::*;
pub use element::*;
pub use error::{Result, UfoError};
pub use export::{DumpStyle, dump_lattice_file, dump_mad_file, to_lattice_string, to_mad_string};
pub use flags::*;
pub use instruction::*;
pub use lattice::*;
pub use mad::load_mad_file;
#[cfg(feature = "cubecl")]
pub use optics::*;
pub use particle::*;
#[cfg(feature = "cubecl")]
pub use radiation::*;
#[cfg(feature = "cubecl")]
pub use rdt::*;
#[cfg(feature = "cubecl")]
pub use stable_aperture::*;
#[cfg(feature = "cubecl")]
pub use track::*;

pub const DEFAULT_QUADRUPOLE_SLICES: u32 = 8;
pub const DEFAULT_BEND_SLICES: u32 = 8;
pub const DEFAULT_SEXTUPOLE_SLICES: u32 = 4;
pub const DEFAULT_OCTUPOLE_SLICES: u32 = 4;
