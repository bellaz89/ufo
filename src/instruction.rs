use bytemuck::{Pod, Zeroable};

use crate::{PassFlags, Result, UfoError};

pub const OP_REWIND: u16 = 255;
pub const OP_NEXT_OFFSET: u16 = 254;
pub const OP_DUMP: u16 = 253;
pub const OP_DRIFT: u16 = 1;
pub const OP_KICK: u16 = 2;
pub const OP_TEAPOT: u16 = 3;
pub const OP_QUADRUPOLE: u16 = 4;
pub const OP_SBEND: u16 = 5;
pub const OP_EDGE: u16 = 6;
pub const OP_WIRE: u16 = 9;
pub const OP_CAVITY: u16 = 10;
pub const OP_TRAN_LINEAR: u16 = 11;
pub const OP_SET_APERTURE: u16 = 12;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
pub struct Inst32 {
    pub op: u8,
    pub flags: u8,
    pub aux: u8,
    pub argc: u8,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
pub struct Inst64 {
    pub op: u16,
    pub flags: u16,
    pub aux: u16,
    pub argc: u16,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Instruction {
    pub name: String,
    pub kind: String,
    pub op: u16,
    pub flags: PassFlags,
    pub aux: u16,
    pub args: Vec<f64>,
}

impl Instruction {
    pub fn new(
        name: impl Into<String>,
        kind: impl Into<String>,
        op: u16,
        flags: PassFlags,
        aux: u16,
        args: Vec<f64>,
    ) -> Self {
        Self {
            name: name.into(),
            kind: kind.into(),
            op,
            flags,
            aux,
            args,
        }
    }

    pub fn rewind(name: impl Into<String>) -> Self {
        Self::new(
            name,
            "rewind",
            OP_REWIND,
            PassFlags::NO_APERTURE_CHECK,
            0,
            Vec::new(),
        )
    }

    pub fn next_offset(name: impl Into<String>) -> Self {
        Self::new(
            name,
            "next_offset",
            OP_NEXT_OFFSET,
            PassFlags::NO_APERTURE_CHECK,
            0,
            Vec::new(),
        )
    }

    pub fn dump(name: impl Into<String>) -> Self {
        Self::new(
            name,
            "dump",
            OP_DUMP,
            PassFlags::NO_APERTURE_CHECK,
            0,
            Vec::new(),
        )
    }

    pub fn drift(name: impl Into<String>, length: f64, exact: bool) -> Self {
        let mut flags = PassFlags::empty();
        if exact {
            flags |= PassFlags::EXACT;
        }
        Self::new(name, "drift", OP_DRIFT, flags, 0, vec![length])
    }

    pub fn kick(
        name: impl Into<String>,
        mut knl: Vec<f64>,
        mut ksl: Vec<f64>,
        linear: bool,
        achromatic: bool,
    ) -> Self {
        let mut flags = PassFlags::NO_APERTURE_CHECK;
        if linear {
            flags |= PassFlags::LINEAR;
            knl.truncate(2);
            ksl.truncate(2);
        }
        if achromatic {
            flags |= PassFlags::ACHROMATIC;
        }
        if knl.len() <= 2 && ksl.len() <= 2 {
            flags |= PassFlags::LINEAR;
        }
        let aux = knl.len() as u16;
        knl.extend(ksl);
        Self::new(name, "kick", OP_KICK, flags, aux, knl)
    }

    pub fn linear_kick(
        name: impl Into<String>,
        knl: &[f64],
        ksl: &[f64],
        no_aperture_check: bool,
    ) -> Self {
        let knl0 = knl.first().copied().unwrap_or(0.0);
        let knl1 = knl.get(1).copied().unwrap_or(0.0);
        let ksl0 = ksl.first().copied().unwrap_or(0.0);
        let ksl1 = ksl.get(1).copied().unwrap_or(0.0);
        let mut mat = identity4();
        mat[1][0] -= knl1;
        mat[1][2] += ksl1;
        mat[3][0] += ksl1;
        mat[3][2] += knl1;
        Self::tran_linear_with_options(
            name,
            mat,
            [0.0, 0.0, -knl0, ksl0],
            [0.0; 4],
            no_aperture_check,
            false,
        )
    }

