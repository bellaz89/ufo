use crate::{
    Beam, Element, Lattice, Line, Optics, QUANTUM_RADIATION_CONSTANT, RADIATION_LOSS_CONSTANT,
    Result, UfoError,
};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RadiationIntegrals {
    pub i1: f64,
    pub i2: f64,
    pub i4: f64,
    pub i5: f64,
    pub jx: f64,
    pub jt: f64,
}

impl RadiationIntegrals {
    pub fn new(i1: f64, i2: f64, i4: f64, i5: f64) -> Self {
        Self {
            i1,
            i2,
            i4,
            i5,
            jx: 1.0 - i4 / i2,
            jt: 1.0 + i4 / i2,
        }
    }

    pub fn u0(&self, beam: &Beam) -> f64 {
        RADIATION_LOSS_CONSTANT * (beam.energy * 1.0e-9).powi(4) * self.i2
    }

    pub fn ex(&self, beam: &Beam) -> f64 {
        let k_emitt = QUANTUM_RADIATION_CONSTANT * beam.gamma * beam.gamma;
        k_emitt * self.i5 / self.i2 / self.jx
    }
}

pub fn emittance(lattice: &Lattice, line: &Line, optics: &Optics) -> Result<RadiationIntegrals> {
    let flat = line.flatten(lattice)?;
    let mut i1 = 0.0;
    let mut i2 = 0.0;
    let mut i4 = 0.0;
    let mut i5 = 0.0;

    for (where_, point) in optics.where_.iter().zip(&optics.points) {
        if *where_ == -1.0 {
            continue;
        }
        if where_.fract() != 0.0 || *where_ < 0.0 {
            return Err(UfoError::UnsupportedObservation(where_.to_string()));
        }
        let element = flat
            .get(*where_ as usize)
            .ok_or_else(|| UfoError::UnsupportedObservation(where_.to_string()))?;
        let angle = element.angle();
        if angle == 0.0 {
            continue;
        }

        let length = element.length();
        let k = element_k1(element);
        let rho = length / angle;
        i2 += length / rho.powi(2);

        if k == 0.0 {
            continue;
        }

        let k2 = k.abs().sqrt();
        let k2l = length * k2;
        let gamma = (1.0 + point.ax * point.ax) / point.bx;
        let (c, s) = if k < 0.0 {
            (k2l.cosh(), k2l.sinh())
        } else {
            (k2l.cos(), k2l.sin())
        };

        let id =
            point.dx * s / k2l + point.dpx * (1.0 - c) / (length * k) + (k2l - s) / (rho * k2l * k);
        i1 += id * length / rho;
        i4 += id * (1.0 / rho.powi(2) + 2.0 * k) * length / rho;

        let h = gamma * point.dx.powi(2)
            + 2.0 * point.ax * point.dx * point.dpx
            + point.dpx.powi(2) * point.bx
            + 2.0
                * angle
                * (-(gamma * point.dx + point.ax * point.dpx) * (k2l - s)
                    / (k2 * k * length.powi(2))
                    + (point.ax * point.dx + point.bx * point.dpx) * (1.0 - c)
                        / (k * length.powi(2)))
            + angle.powi(2)
                * (gamma * (3.0 * k2l - 4.0 * s + s * c) / (2.0 * k2 * k.powi(2) * length.powi(3))
                    - point.ax * (1.0 - c).powi(2) / (k.powi(2) * length.powi(3))
                    + point.bx * (k2l - c * s) / (2.0 * k2l * k * length.powi(2)));

        i5 += h * length / rho.abs().powi(3);
    }

    Ok(RadiationIntegrals::new(i1, i2, i4, i5))
}

pub fn radiation_observations(lattice: &Lattice, line: &Line) -> Result<Vec<f64>> {
    Ok(line
        .flatten(lattice)?
        .into_iter()
        .enumerate()
        .filter_map(|(idx, element)| (element.angle() != 0.0).then_some(idx as f64))
        .collect())
}

fn element_k1(element: &Element) -> f64 {
    match element {
        Element::Sbend(value) => value.k1,
        Element::Rbend(value) => value.k1,
        _ => 0.0,
    }
}
