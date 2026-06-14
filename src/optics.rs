use crate::{
    Lattice, Line, Particle, PassFlags, Result, Track, TrackOptions, UfoError,
    opencl::TrackRunOptions,
};

#[derive(Clone, Debug)]
pub struct OpticsOptions {
    pub where_: Vec<f64>,
    pub flags: PassFlags,
    pub is_64bit: bool,
    pub run_options: TrackRunOptions,
}

impl Default for OpticsOptions {
    fn default() -> Self {
        Self {
            where_: Vec::new(),
            flags: PassFlags::LINEAR | PassFlags::ACHROMATIC,
            is_64bit: false,
            run_options: TrackRunOptions::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InitialOptics {
    pub ax: f64,
    pub ay: f64,
    pub bx: f64,
    pub by: f64,
    pub dx: f64,
    pub dy: f64,
    pub dpx: f64,
    pub dpy: f64,
}

impl Default for InitialOptics {
    fn default() -> Self {
        Self {
            ax: 0.0,
            ay: 0.0,
            bx: 1.0,
            by: 1.0,
            dx: 0.0,
            dy: 0.0,
            dpx: 0.0,
            dpy: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct OpticsPoint {
    pub ax: f64,
    pub ay: f64,
    pub bx: f64,
    pub by: f64,
    pub dx: f64,
    pub dy: f64,
    pub dpx: f64,
    pub dpy: f64,
    pub mux: f64,
    pub muy: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Optics {
    pub where_: Vec<f64>,
    pub qx: f64,
    pub qy: f64,
    pub ax0: f64,
    pub ay0: f64,
    pub bx0: f64,
    pub by0: f64,
    pub dx0: f64,
    pub dy0: f64,
    pub dpx0: f64,
    pub dpy0: f64,
    pub points: Vec<OpticsPoint>,
}

impl Optics {
    pub fn periodic(lattice: &Lattice, line: &Line, options: OpticsOptions) -> Result<Self> {
        let mut where_ = options.where_;
        if !where_.iter().any(|value| *value == -1.0) {
            where_.push(-1.0);
        }
        if where_.is_empty() {
            return Err(UfoError::UnsupportedObservation(
                "no optics observation points".to_string(),
            ));
        }

        let particles = vec![
            Particle {
                x: 1.0,
                y: 1.0,
                ..Particle::default()
            },
            Particle {
                px: 1.0,
                py: 1.0,
                ..Particle::default()
            },
            Particle {
                dp: 1.0,
                ..Particle::default()
            },
        ];
        let mut track = Track::new(
            lattice,
            line,
            particles,
            TrackOptions {
                flags: options.flags,
                turns: 1,
                where_: where_.clone(),
                is_64bit: options.is_64bit,
                collapse_linear: false,
            },
        )?;
        let mut run_options = options.run_options;
        run_options.turns = 1;
        track.run_with_options(&run_options)?;

        let final_dump = where_
            .iter()
            .position(|value| *value == -1.0)
            .ok_or_else(missing_sample)?;
        let re = *track.sample(0, final_dump, 0).ok_or_else(missing_sample)?;
        let im = *track.sample(0, final_dump, 1).ok_or_else(missing_sample)?;
        let dp = *track.sample(0, final_dump, 2).ok_or_else(missing_sample)?;

        let (ax0, bx0, qx, dx0, dpx0) = periodic_plane(re.x, re.px, im.x, im.px, dp.x, dp.px)?;
        let (ay0, by0, qy, dy0, dpy0) = periodic_plane(re.y, re.py, im.y, im.py, dp.y, dp.py)?;

        let points = (0..where_.len())
            .map(|dump| {
                let re = *track.sample(0, dump, 0).ok_or_else(missing_sample)?;
                let im = *track.sample(0, dump, 1).ok_or_else(missing_sample)?;
                let dp = *track.sample(0, dump, 2).ok_or_else(missing_sample)?;
                Ok(OpticsPoint {
                    ax: alpha_at(re.x, re.px, im.x, im.px, ax0, bx0),
                    bx: beta_at(re.x, im.x, ax0, bx0),
                    dx: re.x * dx0 + im.x * dpx0 + dp.x,
                    dpx: re.px * dx0 + im.px * dpx0 + dp.px,
                    mux: phase_at(re.x, im.x, ax0, bx0, beta_at(re.x, im.x, ax0, bx0)),
                    ay: alpha_at(re.y, re.py, im.y, im.py, ay0, by0),
                    by: beta_at(re.y, im.y, ay0, by0),
                    dy: re.y * dy0 + im.y * dpy0 + dp.y,
                    dpy: re.py * dy0 + im.py * dpy0 + dp.py,
                    muy: phase_at(re.y, im.y, ay0, by0, beta_at(re.y, im.y, ay0, by0)),
                })
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(Self {
            where_,
            qx,
            qy,
            ax0,
            ay0,
            bx0,
            by0,
            dx0,
            dy0,
            dpx0,
            dpy0,
            points,
        })
    }

    pub fn propagate(
        lattice: &Lattice,
        line: &Line,
        initial: InitialOptics,
        options: OpticsOptions,
    ) -> Result<Self> {
        if initial.bx <= 0.0 || initial.by <= 0.0 {
            return Err(UfoError::Parse(
                "initial beta functions must be positive".to_string(),
            ));
        }
        let mut where_ = options.where_;
        if where_.is_empty() {
            where_.push(-1.0);
        }

        let particles = vec![
            Particle {
                x: 1.0,
                y: 1.0,
                ..Particle::default()
            },
            Particle {
                px: 1.0,
                py: 1.0,
                ..Particle::default()
            },
            Particle {
                dp: 1.0,
                ..Particle::default()
            },
        ];
        let mut track = Track::new(
            lattice,
            line,
            particles,
            TrackOptions {
                flags: options.flags,
                turns: 1,
                where_: where_.clone(),
                is_64bit: options.is_64bit,
                collapse_linear: false,
            },
        )?;
        let mut run_options = options.run_options;
        run_options.turns = 1;
        track.run_with_options(&run_options)?;

        let points = (0..where_.len())
            .map(|dump| {
                let re = *track.sample(0, dump, 0).ok_or_else(missing_sample)?;
                let im = *track.sample(0, dump, 1).ok_or_else(missing_sample)?;
                let dp = *track.sample(0, dump, 2).ok_or_else(missing_sample)?;
                let bx = beta_at(re.x, im.x, initial.ax, initial.bx);
                let by = beta_at(re.y, im.y, initial.ay, initial.by);
                Ok(OpticsPoint {
                    ax: alpha_at(re.x, re.px, im.x, im.px, initial.ax, initial.bx),
                    bx,
                    dx: re.x * initial.dx + im.x * initial.dpx + dp.x,
                    dpx: re.px * initial.dx + im.px * initial.dpx + dp.px,
                    mux: phase_at(re.x, im.x, initial.ax, initial.bx, bx),
                    ay: alpha_at(re.y, re.py, im.y, im.py, initial.ay, initial.by),
                    by,
                    dy: re.y * initial.dy + im.y * initial.dpy + dp.y,
                    dpy: re.py * initial.dy + im.py * initial.dpy + dp.py,
                    muy: phase_at(re.y, im.y, initial.ay, initial.by, by),
                })
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(Self {
            where_,
            qx: f64::NAN,
            qy: f64::NAN,
            ax0: initial.ax,
            ay0: initial.ay,
            bx0: initial.bx,
            by0: initial.by,
            dx0: initial.dx,
            dy0: initial.dy,
            dpx0: initial.dpx,
            dpy0: initial.dpy,
            points,
        })
    }
}

fn periodic_plane(
    re_pos: f64,
    re_mom: f64,
    im_pos: f64,
    im_mom: f64,
    dp_pos: f64,
    dp_mom: f64,
) -> Result<(f64, f64, f64, f64, f64)> {
    let cos_nu = (re_pos + im_mom) * 0.5;
    if cos_nu.abs() > 1.0 {
        return Err(UfoError::Parse(
            "periodic optics plane is unstable".to_string(),
        ));
    }
    let mut nu = cos_nu.acos();
    let beta = (im_pos / nu.sin()).abs();
    let sin_nu = im_pos / beta;
    nu = sin_nu.atan2(cos_nu);
    if nu < 0.0 {
        nu += 2.0 * std::f64::consts::PI;
    }
    let alpha = (re_pos - im_mom) / (2.0 * nu.sin());
    let tune = nu / (2.0 * std::f64::consts::PI);
    let denom = 2.0 - re_pos - im_mom;
    let dispersion = ((1.0 - im_mom) * dp_pos + im_pos * dp_mom) / denom;
    let dispersion_prime = ((1.0 - re_pos) * dp_mom + re_mom * dp_pos) / denom;
    Ok((alpha, beta, tune, dispersion, dispersion_prime))
}

fn alpha_at(re_pos: f64, re_mom: f64, im_pos: f64, im_mom: f64, alpha0: f64, beta0: f64) -> f64 {
    let gamma0 = (1.0 + alpha0 * alpha0) / beta0;
    -re_pos * re_mom * beta0 + (re_pos * im_mom + im_pos * re_mom) * alpha0
        - im_pos * im_mom * gamma0
}

fn beta_at(re_pos: f64, im_pos: f64, alpha0: f64, beta0: f64) -> f64 {
    let gamma0 = (1.0 + alpha0 * alpha0) / beta0;
    re_pos * re_pos * beta0 - re_pos * im_pos * alpha0 * 2.0 + im_pos * im_pos * gamma0
}

fn phase_at(re_pos: f64, im_pos: f64, alpha0: f64, beta0: f64, beta: f64) -> f64 {
    let sin_mu = im_pos / (beta0 * beta).sqrt();
    let cos_mu = re_pos * (beta0 / beta).sqrt() - alpha0 * sin_mu;
    sin_mu.atan2(cos_mu)
}

fn missing_sample() -> UfoError {
    UfoError::Parse("missing optics tracking sample".to_string())
}
