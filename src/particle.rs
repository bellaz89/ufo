use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
/// Interpreter ABI particle for the default single-precision path.
pub struct Particle32 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub px: f32,
    pub py: f32,
    pub dp: f32,
    pub passed_elements: u32,
    pub alive: u32,
}

impl Default for Particle32 {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            px: 0.0,
            py: 0.0,
            dp: 0.0,
            passed_elements: 0,
            alive: 1,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
/// Interpreter ABI particle for double-precision runs.
pub struct Particle64 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub px: f64,
    pub py: f64,
    pub dp: f64,
    pub passed_elements: u32,
    pub alive: u32,
}

impl Default for Particle64 {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            px: 0.0,
            py: 0.0,
            dp: 0.0,
            passed_elements: 0,
            alive: 1,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
/// Host-side particle state. Rust-side calculations keep coordinates in f64.
pub struct Particle {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub px: f64,
    pub py: f64,
    pub dp: f64,
    pub passed_elements: u32,
    pub alive: bool,
}

impl Default for Particle {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            px: 0.0,
            py: 0.0,
            dp: 0.0,
            passed_elements: 0,
            alive: true,
        }
    }
}

impl From<Particle> for Particle32 {
    fn from(value: Particle) -> Self {
        Self {
            x: value.x as f32,
            y: value.y as f32,
            z: value.z as f32,
            px: value.px as f32,
            py: value.py as f32,
            dp: value.dp as f32,
            passed_elements: value.passed_elements,
            alive: value.alive as u32,
        }
    }
}

impl From<Particle32> for Particle {
    fn from(value: Particle32) -> Self {
        Self {
            x: value.x as f64,
            y: value.y as f64,
            z: value.z as f64,
            px: value.px as f64,
            py: value.py as f64,
            dp: value.dp as f64,
            passed_elements: value.passed_elements,
            alive: value.alive != 0,
        }
    }
}

impl From<Particle> for Particle64 {
    fn from(value: Particle) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
            px: value.px,
            py: value.py,
            dp: value.dp,
            passed_elements: value.passed_elements,
            alive: value.alive as u32,
        }
    }
}

impl From<Particle64> for Particle {
    fn from(value: Particle64) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
            px: value.px,
            py: value.py,
            dp: value.dp,
            passed_elements: value.passed_elements,
            alive: value.alive != 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn particle32_matches_interpreter_layout() {
        assert_eq!(std::mem::size_of::<Particle32>(), 32);
        let particle = Particle32 {
            px: 0.25,
            alive: 1,
            ..Particle32::default()
        };
        let bytes = bytemuck::bytes_of(&particle);
        assert_eq!(f32::from_le_bytes(bytes[12..16].try_into().unwrap()), 0.25);
        assert_eq!(u32::from_le_bytes(bytes[28..32].try_into().unwrap()), 1);
    }

    #[test]
    fn particle64_matches_interpreter_layout() {
        assert_eq!(std::mem::size_of::<Particle64>(), 56);
        let particle = Particle64 {
            px: 0.25,
            alive: 1,
            ..Particle64::default()
        };
        let bytes = bytemuck::bytes_of(&particle);
        assert_eq!(f64::from_le_bytes(bytes[24..32].try_into().unwrap()), 0.25);
        assert_eq!(u32::from_le_bytes(bytes[52..56].try_into().unwrap()), 1);
    }

    #[test]
    fn host_particle_keeps_f64_until_interpreter_abi_conversion() {
        let particle = Particle {
            x: 1.0 / 3.0,
            px: 1.0 / 7.0,
            ..Particle::default()
        };

        let single = Particle32::from(particle);
        let double = Particle64::from(particle);

        assert_eq!(double.x, particle.x);
        assert_eq!(double.px, particle.px);
        assert_eq!(single.x, particle.x as f32);
        assert_ne!(Particle::from(single).x, particle.x);
    }
}
