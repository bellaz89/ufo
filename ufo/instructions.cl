#include "base.cl"

#ifndef __UFO_INSTRUCTIONS__
#define __UFO_INSTRUCTIONS__

#define OP_ALIGN 0
#define OP_DRIFT 1
#define OP_KICK 2
#define OP_TEAPOT 3
#define OP_QUADRUPOLE 4
#define OP_SBEND 5
#define OP_EDGE 6
#define OP_SEXTUPOLE 7
#define OP_OCTUPOLE 8
#define OP_WIRE 9
#define OP_CAVITY 10
#define OP_TRAV_LINEAR 11
#define OP_SET_APERTURE 12

#define FLAG_LINEAR (0x1 << 0)
#define FLAG_FIVED (0x1 << 1)
#define FLAG_EXACT (0x1 << 2)
#define FLAG_KICK (0x1 << 3)
#define FLAG_RADIATION (0x1 << 4)
#define FLAG_DOUBLE_PRECISION (0x1 << 5)
#define FLAG_ACHROMATIC (0x1 << 6)

#endif

inline void align(particle_work_t* part_data, const flags_t flags,
                  const intarg_t args0, const intarg_t args1,
                  __local const float_t* args0_arr,
                  __local const float_t* args1_arr) {
  UNUSED(flags)
  UNUSED(args0)
  UNUSED(args1)
  UNUSED(args1_arr)

  float_t dx = args0_arr[0];
  float_t dy = args0_arr[1];

  part_data->particle.x -= dx;
  part_data->particle.y -= dy;
}

inline void _drift(particle_work_t* part_data, const flags_t flags,
                   const float_t length) {
  const float_t px0 = part_data->particle.px;
  const float_t py0 = part_data->particle.py;
  const float_t opdp = part_data->opdp;
  float_t eff_length = length;

  if (flags & FLAG_EXACT) {
    eff_length /= sqrt(opdp * opdp - px0 * px0 - py0 * py0);
  }

  part_data->particle.x += px0 * eff_length;
  part_data->particle.y += py0 * eff_length;
}

inline void drift(particle_work_t* part_data, const flags_t flags,
                  const intarg_t args0, const intarg_t args1,
                  __local const float_t* args0_arr,
                  __local const float_t* args1_arr) {
  UNUSED(flags)
  UNUSED(args0)
  UNUSED(args1)
  UNUSED(args1_arr)

  float_t length = args0_arr[0];
  _drift(part_data, flags, length);
}

inline void kick(particle_work_t* part_data, const flags_t flags,
                 const intarg_t args0, const intarg_t args1,
                 __local const float_t* args0_arr,
                 __local const float_t* args1_arr) {
  const intarg_t knl_size = args0;
  const intarg_t ksl_size = args1;
  __local const float_t* knl = args0_arr;
  __local const float_t* ksl = args1_arr;
  intarg_t max_order;
  float_t dpx, dpy, aux;

  const float_t x0 = part_data->particle.x;
  const float_t y0 = part_data->particle.y;
  const float_t oodppo = part_data->oodppo;

  if (knl_size) {
    max_order = knl_size;
    dpx = knl[max_order - 1];
    dpx = (flags & FLAG_ACHROMATIC) ? dpx : dpx * oodppo;
    dpy = 0.0;

// Should suffice for dodecapole
#pragma unroll(5)
    for (intarg_t order = max_order - 1; order > 0; order--) {
      aux = (dpx * x0 - dpy * y0) / order;
      dpy = (dpx * y0 + dpy * x0) / order;
      dpx = knl[order - 1];
      dpx = (flags & FLAG_ACHROMATIC) ? dpx : dpx * oodppo;
      dpx += aux;
    }

    part_data->particle.px += dpx;
    part_data->particle.py += dpy;
  }

  if (ksl_size) {
    max_order = ksl_size;
    dpy = ksl[max_order - 1];
    dpy = (flags & FLAG_ACHROMATIC) ? dpy : dpy * oodppo;
    dpx = 0.0;

// Should suffice for dodecapole
#pragma unroll(5)
    for (intarg_t order = max_order - 1; order > 0; order--) {
      aux = (dpx * y0 + dpy * x0) / order;
      dpx = (dpx * x0 - dpy * y0) / order;
      dpy = ksl[order - 1];
      dpy = (flags & FLAG_ACHROMATIC) ? dpx : dpx * oodppo;
      dpy += aux;
    }

    part_data->particle.px += dpx;
    part_data->particle.py += dpy;
  }
}

