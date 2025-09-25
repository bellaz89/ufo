import struct

import numpy as np

OP_REWIND = -1
OP_NEXT_OFFSET = -2
OP_DUMP = -3
OP_DRIFT = 1
OP_KICK = 2
OP_TEAPOT = 3
OP_SBEND = 5
OP_EDGE = 6
OP_OCTUPOLE = 8
OP_WIRE = 9
OP_CAVITY = 10
OP_TRAN_LINEAR = 11
OP_SET_APERTURE = 12

FLAG_LINEAR = 0x1 << 0
FLAG_FIVED = 0x1 << 1
FLAG_EXACT = 0x1 << 2
FLAG_KICK = 0x1 << 3
FLAG_RADIATION = 0x1 << 4
FLAG_DOUBLE_PRECISION = 0x1 << 5
FLAG_ACHROMATIC = 0x1 << 6
FLAG_NO_APERTURE_CHECK = 0x1 << 7

FLAGS = {}
FLAGS[FLAG_LINEAR] = "LINEAR"
FLAGS[FLAG_FIVED] = "FIVED"
FLAGS[FLAG_EXACT] = "EXACT"
FLAGS[FLAG_KICK] = "KICK"
FLAGS[FLAG_RADIATION] = "RADIATION"
FLAGS[FLAG_DOUBLE_PRECISION] = "DOUBLE_PRECISION"
FLAGS[FLAG_ACHROMATIC] = "ACHROMATIC"
FLAGS[FLAG_NO_APERTURE_CHECK] = "NO_APERTURE_CHECK"

FLAG_TRAN_LINEAR_VEC = 0x1 << 0
FLAG_TRAN_LINEAR_MAT_XX = 0x1 << 1
FLAG_TRAN_LINEAR_MAT_PXX = 0x1 << 2
FLAG_TRAN_LINEAR_MAT_XPX = 0x1 << 3
FLAG_TRAN_LINEAR_MAT_PXPX = 0x1 << 4

FLAGS_TRAN_LINEAR = {}
FLAGS_TRAN_LINEAR[FLAG_TRAN_LINEAR_VEC] = "VEC"
FLAGS_TRAN_LINEAR[FLAG_TRAN_LINEAR_MAT_XX] = "XX"
FLAGS_TRAN_LINEAR[FLAG_TRAN_LINEAR_MAT_PXX] = "PXX"
FLAGS_TRAN_LINEAR[FLAG_TRAN_LINEAR_MAT_XPX] = "XPX"
FLAGS_TRAN_LINEAR[FLAG_TRAN_LINEAR_MAT_PXPX] = "PXPX"
FLAGS_TRAN_LINEAR[FLAG_NO_APERTURE_CHECK] = "NO_APERTURE_CHECK"

CODE32_FORMAT = ("<BBBB", "f")
CODE64_FORMAT = ("<HHHH", "d")

def format_flags_generic(flags, flags_dict):
    identifiers = [
        identifier for flag, identifier in flags_dict.items() if identifier & flags
    ]
    return "|".join(identifiers)


def format_flags(flags):
    return format_flags_generic(flags, FLAGS)


def format_flags_trav_linear(flags):
    return format_flags_generic(flags, FLAGS_TRAN_LINEAR)


def pack(code_format, op, flags, args0_arr, args1_arr):
    _format = code_format[0] + code_format[1] * (len(args0_arr) + len(args1_arr))
    return struct.pack(
        op, flags, len(args0_arr), len(args1_arr), *args0_arr, *args1_arr
    )


def pack32(op, flags, args0_arr, args1_arr):
    return pack(CODE32_FORMAT, op, flags, args0_arr, args1_arr)


def pack64(op, flags, args0_arr, args1_arr):
    return pack(CODE32_FORMAT, op, flags, args0_arr, args1_arr)


class Instruction:
    def __init__(
        self, op, flags, args0_arr, args1_arr=[], name="unknown", itype="unknown"
    ):
        self._name = name
        self._type = itype
        self._op = op
        self._flags = flags
        self._args0_arr = args0_arr
        self._args1_arr = args1_arr

    @property
    def name(self):
        return self._name

    @property
    def type(self):
        return self._type

    @property
    def op(self):
        return self._op

    @property
    def flags(self):
        return self._flags

    @property
    def args0(self):
        return len(self._args0_arr)

    @property
    def args1(self):
        return len(self._args1_arr)

    @property
    def args0_arr(self):
        return len(self._args0_arr)

    @property
    def args1_arr(self):
        return len(self._args1_arr)

    def __repr__(self):
        return (
            f"(instruction name: {self.name}, "
            f"type: {self.type}({self.op}), "
            f"flags: {format_flags(self.flags)}, "
            f"args0: {self.args0}, "
            f"args1: {self.args1})"
        )

    def emit_bytecode(self, is_64bit=False):
        if is_64bit:
            return pack64(self.op, self.flags, self.args0_arr, self.args1_arr)
        else:
            return pack32(self.op, self.flags, self.args0_arr, self.args1_arr)

class 

class TranLinear(Instruction):
    def __init__(self, mat, vec, name="unknown"):
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

        super().__init__(
            OP_TRAN_LINEAR, flags, args0_arr, name=name, itype="transverse_linear"
        )

    # Compose two TranLinear instructions with new = prec <= succ syntax
    def __le__(self, other):
        assert (
            self.flags & FLAG_NO_APERTURE_CHECK
        ), "cannot fuse to self since self must check for aperture"
        assert (
            other.flags & FLAG_NO_APERTURE_CHECK
        ), "cannot fuse to other since other must check for aperture"
        mat = other.mat @ self.mat
        vec = other.mat @ self.vec + other.vec
        return TranLinear(mat, vec)



def trav_linear(mat, vec):
    return TranLinear(mat, vec)


def align(flags, dx, dy, use_linear=True):
    if use_linear:
        return TranLinear(np.eye(4), [dx, dy, 0.0, 0.0])
    else:
        return Instruction(OP_ALIGN, flags, [dx.dy])


def drift(flags, length):
    return Instruction(OP_DRIFT, flags, [length])


def kick(flags, knl, ksl):
    pass
