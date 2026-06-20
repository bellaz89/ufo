use std::{path::PathBuf, sync::Arc};

use pyo3::{
    exceptions::{PyRuntimeError, PyValueError},
    prelude::*,
    types::PyModule,
};

fn py_error(error: crate::UfoError) -> PyErr {
    PyRuntimeError::new_err(error.to_string())
}

#[pyclass(name = "Lattice", module = "gufo")]
struct PyLattice {
    lattice: Arc<crate::Lattice>,
}

#[pymethods]
impl PyLattice {
    #[new]
    #[pyo3(signature = (path=None))]
    fn new(path: Option<PathBuf>) -> PyResult<Self> {
        let lattice = if let Some(path) = path {
            crate::load_mad_file(path).map_err(py_error)?
        } else {
            crate::Lattice::default()
        };
        Ok(Self {
            lattice: Arc::new(lattice),
        })
    }

    fn __getattr__(&self, name: &str) -> PyResult<PyLine> {
        if self.lattice.lines.contains_key(name) {
            Ok(PyLine {
                lattice: Arc::clone(&self.lattice),
                name: name.to_string(),
            })
        } else {
            Err(PyValueError::new_err(format!("unknown line `{name}`")))
        }
    }
}

#[pyclass(name = "Line", module = "gufo", skip_from_py_object)]
#[derive(Clone)]
struct PyLine {
    lattice: Arc<crate::Lattice>,
    name: String,
}

#[pymethods]
impl PyLine {
    #[getter]
    fn label(&self) -> &str {
        &self.name
    }

    fn count(&self) -> PyResult<usize> {
        let line = self.lattice.line(&self.name).map_err(py_error)?;
        line.count(&self.lattice).map_err(py_error)
    }

    fn length(&self) -> PyResult<f64> {
        let line = self.lattice.line(&self.name).map_err(py_error)?;
        line.length(&self.lattice).map_err(py_error)
    }

    fn angle(&self) -> PyResult<f64> {
        let line = self.lattice.line(&self.name).map_err(py_error)?;
        line.angle(&self.lattice).map_err(py_error)
    }
}

#[pyclass(name = "Track", module = "gufo")]
struct PyTrack {
    line: PyLine,
    flags: crate::PassFlags,
    turns: u32,
    parameters_order: Vec<String>,
    where_: Vec<f64>,
    dp: f64,
    parameters: Py<PyAny>,
    tracks: Py<PyAny>,
}

#[pymethods]
impl PyTrack {
    #[new]
    #[pyo3(signature = (line, flags=0, turns=1000, particles=1000, parameters=None, r#where=None, dp=0.0, context=None, options=None))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        py: Python<'_>,
        line: PyRef<'_, PyLine>,
        flags: u8,
        turns: u32,
        particles: usize,
        parameters: Option<Vec<String>>,
        r#where: Option<Vec<f64>>,
        dp: f64,
        context: Option<Py<PyAny>>,
        options: Option<Py<PyAny>>,
    ) -> PyResult<Self> {
        let _ = (context, options);
        let parameters_order = parameters.unwrap_or_default();
        validate_parameters(&parameters_order)?;
        let numpy = py.import("numpy")?;
        let parameters = numpy
            .call_method1("zeros", ((particles, parameters_order.len()),))?
            .unbind();
        let tracks = numpy
            .call_method1("zeros", ((particles, 0usize, 6usize),))?
            .unbind();
        Ok(Self {
            line: line.clone(),
            flags: crate::PassFlags::from_bits_truncate(flags),
            turns,
            parameters_order,
            where_: r#where.unwrap_or_default(),
            dp,
            parameters,
            tracks,
        })
    }

    #[getter]
    fn parameters(&self, py: Python<'_>) -> Py<PyAny> {
        self.parameters.clone_ref(py)
    }

    #[setter]
    fn set_parameters(&mut self, value: Py<PyAny>) {
        self.parameters = value;
    }

    #[getter]
    fn tracks(&self, py: Python<'_>) -> Py<PyAny> {
        self.tracks.clone_ref(py)
    }

    #[pyo3(signature = (threads=None))]
    fn run(&mut self, py: Python<'_>, threads: Option<usize>) -> PyResult<()> {
        let _ = threads;
        let particles = self.read_particles(py)?;
        let line = self
            .line
            .lattice
            .line(&self.line.name)
            .map_err(py_error)?
            .clone();
        let mut track = crate::Track::new(
            &self.line.lattice,
            &line,
            particles,
            crate::TrackOptions {
                flags: self.flags,
                turns: self.turns,
                where_: self.where_.clone(),
                is_64bit: self.flags.contains(crate::PassFlags::DOUBLE_PRECISION),
                collapse_linear: false,
            },
        )
        .map_err(py_error)?;
        track
            .run_with_options(&crate::cubecl::CubeClTrackRunOptions {
                turns: self.turns,
                backend: crate::cubecl::CubeClBackend::Auto,
                device: crate::cubecl::CubeClDevice::Default,
            })
            .map_err(py_error)?;
        self.tracks = samples_to_numpy(py, track.samples, track.particles.len())?;
        Ok(())
    }
}

