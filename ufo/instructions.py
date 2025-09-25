import struct

import numpy as np

OP_ALIGN = 0
OP_DRIFT = 1
OP_KICK = 2
OP_TEAPOT = 3
OP_QUADRUPOLE = 4
OP_SBEND = 5
OP_EDGE = 6
OP_SEXTUPOLE = 7
OP_OCTUPOLE = 8
OP_WIRE = 9
OP_CAVITY = 10
OP_TRAV_LINEAR = 11
OP_SET_APERTURE = 12

FLAG_LINEAR = 0x1 << 0
FLAG_FIVED = 0x1 << 1
FLAG_EXACT = 0x1 << 2
FLAG_KICK = 0x1 << 3
FLAG_RADIATION = 0x1 << 4
FLAG_DOUBLE_PRECISION = 0x1 << 5
FLAG_ACHROMATIC = 0x1 << 6


class Instruction:
    def __init__(self, op, flags, args0_arr, args1_arr=[]):
        self.op = op
        self.flags = flags
        self.args0_arr = args0_arr
        self.args1_arr = args1_arr

    def emit_bytecode(self, is_64bit=False):
        op_t = "H" if is_64bit else "B"
        flags_t = "H" if is_64bit else "B"
        intargs_t = "H" if is_64bit else "B"
        float_t = "d" if is_64bit else "f"

        float_conv = np.float64 if is_64bit else np.float32
        args0_arr = [float_conv(f) for f in self.args0_arr]
        args1_arr = [float_conv(f) for f in self.args1_arr]
        args0 = len(args0_arr)
        args1 = len(args1_arr)
        inst_format = "<" + op_t + flags_t + 2 * intargs_t + (args0 + args1) * float_t
        return struct.pack(
            inst_format, self.op, self.flags, args0, args1, *args0_arr, *args1_arr
        )


class TravLinear(Instruction):
    def __init__(self, mat, vec):
        self.mat = np.array(mat)
        self.vec = np.array(vec)
        mat_m_eye = np.array(mat) - np.eye(4)
        flags = 0
        args0_arr = []

        flag = 0x1

        if vec != np.zeros(4):
            flags |= flag
            args0_arr.update(list(vec))

        for y in [0, 2]:
            for x in [0, 2]:
                flag <<= 1
                sub_mat = mat_m_eye[y : y + 2, x : x + 2]

                if sub_mat != np.zeros((2, 2)):
                    flags |= flag
                    args0_arr.update(list(sub_mat[0, 0:1]))
                    args0_arr.update(list(sub_mat[1, 0:1]))

        super().__init__(OP_TRAV_LINEAR, flags, args0_arr)

    # Compose two TravLinear instructions with new = prec <= succ syntax
    def __le__(self, other):
        mat = other.mat @ self.mat
        vec = other.mat @ self.vec + other.vec
        return TravLinear(mat, vec)


def trav_linear(mat, vec):
    return TravLinear(mat, vec)


def align(flags, dx, dy, use_linear=True):
    if use_linear:
        return TravLinear(np.eye(4), [dx, dy, 0.0, 0.0])
    else:
        return Instruction(OP_ALIGN, flags, [dx.dy])


def drift(flags, length):
    return Instruction(OP_DRIFT, flags, [length])


def kick(flags, knl, ksl):
    pass
