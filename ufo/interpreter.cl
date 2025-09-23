
typedef uchar inst_buf_t;
typedef ulong2 inst_load_t;

#ifdef inst_64
typedef short op_t;
typedef ushort flags_t;
typedef ushort intarg_t;
#else
typedef char op_t;
typedef uchar flags_t;
typedef uchar intarg_t;
#endif

typedef struct { op_t op; 
                 flags_t flags; 
                 intarg_t arg0;
                 intarg_t arg1; } inst_t;

#define OP_REWIND      -1
#define OP_NEXT_OFFSET -2
#define OP_ALIGN        0
#define OP_DRIFT        1
#define OP_KICK         2
#define OP_TEAPOT       3
#define OP_QUADRUPOLE   4
#define OP_SBEND        5
#define OP_EDGE         6
#define OP_SEXTUPOLE    7
#define OP_OCTUPOLE     8
#define OP_WIRE         9
#define OP_CAVITY      10


#define FLAG_LINEAR           (1 << 0)
#define FLAG_FIVED            (1 << 1)
#define FLAG_EXACT            (1 << 2)
#define FLAG_KICK             (1 << 3)
#define FLAG_RADIATION        (1 << 4)
#define FLAG_DOUBLE_PRECISION (1 << 5)
#define FLAG_ACHROMATIC       (1 << 6)

#ifdef data_64
typedef double float_t;
#else
typedef float float_t;
#endif

typedef struct {
                float_t x, y, z;
                float_t px, py, dp;
               } particle_t;

inline void load_inst_offset(inst_load_t** inst_offset,
                             inst_buf_t** inst_current,
                             inst_buf_t* inst_buf,
                             const ulong inst_buf_size) {

  const size_t lidx = get_local_id(0);
  const size_t lsize = get_local_size(0);
  const size_t load_words = inst_buf_size / sizeof(inst_load_t);

  __global inst_load_t* curr_offset = *inst_offset;
  __local inst_load_t* inst_buf_load = (inst_load_t*) inst_buf;
  
  for (size_t i = lidx; i < load_words; i += lsize) {
    inst_buf_load_t[i] = curr_offset[i];
  }

  barrier(CLK_LOCAL_MEM_FENCE);

  *inst_offset += load_words;
  *inst_current = inst_buf;
}

inline void load_inst_first_offset(inst_loat_t* inst,
                                   inst_load_t** inst_offset,
                                   inst_buf_t** inst_current,
                                   inst_buf_t* inst_buf,
                                   const ulong inst_buf_size) {

  *inst_offset = inst;
  load_offset(inst_offset, inst_current, inst_buf, inst_buf_size);
}

__kernel void run(__global const particle_t* input, 
                  __global particle_t* output,
                  __global inst_load_t* inst,
                  __local inst_buf_t* inst_buf,
                  const ulong particles,
                  const ulong inst_buf_size,
                  const ulong turns) {
  
  particle_t particle;
  __local inst_buf_t* inst_current;
  __global inst_load_t* inst_offset;

  const size_t idx = get_global_id(0);

  if (idx < particles) {
    particle = input[idx];
  }
  
  for (ulong turn = 0; turn < turns; turns++) {
    load_inst_first_offset(inst, &inst_offset, &inst_current, inst_buf, inst_buf_size);

    while (true) {
      __local op_t* op = inst_current;
      switch (*op) {
        case OP_NEXT_OFFSET: {
                              load_inst_offset(&inst_offset, &inst_current, inst_buf, inst_buf_size);
                              break;}
        case OP_ALIGN      : {
                              inst_current += align_args_size(args);
                              break;}
        case OP_DRIFT      : {
                              inst_current += drift_args_size(args);
                              break;}
        case OP_KICK       : {
                              inst_current += kick_args_size(args);
                              break;}
        case OP_TEAPOT     : {
                              inst_current += teapot_args_size(args);
                              break;}
        case OP_QUADRUPOLE : {
                              inst_current += quadrupole_args_size(args);
                              break;}
        case OP_SBEND      : {
                              inst_current += sbend_args_size(args);
                              break;}
        case OP_EDGE       : {
                              inst_current += edge_args_size(args);
                              break;}
        case OP_SEXTUPOLE  : {
                              inst_current += sextupole_args_size(args);
                              break;}
        case OP_OCTUPOLE   : {
                              inst_current += octupole_args_size(args);
                              break;}
        case OP_WIRE       : {
                              inst_current += wire_args_size(args);
                              break;}
        case OP_CAVITY     : {
                              inst_current += cavity_args_size(args);
                              break;}
        default: goto turn_end; // Case for OP_REWIND
      }
    }

    turn_end:
  }
}
