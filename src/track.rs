use crate::{
    Lattice, Line, Particle, PassFlags, Result, TrackCompileOptions, TrackingBytecode,
    compile_tracking_line, opencl,
};

#[derive(Clone, Debug)]
pub struct TrackOptions {
    pub flags: PassFlags,
    pub turns: u32,
    pub where_: Vec<f64>,
    pub is_64bit: bool,
    pub collapse_linear: bool,
}

impl Default for TrackOptions {
    fn default() -> Self {
        Self {
            flags: PassFlags::empty(),
            turns: 1,
            where_: vec![-1.0],
            is_64bit: false,
            collapse_linear: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Track {
    pub particles: Vec<Particle>,
    pub samples: Vec<Particle>,
    pub dumps_per_turn: usize,
    pub turns: u32,
    bytecode: TrackingBytecode,
}

impl Track {
    pub fn new(
        lattice: &Lattice,
        line: &Line,
        particles: Vec<Particle>,
        options: TrackOptions,
    ) -> Result<Self> {
        let bytecode = compile_tracking_line(
            lattice,
            line,
            &TrackCompileOptions {
                flags: options.flags,
                turns: options.turns,
                is_64bit: options.is_64bit || options.flags.contains(PassFlags::DOUBLE_PRECISION),
                where_: options.where_,
                collapse_linear: options.collapse_linear,
            },
        )?;
        Ok(Self {
            particles,
            samples: Vec::new(),
            dumps_per_turn: bytecode.dumps_per_turn,
            turns: options.turns,
            bytecode,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        self.run_with_options(&opencl::TrackRunOptions {
            turns: self.turns,
            ..opencl::TrackRunOptions::default()
        })
    }

    pub fn run_with_options(&mut self, options: &opencl::TrackRunOptions) -> Result<()> {
        self.samples = opencl::track_first_device(&self.bytecode, &self.particles, options)?;
        Ok(())
    }

    pub fn sample(&self, turn: usize, dump: usize, particle: usize) -> Option<&Particle> {
        let particles = self.particles.len();
        let idx = (turn * self.dumps_per_turn + dump) * particles + particle;
        self.samples.get(idx)
    }
}