    pub fn teapot(
        name: impl Into<String>,
        inner: f64,
        outer: f64,
        weak_coeff: f64,
        slices: f64,
        knl: Vec<f64>,
        ksl: Vec<f64>,
        flags: PassFlags,
    ) -> Self {
        let aux = knl.len() as u16;
        Self::new(
            name,
            "teapot",
            OP_TEAPOT,
            flags,
            aux,
            [vec![inner, outer, weak_coeff, slices], knl, ksl].concat(),
        )
    }

    pub fn quadrupole(name: impl Into<String>, length: f64, k1: f64, achromatic: bool) -> Self {
        let mut flags = PassFlags::empty();
        if achromatic {
            flags |= PassFlags::ACHROMATIC;
        }
        Self::new(
            name,
            "quadrupole",
            OP_QUADRUPOLE,
            flags,
            0,
            vec![k1, length],
        )
    }

    pub fn sbend(
        name: impl Into<String>,
        length: f64,
        angle: f64,
        k1: f64,
        achromatic: bool,
    ) -> Self {
        let mut flags = PassFlags::empty();
        if achromatic {
            flags |= PassFlags::ACHROMATIC;
        }
        Self::new(
            name,
            "sbend",
            OP_SBEND,
            flags,
            0,
            vec![length, angle / length, k1],
        )
    }

    pub fn edge(
        name: impl Into<String>,
        length: f64,
        angle: f64,
        edge_angle: f64,
        fringe: f64,
        achromatic: bool,
    ) -> Self {
        let mut flags = PassFlags::empty();
        if achromatic {
            flags |= PassFlags::ACHROMATIC;
        }
        let angle_length_ratio = angle / length;
        let psi_coeff =
            2.0 * fringe / edge_angle.cos() * (1.0 + edge_angle.sin() * edge_angle.sin());
        Self::new(
            name,
            "edge",
            OP_EDGE,
            flags,
            0,
            vec![angle_length_ratio, edge_angle, edge_angle.tan(), psi_coeff],
        )
    }

    pub fn wire(name: impl Into<String>, x: f64, y: f64, k: f64) -> Self {
        Self::new(
            name,
            "wire",
            OP_WIRE,
            PassFlags::NO_APERTURE_CHECK,
            0,
            vec![k, x, y],
        )
    }

    pub fn cavity(name: impl Into<String>, field: f64, omega: f64, lag: f64) -> Self {
        Self::new(
            name,
            "cavity",
            OP_CAVITY,
            PassFlags::empty(),
            0,
            vec![field, omega, lag],
        )
    }

    pub fn tran_linear(
        name: impl Into<String>,
        mat: [[f64; 4]; 4],
        vec: [f64; 4],
        no_aperture_check: bool,
    ) -> Self {
        Self::tran_linear_with_options(
            name,
            mat,
            vec,
            [0.0; 4],
            no_aperture_check,
            no_aperture_check,
        )
    }

    pub fn tran_linear_dp(
        name: impl Into<String>,
        mat: [[f64; 4]; 4],
        vec: [f64; 4],
        dp_col: [f64; 4],
        no_aperture_check: bool,
    ) -> Self {
        Self::tran_linear_with_options(name, mat, vec, dp_col, no_aperture_check, no_aperture_check)
    }

