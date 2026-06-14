use std::{fs, path::Path};

use crate::{Beam, Element, Lattice, Line, LineItem, Result, UfoError};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DumpStyle {
    Mad,
    Elegant,
    At,
    Opa,
}

impl DumpStyle {
    pub fn name(self) -> &'static str {
        match self {
            Self::Mad => "mad",
            Self::Elegant => "elegant",
            Self::At => "at",
            Self::Opa => "opa",
        }
    }
}

pub fn dump_lattice_file(
    lattice: &Lattice,
    path: impl AsRef<Path>,
    style: DumpStyle,
) -> Result<()> {
    let path = path.as_ref();
    fs::write(path, to_lattice_string(lattice, style)?).map_err(|source| UfoError::WriteFile {
        path: path.to_path_buf(),
        source,
    })
}

pub fn dump_mad_file(lattice: &Lattice, path: impl AsRef<Path>) -> Result<()> {
    dump_lattice_file(lattice, path, DumpStyle::Mad)
}

pub fn to_mad_string(lattice: &Lattice) -> String {
    to_lattice_string(lattice, DumpStyle::Mad).expect("MAD export is supported for all elements")
}

pub fn to_lattice_string(lattice: &Lattice, style: DumpStyle) -> Result<String> {
    let mut out = String::new();
    if style == DumpStyle::At {
        out.push_str(&at_header("ufo_lattice", &Beam::default()));
    } else if style == DumpStyle::Opa {
        out.push_str(&format!(
            "energy = {};\r\n",
            Beam::default().energy * 1.0e-9
        ));
    }

    for element in lattice.elements.values() {
        out.push_str(&element_to_style(element, style)?);
    }
    if !lattice.elements.is_empty() && !lattice.lines.is_empty() {
        out.push('\n');
    }
    for line in lattice.lines.values() {
        out.push_str(&line_to_style(line, style));
    }

    if style == DumpStyle::At {
        out.push_str("\n\nbuildlat(RING);\n");
        out.push_str("% Set all magnets to same energy\n");
        out.push_str(
            "THERING = setcellstruct(THERING, 'Energy', 1:length(THERING), GLOBVAL.E0);\n",
        );
        out.push_str("evalin('caller', 'global THERING FAMLIST GLOBVAL');\n");
        out.push_str("if nargout\n    varargout{1} = THERING;\nend\nend");
    }
    Ok(out)
}

fn element_to_style(element: &Element, style: DumpStyle) -> Result<String> {
    match style {
        DumpStyle::Mad | DumpStyle::Elegant => Ok(element_to_mad_like(element)),
        DumpStyle::At => element_to_at(element),
        DumpStyle::Opa => element_to_opa(element),
    }
}

fn element_to_mad_like(element: &Element) -> String {
    match element {
        Element::Marker(v) => format!("{}: MARKER;\n", v.label),
        Element::Drift(v) => format!("{}: DRIFT, L={};\n", v.label, v.length),
        Element::Multipole(v) => format!(
            "{}: MULTIPOLE{}{}{}{};\n",
            v.label,
            vector_param("KNL", &v.knl),
            vector_param("KSL", &v.ksl),
            nonzero_param("DX", v.dx),
            nonzero_param("DY", v.dy),
        ),
        Element::Quadrupole(v) => format!(
            "{}: QUADRUPOLE, L={}, K1={}{}{}{}{};\n",
            v.label,
            v.length,
            v.k1,
            nonzero_param("DX", v.dx),
            nonzero_param("DY", v.dy),
            vector_param("DKN", &v.dkn),
            vector_param("DKS", &v.dks),
        ),
        Element::Sbend(v) => format!(
            "{}: SBEND, L={}, ANGLE={}, K1={}, E1={}, E2={}, HGAP={}, FINT={}{}{}{}{};\n",
            v.label,
            v.length,
            v.angle,
            v.k1,
            v.e1,
            v.e2,
            v.hgap,
            v.fint,
            nonzero_param("DX", v.dx),
            nonzero_param("DY", v.dy),
            vector_param("DKN", &v.dkn),
            vector_param("DKS", &v.dks),
        ),
        Element::Rbend(v) => format!(
            "{}: RBEND, L={}, ANGLE={}, K1={}, E1={}, E2={}, HGAP={}, FINT={}{}{}{}{};\n",
            v.label,
            v.length,
            v.angle,
            v.k1,
            v.e1,
            v.e2,
            v.hgap,
            v.fint,
            nonzero_param("DX", v.dx),
            nonzero_param("DY", v.dy),
            vector_param("DKN", &v.dkn),
            vector_param("DKS", &v.dks),
        ),
        Element::Sextupole(v) => format!(
            "{}: SEXTUPOLE, L={}, K2={}, K2S={}{}{}{}{};\n",
            v.label,
            v.length,
            v.k2,
            v.k2s,
            nonzero_param("DX", v.dx),
            nonzero_param("DY", v.dy),
            vector_param("DKN", &v.dkn),
            vector_param("DKS", &v.dks),
        ),
        Element::Octupole(v) => format!(
            "{}: OCTUPOLE, L={}, K3={}, K3S={}{}{}{}{};\n",
            v.label,
            v.length,
            v.k3,
            v.k3s,
            nonzero_param("DX", v.dx),
            nonzero_param("DY", v.dy),
            vector_param("DKN", &v.dkn),
            vector_param("DKS", &v.dks),
        ),
        Element::Wire(v) => format!("{}: WIRE, X={}, Y={}, K={};\n", v.label, v.x, v.y, v.k),
        Element::Cavity(v) => format!(
            "{}: CAVITY, FIELD={}, OMEGA={}, LAG={};\n",
            v.label, v.field, v.omega, v.lag
        ),
        Element::Aperture(v) => {
            if let Some(radius) = v.radius {
                format!("{}: APERTURE, RADIUS={};\n", v.label, radius)
            } else {
                format!("{}: APERTURE, WINDOW=\"{}\";\n", v.label, v.window)
            }
        }
    }
}

