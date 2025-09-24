
#ifdef data_64
typedef short op_t;
typedef ushort flags_t;
typedef ushort intarg_t;
typedef double float_t;
typedef ulong uint_t;
__constant const float_t FLOAT_MAX = DBL_MAX;
#else
typedef char op_t;
typedef uchar flags_t;
typedef uchar intarg_t;
typedef float float_t;
typedef uint uint_t;
__constant const float_t FLOAT_MAX = FLT_MAX;
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
