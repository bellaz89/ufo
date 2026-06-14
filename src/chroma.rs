use crate::{Element, Lattice, Line, Optics, Result, UfoError};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Chromaticity {
    pub natural: [f64; 2],
    pub total: [f64; 2],
}

pub fn chromaticity(lattice: &Lattice, line: &Line, optics: &Optics) -> Result<Chromaticity> {
    let flat = line.flatten(lattice)?;
    let mut natural = [0.0, 0.0];
    let mut sextupole = [0.0, 0.0];

    for (where_, point) in optics.where_.iter().zip(&optics.points) {
        if *where_ == -1.0 {
            continue;
        }
        if where_.fract() != 0.0 || *where_ < 0.0 {
            return Err(UfoError::UnsupportedObservation(where_.to_string()));
        }
        let idx = *where_ as usize;
        let element = flat
            .get(idx)
            .ok_or_else(|| UfoError::UnsupportedObservation(where_.to_string()))?;
        let length = element.length();
        if length == 0.0 {
            continue;
        }

        let k1 = element_k1(element);
        if k1 != 0.0 {
            add_quadrupole_chromaticity(
                &mut natural,
                length,
                k1,
                point.ax,
                point.ay,
                point.bx,
                point.by,
            );
        }

        let k2 = element_k2(element);
        if k2 != 0.0 {
            add_sextupole_chromaticity(
                &mut sextupole,
                length,
                k2,
                point.ax,
                point.ay,
                point.bx,
                point.by,
                point.dx,
                point.dpx,
            );
        }
    }

    for value in natural.iter_mut().chain(sextupole.iter_mut()) {
        *value /= 4.0 * std::f64::consts::PI;
    }

    Ok(Chromaticity {
        natural,
        total: [natural[0] + sextupole[0], natural[1] + sextupole[1]],
    })
}

pub fn chromaticity_observations(lattice: &Lattice, line: &Line) -> Result<Vec<f64>> {
    Ok(line
        .flatten(lattice)?
        .into_iter()
        .enumerate()
        .filter_map(|(idx, element)| {
            (element_k1(element) != 0.0 || element_k2(element) != 0.0).then_some(idx as f64)
        })
        .collect())
}

fn add_quadrupole_chromaticity(
    out: &mut [f64; 2],
    length: f64,
    k: f64,
    ax: f64,
    ay: f64,
    bx: f64,
    by: f64,
) {
    let k2 = k.abs().sqrt();
    let k2l = length * k2;
    let gx = (1.0 + ax * ax) / bx;
    let gy = (1.0 + ay * ay) / by;

    let s = (2.0 * k2l).sin() / 4.0 / k2;
    let sh = (2.0 * k2l).sinh() / 4.0 / k2;
    let c = (k2l.cos().powi(2) - 1.0) / k2 / k2;
    let ch = (k2l.cosh().powi(2) - 1.0) / k2 / k2;

    if k > 0.0 {
        out[0] -= (bx * (length / 2.0 + s) + ax * c + gx * (length / 2.0 - s) / k2 / k2) * k;
        out[1] += (by * (length / 2.0 + sh) - ay * ch - gy * (length / 2.0 - sh) / k2 / k2) * k;
    } else {
        out[1] += (by * (length / 2.0 + s) + ay * c + gy * (length / 2.0 - s) / k2 / k2) * k;
        out[0] -= (bx * (length / 2.0 + sh) - ax * ch - gx * (length / 2.0 - sh) / k2 / k2) * k;
    }
}

#[allow(clippy::too_many_arguments)]
fn add_sextupole_chromaticity(
    out: &mut [f64; 2],
    length: f64,
    k: f64,
    ax: f64,
    ay: f64,
    bx: f64,
    by: f64,
    dx: f64,
    dpx: f64,
) {
    let gx = (1.0 + ax * ax) / bx;
    let gy = (1.0 + ay * ay) / by;

    out[0] += (dx * (bx + (gx * length / 3.0 - ax) * length)
        + dpx * length * (bx / 2.0 - length * (2.0 * ax / 3.0 + gx * length / 4.0)))
        * length
        * k;
    out[1] -= (dx * (by + (gy * length / 3.0 - ay) * length)
        + dpx * length * (by / 2.0 - length * (2.0 * ay / 3.0 + gy * length / 4.0)))
        * length
        * k;
}

fn element_k1(element: &Element) -> f64 {
    match element {
        Element::Quadrupole(value) => value.k1,
        Element::Sbend(value) => value.k1,
        Element::Rbend(value) => value.k1,
        _ => 0.0,
    }
}

fn element_k2(element: &Element) -> f64 {
    match element {
        Element::Sextupole(value) => value.k2,
        _ => 0.0,
    }
}
