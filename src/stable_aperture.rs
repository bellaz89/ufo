use crate::{Lattice, Line, Particle, PassFlags, Result, Track, TrackOptions, opencl};

#[derive(Clone, Debug)]
pub struct StableApertureOptions {
    pub flags: PassFlags,
    pub turns: u32,
    pub is_64bit: bool,
    pub run_options: opencl::TrackRunOptions,
}

impl Default for StableApertureOptions {
    fn default() -> Self {
        Self {
            flags: PassFlags::empty(),
            turns: 1000,
            is_64bit: false,
            run_options: opencl::TrackRunOptions::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct StableAperture {
    pub lost_turns: Vec<u32>,
}

pub fn stable_aperture(
    lattice: &Lattice,
    line: &Line,
    particles: Vec<Particle>,
    options: StableApertureOptions,
) -> Result<StableAperture> {
    let turns = options.turns.max(1);
    let particle_count = particles.len();
    let mut track = Track::new(
        lattice,
        line,
        particles,
        TrackOptions {
            flags: options.flags,
            turns,
            where_: vec![-1.0],
            is_64bit: options.is_64bit,
            collapse_linear: false,
        },
    )?;
    let mut run_options = options.run_options;
    run_options.turns = turns;
    track.run_with_options(&run_options)?;

    let mut lost_turns = vec![turns; particle_count];
    for turn in 0..turns as usize {
        for particle in 0..particle_count {
            let sample = track
                .sample(turn, 0, particle)
                .expect("sample shape is validated by Track");
            if lost_turns[particle] == turns && is_lost(sample) {
                lost_turns[particle] = turn as u32;
            }
        }
    }

    Ok(StableAperture { lost_turns })
}

fn is_lost(particle: &Particle) -> bool {
    !particle.alive
        || !particle.x.is_finite()
        || !particle.y.is_finite()
        || particle.x.abs() > 1.0
        || particle.y.abs() > 1.0
}