    pub(crate) fn tran_linear_with_options(
        name: impl Into<String>,
        mat: [[f64; 4]; 4],
        vec: [f64; 4],
        dp_col: [f64; 4],
        no_aperture_check: bool,
        no_pass: bool,
    ) -> Self {
        const FLAG_VEC: u8 = 0x1 << 0;
        const FLAG_MAT_XX: u8 = 0x1 << 1;
        const FLAG_MAT_PXX: u8 = 0x1 << 2;
        const FLAG_MAT_XPX: u8 = 0x1 << 3;
        const FLAG_MAT_PXPX: u8 = 0x1 << 4;
        const FLAG_DP: u8 = 0x1 << 5;
        const FLAG_NO_PASS: u8 = 0x1 << 6;

        let mut bits = 0u8;
        let mut args = Vec::new();
        if vec.iter().any(|v| *v != 0.0) {
            bits |= FLAG_VEC;
            args.extend_from_slice(&vec);
        }

        for (flag, rows, cols) in [
            (FLAG_MAT_XX, [0usize, 2usize], [0usize, 2usize]),
            (FLAG_MAT_PXX, [0usize, 2usize], [1usize, 3usize]),
            (FLAG_MAT_XPX, [1usize, 3usize], [0usize, 2usize]),
            (FLAG_MAT_PXPX, [1usize, 3usize], [1usize, 3usize]),
        ] {
            let mut block = [0.0; 4];
            let mut changed = false;
            for (r_idx, row) in rows.into_iter().enumerate() {
                for (c_idx, col) in cols.into_iter().enumerate() {
                    let identity = if row == col { 1.0 } else { 0.0 };
                    let value = mat[row][col] - identity;
                    block[r_idx * 2 + c_idx] = value;
                    changed |= value != 0.0;
                }
            }
            if changed {
                bits |= flag;
                args.extend_from_slice(&block);
            }
        }

        if dp_col.iter().any(|v| *v != 0.0) {
            bits |= FLAG_DP;
            args.extend_from_slice(&dp_col);
        }

        if no_aperture_check {
            bits |= PassFlags::NO_APERTURE_CHECK.bits();
        }
        if no_pass {
            bits |= FLAG_NO_PASS;
        }
        Self::new(
            name,
            "tran_linear",
            OP_TRAN_LINEAR,
            PassFlags::from_bits_retain(bits),
            0,
            args,
        )
    }

    pub fn noop(name: impl Into<String>) -> Self {
        const FLAG_NO_PASS: u8 = 0x1 << 6;
        Self::new(
            name,
            "noop",
            OP_TRAN_LINEAR,
            PassFlags::from_bits_retain(PassFlags::NO_APERTURE_CHECK.bits() | FLAG_NO_PASS),
            0,
            Vec::new(),
        )
    }

    pub fn set_aperture(name: impl Into<String>, radius: f64) -> Self {
        Self::new(
            name,
            "set_aperture",
            OP_SET_APERTURE,
            PassFlags::NO_APERTURE_CHECK,
            0,
            vec![radius * radius],
        )
    }

    pub fn encoded_word_count(&self) -> usize {
        1 + self.args.len()
    }

    pub fn encode32(&self) -> Result<Vec<u8>> {
        if self.args.len() > u8::MAX as usize {
            return Err(UfoError::TooManyInstructionArgs {
                name: self.name.clone(),
                count: self.args.len(),
                max: u8::MAX as usize,
            });
        }
        if self.aux > u8::MAX as u16 {
            return Err(UfoError::TooManyInstructionArgs {
                name: self.name.clone(),
                count: self.aux as usize,
                max: u8::MAX as usize,
            });
        }
        let header = Inst32 {
            op: self.op as u8,
            flags: self.flags.bits(),
            aux: self.aux as u8,
            argc: self.args.len() as u8,
        };
        let mut bytes = bytemuck::bytes_of(&header).to_vec();
        for value in &self.args {
            bytes.extend_from_slice(&(*value as f32).to_le_bytes());
        }
        Ok(bytes)
    }

    pub fn encode64(&self) -> Result<Vec<u8>> {
        if self.args.len() > u16::MAX as usize {
            return Err(UfoError::TooManyInstructionArgs {
                name: self.name.clone(),
                count: self.args.len(),
                max: u16::MAX as usize,
            });
        }
        let header = Inst64 {
            op: self.op,
            flags: self.flags.bits() as u16,
            aux: self.aux,
            argc: self.args.len() as u16,
        };
        let mut bytes = bytemuck::bytes_of(&header).to_vec();
        for value in &self.args {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        Ok(bytes)
    }
}

fn identity4() -> [[f64; 4]; 4] {
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}
