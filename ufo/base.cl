#ifndef __UFO_BASE__
#define __UFO_BASE__

#ifndef __func__

#define __func__ "<unknown>"

#endif

#define UNUSED(x) (void)(x);

#ifdef UFO_NDEBUG
#define UFO_DEBUG(...) ((void)0);
#define UFO_ASSERT(test, ...) ((void)0);
#else
#define UFO_DEBUG(fmt_str, ...)                                              \
  do {                                                                       \
    if (get_global_id(0) == 0) {                                             \
      printf("INFO(%s:%s-l%d): " fmt_str "\n", __func__, __FILE__, __LINE__, \
             __VA_ARGS__);                                                   \
    }                                                                        \
  } while (false);

#define UFO_ASSERT(test, fmt_str, ...)                                \
  do {                                                                \
    if (!(test)) {                                                    \
      if (get_global_id(0) == 0) {                                    \
        printf("ERROR(%s:%s-l%d): " fmt_str "\n", __func__, __FILE__, \
               __LINE__, __VA_ARGS__);                                \
      }                                                               \
      while (true) {                                                  \
      };                                                              \
    }                                                                 \
  } while (false);

#endif

#define INNER_CAT(a, b) a##b
#define CAT(a, b) INNER_CAT(a, b)

#ifdef UFO64
#ifndef UFO_NDEBUG

#pragma message("#INFO: base.cl: using 64bit bytecode")
#endif
typedef short op_t;
typedef ushort flags_t;
typedef ushort intarg_t;
typedef double float_t;
typedef ulong uint_t;
__constant const float_t FLOAT_MAX = DBL_MAX;
#else
#ifndef UFO_NDEBUG

#pragma message("#INFO: base.cl: using 32bit bytecode")

#endif
typedef char op_t;
typedef uchar flags_t;
typedef uchar intarg_t;
typedef float float_t;
typedef uint uint_t;
__constant const float_t FLOAT_MAX = FLT_MAX;
#endif

#ifdef UFO_NATIVE
#ifndef UFO_NDEBUG
#pragma message("#INFO: base.cl: using native instructions")
#endif
#define _sqrt(x) (native_sqrt(x))
#define _sin(x) (native_sin(x))
#define _cos(x) (native_cos(x))
#define _tan(x) (native_tan(x))
#define _rsqrt(x) (native_rsqrt(x))
#define _recip(x) (native_recip(x))
#define _divide(x, y) (native_divide(x, y))
#define _hypot(x, y) (native_sqrt(x * x + y * y))
#define _atan2(x, y) (atan2(x, y))
#define _cosh(x) (0.5 * (native_exp(x) + native_exp(-x)))
#define _sinh(x) (0.5 * (native_exp(x) - native_exp(-x)))

#else
#ifndef UFO_NDEBUG

#pragma message("#INFO: base.cl: using standard instructions")
#endif
#define _sqrt(x) (sqrt(x))
#define _sin(x) (sin(x))
#define _cos(x) (cos(x))
#define _tan(x) (tan(x))
#define _rsqrt(x) (rsqrt(x))
#define _recip(x) (1.0 / x)
#define _divide(x, y) (x / y)
#define _hypot(x, y) (hypot(x, y))
#define _atan2(x, y) (atan2(x, y))
#define _cosh(x) (cosh(x))
#define _sinh(x) (sinh(x))
#endif

// Instruction type
typedef struct {
  op_t op;
  flags_t flags;
  intarg_t args0;
  intarg_t args1;
} inst_t;

// Particle type
typedef struct {
  float_t x, y, z;
  float_t px, py, dp;
  uint passed_elements;
  bool alive;
} particle_t;

// Data structure that holds all particle-related variable data of a single
// thread
typedef struct {
  particle_t particle;
  float_t opdp;
  float_t oodppo;
  float_t aperture_sq;
} particle_work_t;

// Updates oodppo and opdp using particle's dp
inline void update_oodppo(particle_work_t* part_data) {
  part_data->opdp = part_data->particle.dp + 1.0;
  part_data->oodppo = 1.0 / (part_data->opdp);
}
#endif
