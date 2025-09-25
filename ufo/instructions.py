import struct

import numpy as np

OP_REWIND = -1
OP_NEXT_OFFSET = -2
OP_DUMP = -3
OP_DRIFT = 1
OP_KICK = 2
OP_TEAPOT = 3
OP_QUADRUPOLE = 4
OP_SBEND = 5
OP_EDGE = 6
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


def format_flags_tran_linear(flags):
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
        self, op, flags=0, args0_arr=[], args1_arr=[], name="unknown", itype="unknown"
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


class Rewind(Instruction):
    def __init__(self, name):
        super().__init__(
            OP_REWIND, flags=FLAG_NO_APERTURE_CHECK, name=name, itype="rewind"
        )


class NextOffset(Instruction):
    def __init__(self, name):
        super().__init__(
            OP_NEXT_OFFSET, flags=FLAG_NO_APERTURE_CHECK, name=name, itype="next_offset"
        )


class Dump(Instruction):
    def __init__(self, name):
        super().__init__(OP_DUMP, flags=FLAG_NO_APERTURE_CHECK, name=name, itype="dump")


class Drift(Instruction):
    def __init__(self, name, length, exact=True, no_aperture_check=False):
        self.length = length
        flags = FLAG_EXACT if exact else 0
        flags |= FLAG_NO_APERTURE_CHECK if no_aperture_check else 0
        super().__init__(
            OP_DUMP, flags=flags, args0_arr=[length], name=name, itype="drift"
        )


class Kick(Instruction):
    def __init__(self, name, knl, ksl, linear=False, achromatic=False):
        flags = FLAG_NO_APERTURE_CHECK
        flags |= FLAG_LINEAR if linear else 0
        flags |= FLAG_ACHROMATIC if achromatic else 0

        n = len(knl)  # Avoid non-linear terms when LINEAR
        self.max_order_knl = min(n, 2) if linear else n

        n = len(ksl)  # Avoid non-linear terms when LINEAR
        self.max_order_ksl = min(n, 2) if linear else n

        if (self.max_order_knl <= 2) and (self.max_order_ksl <= 2):
            flags |= FLAG_LINEAR

        self.knl = knl[: self.max_order_knl]
        self.ksl[: self.max_order_ksl]

        super().__init__(
            OP_KICK,
            flags=flags,
            args0_arr=self.knl,
            args1_arr=self.ksl,
            name=name,
            itype="kick",
        )


class Teapot(Instruction):
    def __init__(
        self,
        name,
        length,
        slices,
        knl,
        ksl,
        angle=0,
        exact=True,
        linear=False,
        achromatic=False,
        no_aperture_check=False,
    ):
        flags = FLAG_NO_APERTURE_CHECK if no_aperture_check else 0
        flags |= FLAG_EXACT if exact else 0
        flags |= FLAG_LINEAR if linear else 0
        flags |= FLAG_ACHROMATIC if achromatic else 0

        self.inner = (length - 2.0 * self.outer) / (slices - 1.0) if slices > 1 else 0
        self.outer = 0.5 * length if slices == 1 else 0.5 * length / (1 + slices)
        self.weak_coeff = angle**2 / (slices * length)
        self.slices = float(slices)

        self.knl = [k * length / slices for k in knl]
        args0_arr = [self.inner, self.outer, self.weak_coeff, self.slices] + self.knl
        self.ksl = [k * length / slices for k in ksl]

        flags |= Kick(name, self.knl, self.ksl).flags

        super().__init__(
            OP_TEAPOT,
            flags=flags,
            args0_arr=args0_arr,
            args1_arr=self.ksl,
            name=name,
            itype="kick",
        )


class Quadrupole(Instruction):
    def __init__(
        self,
        name,
        length,
        k,
        achromatic=False,
        no_aperture_check=False,
    ):

        flags = FLAG_NO_APERTURE_CHECK if no_aperture_check else 0
        flags |= FLAG_ACHROMATIC if achromatic else 0

        super().__init__(
            OP_QUADRUPOLE, flags, [k, length], name=name, itype="quadrupole"
        )


# class Sbend(Instruction):
# class Edge(Instruction):
# class Wire(Instruction):
# class Cavity(Instruction):


class TranLinear(Instruction):

    def __init__(self, name, mat, vec):
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
            OP_TRAN_LINEAR, flags, args0_arr, name=name, itype="tran_linear"
        )

    def __repr__(self):
        return (
            f"(instruction name: {self.name}, "
            f"type: {self.type}({self.op}), "
            f"flags: {format_flags_tran_linear(self.flags)}, "
            f"args0: {self.args0}, "
            f"args1: {self.args1})"
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


# class SetAperture:
#
#
# def tran_linear(mat, vec):
#    return TranLinear(mat, vec)
#
#
# def drift(flags, length):
#    return Instruction(OP_DRIFT, flags, [length])
#
#
# def kick(flags, knl, ksl):
#    pass