inline void teapot(particle_work_t* part_data, const flags_t flags,
                   const intarg_t args0, const intarg_t args1,
                   __local const float_t* args0_arr,
                   __local const float_t* args1_arr) {
  const intarg_t knl_size = args0 - 4;
  const intarg_t ksl_size = args1;
  const float_t inner = args0_arr[0];
  const float_t outer = args0_arr[1];
  const float_t weak_coeff = args0_arr[2];
  const uint slices = (uint)args0_arr[3];

  __local const float_t* knl = args0_arr + 4;
  __local const float_t* ksl = args1_arr;

  // TODO:: suggest pragma unroll
  for (uint i = 0; i < slices - 1; i++) {
    kick(part_data, flags, knl_size, ksl_size, knl, ksl);
    part_data->particle.px -= part_data->particle.x * weak_coeff;
    _drift(part_data, flags, inner);
  }

  kick(part_data, flags, knl_size, ksl_size, knl, ksl);
  part_data->particle.px -= part_data->particle.x * weak_coeff;
  _drift(part_data, flags, outer);
}

inline void quadrupole(particle_work_t* part_data, const flags_t flags,
                       const intarg_t args0, const intarg_t args1,
                       __local const float_t* args0_arr,
                       __local const float_t* args1_arr) {
  UNUSED(flags)
  UNUSED(args0)
  UNUSED(args1)
  UNUSED(args1_arr)

  const float_t oodppo = part_data->oodppo;
  const float_t k =
      (flags & FLAG_ACHROMATIC) ? args0_arr[0] : args0_arr[0] * oodppo;
  const float_t length = args0_arr[1];

  const float_t x0 = part_data->particle.x;
  const float_t y0 = part_data->particle.y;
  const float_t px0 = part_data->particle.px;
  const float_t py0 = part_data->particle.py;

  const float_t k2 = sqrt(fabs(k));
  const float_t k2l = length * k2;
  const float_t C = cos(k2l);
  const float_t CH = cosh(k2l);
  const float_t S = sin(k2l);
  const float_t SH = sinh(k2l);

  if (k > 0.0) {
    part_data->particle.x = C * x0 + S * px0 / k2;
    part_data->particle.y = CH * y0 + SH * py0 / k2;
    part_data->particle.px = -S * x0 * k2 + C * px0;
    part_data->particle.py = SH * y0 * k2 + CH * py0;
  } else {
    part_data->particle.x = CH * x0 + SH * px0 / k2;
    part_data->particle.y = C * y0 + S * py0 / k2;
    part_data->particle.px = SH * x0 * k2 + CH * px0;
    part_data->particle.py = -S * y0 * k2 + C * py0;
  }
}

inline void sbend(particle_work_t* part_data, const flags_t flags,
                  const intarg_t args0, const intarg_t args1,
                  __local const float_t* args0_arr,
                  __local const float_t* args1_arr) {
  UNUSED(flags)
  UNUSED(args0)
  UNUSED(args1)
  UNUSED(args1_arr)

  const float_t oodppo = part_data->oodppo;
  const float_t length = args0_arr[0];
  const float_t curvature_coeff = args0_arr[1];
  const float_t k1 = args0_arr[2];

  const float_t x0 = part_data->particle.x;
  const float_t y0 = part_data->particle.y;
  const float_t px0 = part_data->particle.px;
  const float_t py0 = part_data->particle.py;
  const float_t dp0 = part_data->particle.dp;

  const float_t curvature =
      (flags & FLAG_ACHROMATIC) ? curvature_coeff : curvature_coeff * oodppo;

  float_t k = (flags & FLAG_ACHROMATIC) ? k1 : k1 * oodppo;
  float_t k2 = sqrt(fabs(k));
  float_t k2l = length * k2;
  float_t C = cos(k2l);
  float_t CH = cosh(k2l);
  float_t S = sin(k2l);
  float_t SH = sinh(k2l);

  if (k > 0.) {
    part_data->particle.y = CH * y0 + SH * py0 / k2;
    part_data->particle.py = SH * y0 * k2 + CH * py0;
  }

  if (k < 0.) {
    part_data->particle.y = C * y0 + S * py0 / k2;
    part_data->particle.py = -S * y0 * k2 + C * py0;
  }

  if (k == 0.) {
    part_data->particle.y += length * py0;
  }

  k += (flags & FLAG_ACHROMATIC) ? curvature * curvature
                                 : curvature * curvature_coeff;
  k2 = sqrt(fabs(k));
  k2l = length * k2;
  C = cos(k2l);
  CH = cosh(k2l);
  S = sin(k2l);
  SH = sinh(k2l);

  if (k > 0.0) {
    part_data->particle.x = C * x0 + S * px0 / k2;
    part_data->particle.x += dp0 * curvature * (1. - C) / fabs(k);
    part_data->particle.px = -S * x0 * k2 + C * px0;
    part_data->particle.px += dp0 * curvature * S / k2;
  }

  if (k < 0.0) {
    part_data->particle.x = CH * x0 + SH * px0 / k2;
    part_data->particle.x += dp0 * curvature * (CH - 1.) / fabs(k);
    part_data->particle.px = SH * x0 * k2 + CH * px0;
    part_data->particle.px += dp0 * curvature * SH / k2;
  }

  if (k == 0.0) {
    part_data->particle.x += length * px0;
  }
}

