use crate::{Element, Lattice, Line, Optics, Result, UfoError};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Complex {
    pub re: f64,
    pub im: f64,
}

impl Complex {
    pub fn exp_i(phase: f64) -> Self {
        Self {
            re: phase.cos(),
            im: phase.sin(),
        }
    }

    pub fn scale(self, value: f64) -> Self {
        Self {
            re: self.re * value,
            im: self.im * value,
        }
    }

    pub fn abs(self) -> f64 {
        self.re.hypot(self.im)
    }
}

impl std::ops::AddAssign for Complex {
    fn add_assign(&mut self, rhs: Self) {
        self.re += rhs.re;
        self.im += rhs.im;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ResonanceDrivingTerms {
    pub f3000: Complex,
    pub f1200: Complex,
    pub f1020: Complex,
    pub f0120: Complex,
    pub f0111: Complex,
}

pub fn rdt(lattice: &Lattice, line: &Line, optics: &Optics) -> Result<ResonanceDrivingTerms> {
    let flat = line.flatten(lattice)?;
    let mut out = ResonanceDrivingTerms::default();

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
        let length = element.length();
        let k = element_k2(element);
        if length == 0.0 || k == 0.0 {
            continue;
        }

        let klb = k * length * point.bx.sqrt();
        out.f3000 += Complex::exp_i(3.0 * point.mux).scale(klb * point.bx);
        out.f1200 += Complex::exp_i(-point.mux).scale(klb * point.bx);
        out.f1020 += Complex::exp_i(point.mux + 2.0 * point.muy).scale(klb * point.by);
        out.f0120 += Complex::exp_i(-point.mux + 2.0 * point.muy).scale(klb * point.by);
        out.f0111 += Complex::exp_i(-point.mux).scale(klb * point.by);
    }

    Ok(out)
}

pub fn rdt_observations(lattice: &Lattice, line: &Line) -> Result<Vec<f64>> {
    Ok(line
        .flatten(lattice)?
        .into_iter()
        .enumerate()
        .filter_map(|(idx, element)| (element_k2(element) != 0.0).then_some(idx as f64))
        .collect())
}

fn element_k2(element: &Element) -> f64 {
    match element {
        Element::Sextupole(value) => value.k2,
        _ => 0.0,
    }
}
