use crate::{
    Lattice, Line, Particle, PassFlags, Result, TrackCompileOptions, UfoError,
    compile_tracking_line, opencl,
};

#[derive(Clone, Debug)]
pub struct ClosedOrbitOptions {
    pub flags: PassFlags,
    pub is_64bit: bool,
    pub dp: f64,
    pub iterations: usize,
    pub step: f64,
    pub run_options: opencl::TrackRunOptions,
}

impl Default for ClosedOrbitOptions {
    fn default() -> Self {
        Self {
            flags: PassFlags::empty(),
            is_64bit: false,
            dp: 0.0,
            iterations: 200,
            step: 1.0e-4,
            run_options: opencl::TrackRunOptions::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ClosedOrbit {
    pub orbit: [f64; 4],
    pub residual: f64,
}

pub fn closed_orbit(
    lattice: &Lattice,
    line: &Line,
    options: ClosedOrbitOptions,
) -> Result<ClosedOrbit> {
    let tracking = compile_tracking_line(
        lattice,
        line,
        &TrackCompileOptions {
            flags: options.flags,
            turns: 1,
            is_64bit: options.is_64bit || options.flags.contains(PassFlags::DOUBLE_PRECISION),
            where_: vec![-1.0],
            collapse_linear: false,
        },
    )?;
    let mut run_options = options.run_options;
    run_options.turns = 1;

    let mut simplex = [
        [0.0, 0.0, 0.0, 0.0],
        [options.step, 0.0, 0.0, 0.0],
        [0.0, options.step, 0.0, 0.0],
        [0.0, 0.0, options.step, 0.0],
        [0.0, 0.0, 0.0, options.step],
    ];
    let mut values = residuals(&tracking, &run_options, &simplex, options.dp)?;

    for _ in 0..options.iterations {
        let (lo, hi, next_hi) = order(&values);
        if values[hi] <= 1.0e-24 {
            break;
        }

        let centroid = centroid(&simplex, hi);
        let reflected = transform(centroid, simplex[hi], -1.0);
        let reflected_value = residuals(&tracking, &run_options, &[reflected], options.dp)?[0];

        if reflected_value < values[lo] {
            let expanded = transform(centroid, simplex[hi], 2.0);
            let expanded_value = residuals(&tracking, &run_options, &[expanded], options.dp)?[0];
            if expanded_value < reflected_value {
                simplex[hi] = expanded;
                values[hi] = expanded_value;
            } else {
                simplex[hi] = reflected;
                values[hi] = reflected_value;
            }
        } else if reflected_value < values[next_hi] {
            simplex[hi] = reflected;
            values[hi] = reflected_value;
        } else {
            let contracted = transform(centroid, simplex[hi], 0.5);
            let contracted_value =
                residuals(&tracking, &run_options, &[contracted], options.dp)?[0];
            if contracted_value < values[hi] {
                simplex[hi] = contracted;
                values[hi] = contracted_value;
            } else {
                let best = simplex[lo];
                for idx in 0..simplex.len() {
                    if idx != lo {
                        simplex[idx] = midpoint(best, simplex[idx]);
                    }
                }
                values = residuals(&tracking, &run_options, &simplex, options.dp)?;
            }
        }
    }

    let (lo, _, _) = order(&values);
    Ok(ClosedOrbit {
        orbit: simplex[lo],
        residual: values[lo],
    })
}

fn residuals(
    tracking: &crate::TrackingBytecode,
    options: &opencl::TrackRunOptions,
    points: &[[f64; 4]],
    dp: f64,
) -> Result<Vec<f64>> {
    let particles = points
        .iter()
        .map(|point| Particle {
            x: point[0],
            px: point[1],
            y: point[2],
            py: point[3],
            dp,
            ..Particle::default()
        })
        .collect::<Vec<_>>();
    let output = opencl::track_first_device(tracking, &particles, options)?;
    if output.len() != points.len() {
        return Err(UfoError::Parse(
            "closed-orbit tracking output size mismatch".to_string(),
        ));
    }
    Ok(points
        .iter()
        .zip(output)
        .map(|(initial, final_particle)| {
            (initial[0] - final_particle.x).powi(2)
                + (initial[1] - final_particle.px).powi(2)
                + (initial[2] - final_particle.y).powi(2)
                + (initial[3] - final_particle.py).powi(2)
        })
        .collect())
}

fn order(values: &[f64]) -> (usize, usize, usize) {
    let mut indexes = [0, 1, 2, 3, 4];
    indexes.sort_by(|lhs, rhs| values[*lhs].total_cmp(&values[*rhs]));
    (indexes[0], indexes[4], indexes[3])
}

fn centroid(simplex: &[[f64; 4]; 5], skip: usize) -> [f64; 4] {
    let mut out = [0.0; 4];
    for (idx, point) in simplex.iter().enumerate() {
        if idx == skip {
            continue;
        }
        for dim in 0..4 {
            out[dim] += point[dim] / 4.0;
        }
    }
    out
}

fn transform(centroid: [f64; 4], point: [f64; 4], factor: f64) -> [f64; 4] {
    let mut out = [0.0; 4];
    for dim in 0..4 {
        out[dim] = centroid[dim] + factor * (centroid[dim] - point[dim]);
    }
    out
}

fn midpoint(lhs: [f64; 4], rhs: [f64; 4]) -> [f64; 4] {
    [
        0.5 * (lhs[0] + rhs[0]),
        0.5 * (lhs[1] + rhs[1]),
        0.5 * (lhs[2] + rhs[2]),
        0.5 * (lhs[3] + rhs[3]),
    ]
}
