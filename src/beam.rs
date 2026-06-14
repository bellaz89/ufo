use crate::ELECTRON_MASS;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Beam {
    pub energy: f64,
    pub particle_mass: f64,
    pub gamma: f64,
    pub bunch_charge: f64,
    pub beam_current: f64,
    pub ex: f64,
    pub ey: f64,
    pub bunch_length: f64,
    pub energy_spread: f64,
}

impl Beam {
    pub fn new(energy: f64, particle_mass: f64) -> Self {
        Self {
            energy,
            particle_mass,
            gamma: (1.0 + energy) / particle_mass,
            ..Self::default()
        }
    }
}

impl Default for Beam {
    fn default() -> Self {
        let energy = 3.0e9;
        let particle_mass = ELECTRON_MASS;
        Self {
            energy,
            particle_mass,
            gamma: (1.0 + energy) / particle_mass,
            bunch_charge: 1.0e-9,
            beam_current: 0.25,
            ex: 1.0e-9,
            ey: 1.0e-9,
            bunch_length: 6.0e-3,
            energy_spread: 1.0e-3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_beam_matches_python_defaults() {
        let beam = Beam::default();
        assert_eq!(beam.energy, 3.0e9);
        assert_eq!(beam.particle_mass, ELECTRON_MASS);
        assert_eq!(beam.bunch_charge, 1.0e-9);
        assert_eq!(beam.beam_current, 0.25);
        assert_eq!(beam.ex, 1.0e-9);
        assert_eq!(beam.ey, 1.0e-9);
        assert_eq!(beam.bunch_length, 6.0e-3);
        assert_eq!(beam.energy_spread, 1.0e-3);
        assert_eq!(beam.gamma, (1.0 + beam.energy) / beam.particle_mass);
    }

    #[test]
    fn custom_beam_recomputes_gamma() {
        let beam = Beam::new(6.0e9, ELECTRON_MASS);
        assert_eq!(beam.energy, 6.0e9);
        assert_eq!(beam.gamma, (1.0 + 6.0e9) / ELECTRON_MASS);
    }
}
