use crate::{
    DEFAULT_BEND_SLICES, DEFAULT_OCTUPOLE_SLICES, DEFAULT_QUADRUPOLE_SLICES,
    DEFAULT_SEXTUPOLE_SLICES,
};

#[derive(Clone, Debug, PartialEq)]
pub enum Element {
    Marker(Marker),
    Drift(Drift),
    Multipole(Multipole),
    Quadrupole(Quadrupole),
    Sbend(Sbend),
    Rbend(Rbend),
    Sextupole(Sextupole),
    Octupole(Octupole),
    Wire(Wire),
    Cavity(Cavity),
    Aperture(Aperture),
}

impl Element {
    pub fn label(&self) -> &str {
        match self {
            Self::Marker(v) => &v.label,
            Self::Drift(v) => &v.label,
            Self::Multipole(v) => &v.label,
            Self::Quadrupole(v) => &v.label,
            Self::Sbend(v) => &v.label,
            Self::Rbend(v) => &v.label,
            Self::Sextupole(v) => &v.label,
            Self::Octupole(v) => &v.label,
            Self::Wire(v) => &v.label,
            Self::Cavity(v) => &v.label,
            Self::Aperture(v) => &v.label,
        }
    }

    pub fn length(&self) -> f64 {
        match self {
            Self::Drift(v) => v.length,
            Self::Quadrupole(v) => v.length,
            Self::Sbend(v) => v.length,
            Self::Rbend(v) => v.length,
            Self::Sextupole(v) => v.length,
            Self::Octupole(v) => v.length,
            _ => 0.0,
        }
    }

    pub fn angle(&self) -> f64 {
        match self {
            Self::Sbend(v) => v.angle,
            Self::Rbend(v) => v.angle,
            _ => 0.0,
        }
    }

    pub fn survey_step(&self, position: [f64; 2], alpha: f64) -> ([f64; 2], f64) {
        let alpha = alpha + self.angle();
        let position = [
            position[0] + alpha.cos() * self.length(),
            position[1] + alpha.sin() * self.length(),
        ];
        (position, alpha)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Marker {
    pub label: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Drift {
    pub label: String,
    pub length: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Multipole {
    pub label: String,
    pub knl: Vec<f64>,
    pub ksl: Vec<f64>,
    pub dx: f64,
    pub dy: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Quadrupole {
    pub label: String,
    pub slices: u32,
    pub length: f64,
    pub k1: f64,
    pub dx: f64,
    pub dy: f64,
    pub dkn: Vec<f64>,
    pub dks: Vec<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Sbend {
    pub label: String,
    pub slices: u32,
    pub length: f64,
    pub angle: f64,
    pub k1: f64,
    pub e1: f64,
    pub e2: f64,
    pub hgap: f64,
    pub fint: f64,
    pub dx: f64,
    pub dy: f64,
    pub dkn: Vec<f64>,
    pub dks: Vec<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Rbend {
    pub label: String,
    pub slices: u32,
    pub length: f64,
    pub angle: f64,
    pub k1: f64,
    pub e1: f64,
    pub e2: f64,
    pub hgap: f64,
    pub fint: f64,
    pub dx: f64,
    pub dy: f64,
    pub dkn: Vec<f64>,
    pub dks: Vec<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Sextupole {
    pub label: String,
    pub slices: u32,
    pub length: f64,
    pub k2: f64,
    pub k2s: f64,
    pub dx: f64,
    pub dy: f64,
    pub dkn: Vec<f64>,
    pub dks: Vec<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Octupole {
    pub label: String,
    pub slices: u32,
    pub length: f64,
    pub k3: f64,
    pub k3s: f64,
    pub dx: f64,
    pub dy: f64,
    pub dkn: Vec<f64>,
    pub dks: Vec<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Wire {
    pub label: String,
    pub x: f64,
    pub y: f64,
    pub k: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Cavity {
    pub label: String,
    pub field: f64,
    pub omega: f64,
    pub lag: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Aperture {
    pub label: String,
    pub window: String,
    pub radius: Option<f64>,
}

impl Quadrupole {
    pub fn new(label: impl Into<String>, length: f64, k1: f64) -> Self {
        Self {
            label: label.into(),
            slices: DEFAULT_QUADRUPOLE_SLICES,
            length,
            k1,
            dx: 0.0,
            dy: 0.0,
            dkn: Vec::new(),
            dks: Vec::new(),
        }
    }
}

impl Sbend {
    pub fn new(label: impl Into<String>, length: f64, angle: f64, k1: f64) -> Self {
        Self {
            label: label.into(),
            slices: DEFAULT_BEND_SLICES,
            length,
            angle,
            k1,
            e1: 0.0,
            e2: 0.0,
            hgap: 0.0,
            fint: 0.0,
            dx: 0.0,
            dy: 0.0,
            dkn: Vec::new(),
            dks: Vec::new(),
        }
    }
}

impl Sextupole {
    pub fn new(label: impl Into<String>, length: f64, k2: f64, k2s: f64) -> Self {
        Self {
            label: label.into(),
            slices: DEFAULT_SEXTUPOLE_SLICES,
            length,
            k2,
            k2s,
            dx: 0.0,
            dy: 0.0,
            dkn: Vec::new(),
            dks: Vec::new(),
        }
    }
}

impl Octupole {
    pub fn new(label: impl Into<String>, length: f64, k3: f64, k3s: f64) -> Self {
        Self {
            label: label.into(),
            slices: DEFAULT_OCTUPOLE_SLICES,
            length,
            k3,
            k3s,
            dx: 0.0,
            dy: 0.0,
            dkn: Vec::new(),
            dks: Vec::new(),
        }
    }
}
