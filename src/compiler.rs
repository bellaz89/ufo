use std::collections::BTreeMap;

use nalgebra::{SMatrix, SVector};

use crate::{
    Bytecode, DEFAULT_BEND_SLICES, DEFAULT_OCTUPOLE_SLICES, DEFAULT_SEXTUPOLE_SLICES, Element,
    Instruction, Lattice, Line, PassFlags, Result,
};

#[derive(Clone, Debug)]
pub struct CompileOptions {
    pub flags: PassFlags,
    pub turns: u32,
    pub is_64bit: bool,
    pub collapse_linear: bool,
}

impl Default for CompileOptions {
    fn default() -> Self {
        Self {
            flags: PassFlags::empty(),
            turns: 1,
            is_64bit: false,
            collapse_linear: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TrackCompileOptions {
    pub flags: PassFlags,
    pub turns: u32,
    pub is_64bit: bool,
    pub where_: Vec<f64>,
    pub collapse_linear: bool,
}

impl Default for TrackCompileOptions {
    fn default() -> Self {
        Self {
            flags: PassFlags::empty(),
            turns: 1,
            is_64bit: false,
            where_: vec![-1.0],
            collapse_linear: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TrackingBytecode {
    pub bytecode: Bytecode,
    pub dumps_per_turn: usize,
    pub instructions_per_turn: usize,
}

pub fn compile_line(lattice: &Lattice, line: &Line, options: &CompileOptions) -> Result<Bytecode> {
    let is_64bit = options.is_64bit || options.flags.contains(PassFlags::DOUBLE_PRECISION);
    let mut instructions = Vec::new();
    for element in line.flatten(lattice)? {
        compile_element(element, options.flags, &mut instructions);
    }
    if options.collapse_linear {
        instructions =
            collapse_linear_instructions(instructions, options.flags.contains(PassFlags::LINEAR));
    }
    instructions.push(Instruction::rewind("rewind"));
    Ok(Bytecode::from_instructions(instructions, is_64bit))
}

pub fn compile_tracking_line(
    lattice: &Lattice,
    line: &Line,
    options: &TrackCompileOptions,
) -> Result<TrackingBytecode> {
    let is_64bit = options.is_64bit || options.flags.contains(PassFlags::DOUBLE_PRECISION);
    let flat = line.flatten(lattice)?;
    let observations = observations(&options.where_, flat.len())?;
    let mut instructions = Vec::new();

    for (idx, element) in flat.iter().enumerate() {
        if observations.boundaries.contains(&idx) {
            instructions.push(Instruction::dump(format!("dump_{idx}")));
        }
        if let Some(fractions) = observations.inside.get(&idx) {
            compile_element_with_observations(
                element,
                options.flags,
                idx,
                fractions,
                &mut instructions,
            )?;
        } else {
            compile_element(element, options.flags, &mut instructions);
        }
    }

    if observations.end {
        instructions.push(Instruction::dump("dump_end"));
    }

    if options.collapse_linear {
        instructions =
            collapse_linear_instructions(instructions, options.flags.contains(PassFlags::LINEAR));
    }

    instructions.push(Instruction::rewind("rewind"));
    let instructions_per_turn = instructions.len();
    Ok(TrackingBytecode {
        bytecode: Bytecode::from_instructions(instructions, is_64bit),
        dumps_per_turn: observations.count(),
        instructions_per_turn,
    })
}

#[derive(Clone, Debug, Default)]
struct Observations {
    boundaries: Vec<usize>,
    inside: BTreeMap<usize, Vec<f64>>,
    end: bool,
}

impl Observations {
    fn count(&self) -> usize {
        self.boundaries.len()
            + self.inside.values().map(Vec::len).sum::<usize>()
            + usize::from(self.end)
    }
}

fn observations(values: &[f64], elements: usize) -> Result<Observations> {
    let mut observations = Observations::default();
    for value in values {
        if *value == -1.0 {
            observations.end = true;
            continue;
        }
        if !value.is_finite() || *value < 0.0 {
            return Err(crate::UfoError::UnsupportedObservation(value.to_string()));
        }
        let boundary = value.floor() as usize;
        if boundary > elements {
            return Err(crate::UfoError::UnsupportedObservation(value.to_string()));
        }
        let fraction = value.fract();
        if fraction == 0.0 && boundary == elements {
            observations.end = true;
        } else if fraction == 0.0 && !observations.boundaries.contains(&boundary) {
            observations.boundaries.push(boundary);
        } else if boundary >= elements {
            return Err(crate::UfoError::UnsupportedObservation(value.to_string()));
        } else {
            let fractions = observations.inside.entry(boundary).or_default();
            if !fractions.contains(&fraction) {
                fractions.push(fraction);
                fractions.sort_by(f64::total_cmp);
            }
        }
    }
    Ok(observations)
}

fn compile_element(element: &Element, flags: PassFlags, out: &mut Vec<Instruction>) {
    compile_element_segment(element, flags, 1.0, true, true, out);
}

fn compile_element_with_observations(
    element: &Element,
    flags: PassFlags,
    index: usize,
    fractions: &[f64],
    out: &mut Vec<Instruction>,
) -> Result<()> {
    if element.length() == 0.0 {
        return Err(crate::UfoError::UnsupportedObservation(format!(
            "{index}+fraction for zero-length element `{}`",
            element.label()
        )));
    }

    let mut previous = 0.0;
    for fraction in fractions {
        compile_element_segment(
            element,
            flags,
            fraction - previous,
            previous == 0.0,
            false,
            out,
        );
        out.push(Instruction::dump(format!("dump_{index}_{fraction}")));
        previous = *fraction;
    }
    compile_element_segment(element, flags, 1.0 - previous, false, true, out);
    Ok(())
}

fn compile_element_segment(
    element: &Element,
    flags: PassFlags,
    fraction: f64,
    entry_edge: bool,
    exit_edge: bool,
    out: &mut Vec<Instruction>,
) {
    let exact = flags.contains(PassFlags::EXACT);
    let linear = flags.contains(PassFlags::LINEAR);
    let achromatic = flags.contains(PassFlags::ACHROMATIC);
    let kick = flags.contains(PassFlags::KICK);
    match element {
        Element::Marker(_) => {}
        Element::Drift(v) => out.push(Instruction::drift(&v.label, v.length * fraction, exact)),
        Element::Multipole(v) => {
            push_align(&v.label, v.dx, v.dy, true, out);
            if linear {
                out.push(Instruction::linear_kick(&v.label, &v.knl, &v.ksl, true));
            } else {
                out.push(Instruction::kick(
                    &v.label,
                    v.knl.clone(),
                    v.ksl.clone(),
                    false,
                    achromatic,
                ));
            }
            push_align(&v.label, v.dx, v.dy, false, out);
        }
        Element::Quadrupole(v) => {
            if entry_edge {
                push_align(&v.label, v.dx, v.dy, true, out);
            }
            if kick {
                let mut knl = pad(v.dkn.clone(), 2);
                knl[1] += v.k1;
                out.push(teapot(
                    &v.label,
                    v.length * fraction,
                    v.slices,
                    knl,
                    v.dks.clone(),
                    0.0,
                    flags,
                ));
            } else {
                out.push(Instruction::quadrupole(
                    &v.label,
                    v.length * fraction,
                    v.k1,
                    achromatic,
                ));
            }
            if exit_edge {
                push_align(&v.label, v.dx, v.dy, false, out);
            }
        }
        Element::Sbend(v) => {
            let fringe = v.hgap * v.fint;
            if entry_edge {
                push_align(&v.label, v.dx, v.dy, true, out);
                out.push(Instruction::edge(
                    format!("{}_entry_edge", v.label),
                    v.length,
                    v.angle,
                    v.e1,
                    fringe,
                    achromatic,
                ));
            }
            if kick {
                let mut knl = pad(v.dkn.clone(), 2);
                knl[1] += v.k1;
                out.push(teapot(
                    &v.label,
                    v.length * fraction,
                    v.slices.max(DEFAULT_BEND_SLICES),
                    knl,
                    v.dks.clone(),
                    v.angle * fraction,
                    flags,
                ));
            } else {
                out.push(Instruction::sbend(
                    &v.label,
                    v.length * fraction,
                    v.angle * fraction,
                    v.k1,
                    achromatic,
                ));
            }
            if exit_edge {
                out.push(Instruction::edge(
                    format!("{}_exit_edge", v.label),
                    v.length,
                    v.angle,
                    v.e2,
                    fringe,
                    achromatic,
                ));
                push_align(&v.label, v.dx, v.dy, false, out);
            }
        }
        Element::Rbend(v) => {
            let fringe = v.hgap * v.fint;
            if entry_edge {
                push_align(&v.label, v.dx, v.dy, true, out);
                out.push(Instruction::edge(
                    format!("{}_entry_edge", v.label),
                    v.length,
                    v.angle,
                    v.e1 + 0.5 * v.angle,
                    fringe,
                    achromatic,
                ));
            }
            out.push(Instruction::sbend(
                &v.label,
                v.length * fraction,
                v.angle * fraction,
                v.k1,
                achromatic,
            ));
            if exit_edge {
                out.push(Instruction::edge(
                    format!("{}_exit_edge", v.label),
                    v.length,
                    v.angle,
                    v.e2 + 0.5 * v.angle,
                    fringe,
                    achromatic,
                ));
                push_align(&v.label, v.dx, v.dy, false, out);
            }
        }
        Element::Sextupole(v) => {
            if entry_edge {
                push_align(&v.label, v.dx, v.dy, true, out);
            }
            let mut knl = pad(v.dkn.clone(), 3);
            let mut ksl = pad(v.dks.clone(), 3);
            knl[2] += v.k2;
            ksl[2] += v.k2s;
            out.push(teapot(
                &v.label,
                v.length * fraction,
                v.slices.max(DEFAULT_SEXTUPOLE_SLICES),
                knl,
                ksl,
                0.0,
                flags,
            ));
            if exit_edge {
                push_align(&v.label, v.dx, v.dy, false, out);
            }
        }
        Element::Octupole(v) => {
            if entry_edge {
                push_align(&v.label, v.dx, v.dy, true, out);
            }
            let mut knl = pad(v.dkn.clone(), 4);
            let mut ksl = pad(v.dks.clone(), 4);
            knl[3] += v.k3;
            ksl[3] += v.k3s;
            out.push(teapot(
                &v.label,
                v.length * fraction,
                v.slices.max(DEFAULT_OCTUPOLE_SLICES),
                knl,
                ksl,
                0.0,
                flags,
            ));
            if exit_edge {
                push_align(&v.label, v.dx, v.dy, false, out);
            }
        }
        Element::Wire(v) => out.push(Instruction::wire(&v.label, v.x, v.y, v.k)),
        Element::Cavity(v) => out.push(Instruction::cavity(&v.label, v.field, v.omega, v.lag)),
        Element::Aperture(v) => {
            if let Some(radius) = v.radius {
                out.push(Instruction::set_aperture(&v.label, radius));
            }
            // String WINDOW expressions are intentionally not compiled into
            // bytecode; the kernel only supports circular aperture checks.
        }
    }
}

fn push_align(label: &str, dx: f64, dy: f64, entry: bool, out: &mut Vec<Instruction>) {
    if dx == 0.0 && dy == 0.0 {
        return;
    }
    let sign = if entry { -1.0 } else { 1.0 };
    out.push(Instruction::tran_linear(
        format!("{}_{}align", label, if entry { "entry_" } else { "exit_" }),
        identity4(),
        [sign * dx, sign * dy, 0.0, 0.0],
        true,
    ));
}

fn identity4() -> [[f64; 4]; 4] {
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

fn teapot(
    label: &str,
    length: f64,
    slices: u32,
    knl: Vec<f64>,
    ksl: Vec<f64>,
    angle: f64,
    flags: PassFlags,
) -> Instruction {
    let slices_f = slices as f64;
    let outer = if slices == 1 {
        0.5 * length
    } else {
        0.5 * length / (1.0 + slices_f)
    };
    let inner = if slices > 1 {
        (length - 2.0 * outer) / (slices_f - 1.0)
    } else {
        0.0
    };
    let weak_coeff = if length == 0.0 {
        0.0
    } else {
        angle * angle / (slices_f * length)
    };
    let mut knl: Vec<f64> = knl.into_iter().map(|k| k * length / slices_f).collect();
    let mut ksl: Vec<f64> = ksl.into_iter().map(|k| k * length / slices_f).collect();
    let mut inst_flags = PassFlags::empty();
    if flags.contains(PassFlags::EXACT) {
        inst_flags |= PassFlags::EXACT;
    }
    if flags.contains(PassFlags::LINEAR) {
        inst_flags |= PassFlags::LINEAR;
        knl.truncate(2);
        ksl.truncate(2);
    }
    if flags.contains(PassFlags::ACHROMATIC) {
        inst_flags |= PassFlags::ACHROMATIC;
    }
    Instruction::teapot(
        label, inner, outer, weak_coeff, slices_f, knl, ksl, inst_flags,
    )
}

fn pad(mut values: Vec<f64>, len: usize) -> Vec<f64> {
    values.resize(len, 0.0);
    values
}

const TRAN_FLAG_VEC: u8 = 0x1 << 0;
const TRAN_FLAG_MAT_XX: u8 = 0x1 << 1;
const TRAN_FLAG_MAT_PXX: u8 = 0x1 << 2;
const TRAN_FLAG_MAT_XPX: u8 = 0x1 << 3;
const TRAN_FLAG_MAT_PXPX: u8 = 0x1 << 4;
const TRAN_FLAG_DP: u8 = 0x1 << 5;
const TRAN_FLAG_NO_PASS: u8 = 0x1 << 6;

#[derive(Clone, Debug)]
struct Affine5 {
    mat: SMatrix<f64, 5, 5>,
    vec: SVector<f64, 5>,
    pass_count: usize,
    names: Vec<String>,
}

impl Affine5 {
    fn then(&mut self, next: &Self) {
        self.vec = next.mat * self.vec + next.vec;
        self.mat = next.mat * self.mat;
        self.pass_count += next.pass_count;
        self.names.extend(next.names.iter().cloned());
    }

    fn into_instruction(self) -> Instruction {
        let name = if self.names.is_empty() {
            "collapsed_linear".to_string()
        } else {
            format!("collapsed_{}", self.names.join("_"))
        };
        Instruction::tran_linear_with_options(
            name,
            canonical_to_instruction_matrix(&self.mat),
            [self.vec[0], self.vec[1], self.vec[2], self.vec[3]],
            [
                self.mat[(0, 4)],
                self.mat[(1, 4)],
                self.mat[(2, 4)],
                self.mat[(3, 4)],
            ],
            self.pass_count == 0,
            self.pass_count == 0,
        )
    }
}

fn collapse_linear_instructions(
    instructions: Vec<Instruction>,
    collapse_element_ops: bool,
) -> Vec<Instruction> {
    let mut out = Vec::with_capacity(instructions.len());
    let mut current: Option<Affine5> = None;

    for instruction in instructions {
        let Some(next) = affine_from_instruction(&instruction, collapse_element_ops) else {
            flush_affine(&mut current, &mut out);
            out.push(instruction);
            continue;
        };

        if current
            .as_ref()
            .is_some_and(|run| run.pass_count + next.pass_count > 1)
        {
            flush_affine(&mut current, &mut out);
        }

        if let Some(run) = &mut current {
            run.then(&next);
        } else {
            current = Some(next);
        }
    }

    flush_affine(&mut current, &mut out);
    out
}

fn flush_affine(current: &mut Option<Affine5>, out: &mut Vec<Instruction>) {
    if let Some(run) = current.take() {
        out.push(run.into_instruction());
    }
}

fn affine_from_instruction(
    instruction: &Instruction,
    collapse_element_ops: bool,
) -> Option<Affine5> {
    if instruction.op == crate::OP_TRAN_LINEAR {
        return tran_linear_affine(instruction);
    }
    if !collapse_element_ops {
        return None;
    }

    match instruction.op {
        crate::OP_DRIFT => drift_affine(instruction),
        crate::OP_QUADRUPOLE => quadrupole_affine(instruction),
        crate::OP_SBEND => sbend_affine(instruction),
        crate::OP_EDGE => edge_affine(instruction),
        crate::OP_TEAPOT => teapot_affine(instruction),
        _ => None,
    }
}

fn tran_linear_affine(instruction: &Instruction) -> Option<Affine5> {
    let flags = instruction.flags.bits();

    let mut mat = identity5_canonical();
    let mut vec = SVector::<f64, 5>::zeros();
    let mut idx = 0usize;

    if flags & TRAN_FLAG_VEC != 0 {
        if instruction.args.len() < idx + 4 {
            return None;
        }
        vec.fixed_rows_mut::<4>(0)
            .copy_from_slice(&instruction.args[idx..idx + 4]);
        idx += 4;
    }

    for (flag, rows, cols) in [
        (TRAN_FLAG_MAT_XX, [0usize, 1usize], [0usize, 1usize]),
        (TRAN_FLAG_MAT_PXX, [0usize, 1usize], [2usize, 3usize]),
        (TRAN_FLAG_MAT_XPX, [2usize, 3usize], [0usize, 1usize]),
        (TRAN_FLAG_MAT_PXPX, [2usize, 3usize], [2usize, 3usize]),
    ] {
        if flags & flag == 0 {
            continue;
        }
        if instruction.args.len() < idx + 4 {
            return None;
        }
        for (r_idx, row) in rows.into_iter().enumerate() {
            for (c_idx, col) in cols.into_iter().enumerate() {
                mat[(row, col)] += instruction.args[idx + r_idx * 2 + c_idx];
            }
        }
        idx += 4;
    }

    if flags & TRAN_FLAG_DP != 0 {
        if instruction.args.len() < idx + 4 {
            return None;
        }
        mat[(0, 4)] = instruction.args[idx];
        mat[(1, 4)] = instruction.args[idx + 1];
        mat[(2, 4)] = instruction.args[idx + 2];
        mat[(3, 4)] = instruction.args[idx + 3];
        idx += 4;
    }

    if idx != instruction.args.len() {
        return None;
    }

    Some(Affine5 {
        mat,
        vec,
        pass_count: usize::from(flags & TRAN_FLAG_NO_PASS == 0),
        names: vec![instruction.name.clone()],
    })
}

fn drift_affine(instruction: &Instruction) -> Option<Affine5> {
    if instruction.flags.contains(PassFlags::EXACT) || instruction.args.len() != 1 {
        return None;
    }
    let length = instruction.args[0];
    let mut mat = identity5_canonical();
    mat[(0, 2)] = length;
    mat[(1, 3)] = length;
    Some(affine_instruction(
        instruction,
        mat,
        SVector::<f64, 5>::zeros(),
    ))
}

fn quadrupole_affine(instruction: &Instruction) -> Option<Affine5> {
    if !instruction.flags.contains(PassFlags::ACHROMATIC) || instruction.args.len() != 2 {
        return None;
    }
    let k1 = instruction.args[0];
    let length = instruction.args[1];
    let mut mat = identity5_canonical();
    set_plane(&mut mat, 0, 2, plane_matrix(k1, length));
    set_plane(&mut mat, 1, 3, plane_matrix(-k1, length));
    Some(affine_instruction(
        instruction,
        mat,
        SVector::<f64, 5>::zeros(),
    ))
}

fn sbend_affine(instruction: &Instruction) -> Option<Affine5> {
    if !instruction.flags.contains(PassFlags::ACHROMATIC) || instruction.args.len() != 3 {
        return None;
    }
    let length = instruction.args[0];
    let curvature = instruction.args[1];
    let k1 = instruction.args[2];
    let horizontal_k = k1 + curvature * curvature;
    let mut mat = identity5_canonical();
    set_plane(&mut mat, 0, 2, plane_matrix(horizontal_k, length));
    set_plane(&mut mat, 1, 3, plane_matrix(-k1, length));

    if horizontal_k > 0.0 {
        let k2 = horizontal_k.sqrt();
        let k2l = length * k2;
        let c = k2l.cos();
        let s = k2l.sin();
        mat[(0, 4)] = curvature * (1.0 - c) / horizontal_k.abs();
        mat[(2, 4)] = curvature * s / k2;
    } else if horizontal_k < 0.0 {
        let k2 = horizontal_k.abs().sqrt();
        let k2l = length * k2;
        let ch = k2l.cosh();
        let sh = k2l.sinh();
        mat[(0, 4)] = curvature * (ch - 1.0) / horizontal_k.abs();
        mat[(2, 4)] = curvature * sh / k2;
    }

    Some(affine_instruction(
        instruction,
        mat,
        SVector::<f64, 5>::zeros(),
    ))
}

fn edge_affine(instruction: &Instruction) -> Option<Affine5> {
    if !instruction.flags.contains(PassFlags::ACHROMATIC) || instruction.args.len() != 4 {
        return None;
    }
    let curvature = instruction.args[0];
    let edge_angle = instruction.args[1];
    let tan_edge = instruction.args[2];
    let psi_coeff = instruction.args[3];
    let psi = edge_angle - curvature * psi_coeff;

    let mut mat = identity5_canonical();
    mat[(2, 0)] += curvature * tan_edge;
    mat[(3, 1)] -= curvature * psi.tan();
    Some(affine_instruction(
        instruction,
        mat,
        SVector::<f64, 5>::zeros(),
    ))
}

fn teapot_affine(instruction: &Instruction) -> Option<Affine5> {
    if instruction.flags.contains(PassFlags::EXACT)
        || !instruction.flags.contains(PassFlags::LINEAR)
        || !instruction.flags.contains(PassFlags::ACHROMATIC)
        || instruction.args.len() < 4
    {
        return None;
    }

    let inner = instruction.args[0];
    let outer = instruction.args[1];
    let weak_coeff = instruction.args[2];
    let slices = instruction.args[3] as u32;
    let knl_size = instruction.aux as usize;
    if instruction.args.len() < 4 + knl_size || slices == 0 {
        return None;
    }
    let knl = &instruction.args[4..4 + knl_size];
    let ksl = &instruction.args[4 + knl_size..];
    if knl.len() > 2 || ksl.len() > 2 {
        return None;
    }

    let mut out = Affine5 {
        mat: identity5_canonical(),
        vec: SVector::<f64, 5>::zeros(),
        pass_count: usize::from(!instruction.flags.contains(PassFlags::NO_APERTURE_CHECK)),
        names: vec![instruction.name.clone()],
    };
    out.then(&drift_map(outer));
    for _ in 0..slices.saturating_sub(1) {
        out.then(&linear_kick_map(knl, ksl));
        out.then(&weak_focusing_map(weak_coeff));
        out.then(&drift_map(inner));
    }
    out.then(&linear_kick_map(knl, ksl));
    out.then(&weak_focusing_map(weak_coeff));
    out.then(&drift_map(outer));
    Some(out)
}

fn affine_instruction(
    instruction: &Instruction,
    mat: SMatrix<f64, 5, 5>,
    vec: SVector<f64, 5>,
) -> Affine5 {
    Affine5 {
        mat,
        vec,
        pass_count: usize::from(!instruction.flags.contains(PassFlags::NO_APERTURE_CHECK)),
        names: vec![instruction.name.clone()],
    }
}

fn drift_map(length: f64) -> Affine5 {
    let mut mat = identity5_canonical();
    mat[(0, 2)] = length;
    mat[(1, 3)] = length;
    Affine5 {
        mat,
        vec: SVector::<f64, 5>::zeros(),
        pass_count: 0,
        names: Vec::new(),
    }
}

fn linear_kick_map(knl: &[f64], ksl: &[f64]) -> Affine5 {
    let knl0 = knl.first().copied().unwrap_or(0.0);
    let knl1 = knl.get(1).copied().unwrap_or(0.0);
    let ksl0 = ksl.first().copied().unwrap_or(0.0);
    let ksl1 = ksl.get(1).copied().unwrap_or(0.0);
    let mut mat = identity5_canonical();
    let mut vec = SVector::<f64, 5>::zeros();
    mat[(2, 0)] -= knl1;
    mat[(2, 1)] += ksl1;
    mat[(3, 0)] += ksl1;
    mat[(3, 1)] += knl1;
    vec[2] = -knl0;
    vec[3] = ksl0;
    Affine5 {
        mat,
        vec,
        pass_count: 0,
        names: Vec::new(),
    }
}

fn weak_focusing_map(weak_coeff: f64) -> Affine5 {
    let mut mat = identity5_canonical();
    mat[(2, 0)] -= weak_coeff;
    Affine5 {
        mat,
        vec: SVector::<f64, 5>::zeros(),
        pass_count: 0,
        names: Vec::new(),
    }
}

fn set_plane(mat: &mut SMatrix<f64, 5, 5>, pos: usize, mom: usize, plane: [[f64; 2]; 2]) {
    mat[(pos, pos)] = plane[0][0];
    mat[(pos, mom)] = plane[0][1];
    mat[(mom, pos)] = plane[1][0];
    mat[(mom, mom)] = plane[1][1];
}

fn plane_matrix(k: f64, length: f64) -> [[f64; 2]; 2] {
    if k > 0.0 {
        let k2 = k.sqrt();
        let k2l = length * k2;
        let c = k2l.cos();
        let s = k2l.sin();
        [[c, s / k2], [-s * k2, c]]
    } else if k < 0.0 {
        let k2 = k.abs().sqrt();
        let k2l = length * k2;
        let ch = k2l.cosh();
        let sh = k2l.sinh();
        [[ch, sh / k2], [sh * k2, ch]]
    } else {
        [[1.0, length], [0.0, 1.0]]
    }
}

fn identity5_canonical() -> SMatrix<f64, 5, 5> {
    SMatrix::identity()
}

fn canonical_to_instruction_matrix(mat: &SMatrix<f64, 5, 5>) -> [[f64; 4]; 4] {
    let order = [0usize, 2usize, 1usize, 3usize];
    let mut out = [[0.0; 4]; 4];
    for row in 0..4 {
        for col in 0..4 {
            out[row][col] = mat[(order[row], order[col])];
        }
    }
    out
}