inline void edge(particle_work_t* part_data, const flags_t flags,
                 const intarg_t args0, const intarg_t args1,
                 __local const float_t* args0_arr,
                 __local const float_t* args1_arr) {
  UNUSED(flags)
  UNUSED(args0)
  UNUSED(args1)
  UNUSED(args1_arr)

  const float_t oodppo = part_data->oodppo;

  const float_t x0 = part_data->particle.x;
  const float_t y0 = part_data->particle.y;

  const float_t angle_length_ratio = args0_arr[0];
  const float_t e = args0_arr[1];
  const float_t tan_e = args0_arr[2];
  const float_t psi_coeff = args0_arr[3];

  const float_t curvature = (flags & FLAG_ACHROMATIC)
                                ? angle_length_ratio
                                : angle_length_ratio * oodppo;
  const float_t psi = e - curvature * psi_coeff;

  part_data->particle.px += x0 * curvature * tan_e;
  part_data->particle.py -= y0 * curvature * tan(psi);
}

inline void wire(particle_work_t* part_data, const flags_t flags,
                 const intarg_t args0, const intarg_t args1,
                 __local const float_t* args0_arr,
                 __local const float_t* args1_arr) {
  UNUSED(flags)
  UNUSED(args0)
  UNUSED(args1)
  UNUSED(args1_arr)

  const float_t k = args0_arr[0];
  const float_t wire_x = args0_arr[1];
  const float_t wire_y = args0_arr[2];

  const float_t x0 = part_data->particle.x;
  const float_t y0 = part_data->particle.y;

  const float_t alpha = atan2(wire_x - y0, wire_y - x0);
  const float_t B = k / hypot(wire_x - x0, wire_y - y0);

  part_data->particle.px += B * cos(alpha);
  part_data->particle.py += B * sin(alpha);
}

inline void cavity(particle_work_t* part_data, const flags_t flags,
                   const intarg_t args0, const intarg_t args1,
                   __local const float_t* args0_arr,
                   __local const float_t* args1_arr) {
  UNUSED(flags)
  UNUSED(args0)
  UNUSED(args1)
  UNUSED(args1_arr)

  const float_t z0 = part_data->particle.z;
  const float_t field = args0_arr[0];
  const float_t omega = args0_arr[1];
  const float_t lag = args0_arr[2];
  part_data->particle.dp += field * sin((2.0 * M_PI) * (lag - omega * z0));
  update_oodppo(part_data);
}

// NOTE: This uses a different flag notation
// Generic 4x4 linear xypxpy phase space transform plus shift
inline void trav_linear(particle_work_t* part_data, const flags_t flags,
                        const intarg_t args0, const intarg_t args1,
                        __local const float_t* args0_arr,
                        __local const float_t* args1_arr) {
  UNUSED(args0)
  UNUSED(args1)
  UNUSED(args1_arr)

  const float_t x0 = part_data->particle.x;
  const float_t y0 = part_data->particle.y;
  const float_t px0 = part_data->particle.px;
  const float_t py0 = part_data->particle.py;
  uint slice = 0;

  if (flags & (0x1 << 0)) {
    part_data->particle.x += args0_arr[slice + 0];
    part_data->particle.y += args0_arr[slice + 1];
    part_data->particle.px += args0_arr[slice + 2];
    part_data->particle.py += args0_arr[slice + 3];
    slice += 4;
  }

  if (flags & (0x1 << 1)) {
    part_data->particle.x +=
        x0 * args0_arr[slice + 0] + y0 * args0_arr[slice + 1];
    part_data->particle.y +=
        x0 * args0_arr[slice + 2] + y0 * args0_arr[slice + 3];
    slice += 4;
  }

  if (flags & (0x1 << 2)) {
    part_data->particle.x +=
        px0 * args0_arr[slice + 0] + py0 * args0_arr[slice + 1];
    part_data->particle.y +=
        px0 * args0_arr[slice + 2] + py0 * args0_arr[slice + 3];
    slice += 4;
  }

  if (flags & (0x1 << 3)) {
    part_data->particle.px +=
        x0 * args0_arr[slice + 0] + y0 * args0_arr[slice + 1];
    part_data->particle.py +=
        x0 * args0_arr[slice + 2] + y0 * args0_arr[slice + 3];
    slice += 4;
  }

  if (flags & (0x1 << 4)) {
    part_data->particle.px +=
        px0 * args0_arr[slice + 0] + py0 * args0_arr[slice + 1];
    part_data->particle.py +=
        px0 * args0_arr[slice + 2] + py0 * args0_arr[slice + 3];
    slice += 4;
  }
}

// NOTE: This uses a different flag notation
inline void set_aperture(particle_work_t* part_data, const flags_t flags,
                         const intarg_t args0, const intarg_t args1,
                         __local const float_t* args0_arr,
                         __local const float_t* args1_arr) {
  UNUSED(flags)
  UNUSED(args0)
  UNUSED(args1)
  UNUSED(args1_arr)

  part_data->aperture_sq = args0_arr[0];
}