impl PyTrack {
    fn read_particles(&self, py: Python<'_>) -> PyResult<Vec<crate::Particle>> {
        let rows: Vec<Vec<f64>> = self.parameters.bind(py).call_method0("tolist")?.extract()?;
        rows.into_iter()
            .map(|row| self.row_to_particle(&row))
            .collect()
    }

    fn row_to_particle(&self, row: &[f64]) -> PyResult<crate::Particle> {
        let mut particle = crate::Particle::default();
        for (idx, name) in self.parameters_order.iter().enumerate() {
            let value = row.get(idx).copied().ok_or_else(|| {
                PyValueError::new_err("parameters row is shorter than parameters list")
            })?;
            match name.as_str() {
                "x" => particle.x = value,
                "px" => particle.px = value,
                "y" => particle.y = value,
                "py" => particle.py = value,
                "z" => particle.z = value,
                "dp" => particle.dp = value,
                _ => unreachable!(),
            }
        }
        if self.flags.contains(crate::PassFlags::FIVED) {
            particle.z = 0.0;
            particle.dp = self.dp;
        }
        Ok(particle)
    }
}

fn validate_parameters(parameters: &[String]) -> PyResult<()> {
    for parameter in parameters {
        if !matches!(parameter.as_str(), "x" | "px" | "y" | "py" | "z" | "dp") {
            return Err(PyValueError::new_err(format!(
                "only coordinate parameters are currently supported: `{parameter}`"
            )));
        }
    }
    Ok(())
}

fn samples_to_numpy(
    py: Python<'_>,
    samples: Vec<crate::Particle>,
    particles: usize,
) -> PyResult<Py<PyAny>> {
    let sample_count = if particles == 0 {
        0
    } else {
        samples.len() / particles
    };
    let mut rows = vec![vec![vec![0.0; 6]; sample_count]; particles];
    for sample in 0..sample_count {
        for particle in 0..particles {
            let value = samples[sample * particles + particle];
            rows[particle][sample] = vec![value.x, value.px, value.y, value.py, value.z, value.dp];
        }
    }
    let numpy = py.import("numpy")?;
    Ok(numpy.call_method1("array", (rows,))?.unbind())
}

#[pyfunction]
fn list_devices() {
    for device in crate::cubecl::list_devices() {
        if let Some(name) = device.name {
            println!("{}\t{}", device.selector, name);
        } else {
            println!("{}", device.selector);
        }
    }
}

#[pymodule]
fn gufo(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyLattice>()?;
    m.add_class::<PyLine>()?;
    m.add_class::<PyTrack>()?;
    m.add_function(wrap_pyfunction!(list_devices, m)?)?;
    m.add("LINEAR", crate::PassFlags::LINEAR.bits())?;
    m.add("FIVED", crate::PassFlags::FIVED.bits())?;
    m.add("EXACT", crate::PassFlags::EXACT.bits())?;
    m.add("KICK", crate::PassFlags::KICK.bits())?;
    m.add("RADIATION", crate::PassFlags::RADIATION.bits())?;
    m.add(
        "DOUBLE_PRECISION",
        crate::PassFlags::DOUBLE_PRECISION.bits(),
    )?;
    m.add("ACHROMATIC", crate::PassFlags::ACHROMATIC.bits())?;
    Ok(())
}