fn element_to_at(element: &Element) -> Result<String> {
    match element {
        Element::Marker(v) => Ok(format!(
            "{} = marker('{}', 'IdentityPass');\n",
            v.label, v.label
        )),
        Element::Drift(v) => Ok(format!(
            "{} = drift('{}', {}, 'DriftPass');\n",
            v.label, v.label, v.length
        )),
        Element::Quadrupole(v) => Ok(format!(
            "{} = quadrupole('{}', {}, {}, 'StrMPoleSymplectic4Pass');\n",
            v.label, v.label, v.length, v.k1
        )),
        Element::Rbend(v) => Ok(format!(
            "{} = rbend('{}', {}, {}, 0, 0, {}, 'BndMPoleSymplectic4Pass');\n",
            v.label, v.label, v.length, v.angle, v.k1
        )),
        Element::Sextupole(v) => Ok(format!(
            "{} = sextupole('{}', {}, {}, 'StrMPoleSymplectic4Pass');\n",
            v.label, v.label, v.length, v.k2
        )),
        Element::Octupole(v) => Ok(format!(
            "{} = octupole('{}', {}, {}, 'StrMPoleSymplectic4Pass');\n",
            v.label, v.label, v.length, v.k3
        )),
        other => unsupported_export(other, DumpStyle::At),
    }
}

fn element_to_opa(element: &Element) -> Result<String> {
    match element {
        Element::Marker(v) => Ok(format!("{} : opticsmarker;\r\n", v.label)),
        Element::Drift(v) => Ok(format!("{} : drift, l = {};\r\n", v.label, v.length)),
        Element::Quadrupole(v) => Ok(format!(
            "{} : quadrupole, l = {}, k = {};\r\n",
            v.label, v.length, v.k1
        )),
        Element::Rbend(v) => {
            let angle_deg = v.angle / (2.0 * std::f64::consts::PI) * 360.0;
            let e1_deg = v.e1 / (2.0 * std::f64::consts::PI) * 360.0;
            let e2_deg = v.e2 / (2.0 * std::f64::consts::PI) * 360.0;
            Ok(format!(
                "{} : bending, l = {}, t = {}, k = {}, t1 = {}, t2 = {};\r\n",
                v.label, v.length, angle_deg, v.k1, e1_deg, e2_deg
            ))
        }
        Element::Sextupole(v) => Ok(format!(
            "{} : sextupole, l = {}, k = {};\r\n",
            v.label,
            v.length,
            v.k2 * 0.5
        )),
        Element::Octupole(v) => Ok(format!(
            "{} : octupole, l = {}, k = {};\r\n",
            v.label,
            v.length,
            v.k3 * 0.5
        )),
        other => unsupported_export(other, DumpStyle::Opa),
    }
}

fn unsupported_export<T>(element: &Element, style: DumpStyle) -> Result<T> {
    Err(UfoError::UnsupportedParameter {
        kind: element.label().to_string(),
        parameter: format!("{} export", style.name()),
    })
}

fn line_to_style(line: &Line, style: DumpStyle) -> String {
    let items = line
        .items
        .iter()
        .map(|item| match item {
            LineItem::Element(label) | LineItem::Line(label) => label.as_str(),
        })
        .collect::<Vec<_>>()
        .join(", ");
    match style {
        DumpStyle::Mad => format!("{}: LINE=({});\n", line.label, items),
        DumpStyle::Elegant => format!("{}: LINE=({})\n", line.label, items),
        DumpStyle::At => format!("{} = [{}];\n\n", line.label, items),
        DumpStyle::Opa => format!("{} : {};\r\n", line.label, items),
    }
}

fn at_header(name: &str, beam: &Beam) -> String {
    format!(
        "function {name}\n\n\
global FAMLIST THERING GLOBVAL;\n\
E_0 = {};\n\
GLOBVAL.E0 = {}-E_0;\n\
GLOBVAL.LatticeFile = '{name}';\n\
FAMLIST = cell(0);\n\
THERING = cell(0);\n\n",
        beam.particle_mass, beam.energy
    )
}

fn vector_param(name: &str, values: &[f64]) -> String {
    if values.is_empty() {
        return String::new();
    }
    let values = values
        .iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    format!(", {name}={{{values}}}")
}

fn nonzero_param(name: &str, value: f64) -> String {
    if value == 0.0 {
        String::new()
    } else {
        format!(", {name}={value}")
    }
}
