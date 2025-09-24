
#include "instructions.cl"
#define OP_REWIND -1
#define OP_NEXT_OFFSET -2
#define OP_DUMP -3

// Kills the particle if ouside aperture
// Updates the passed elements if the particle is alive
inline void update_passed_if_alive(particle_work_t* part_data) {
  float_t center_distance = part_data->particle.x * part_data->particle.x;
  center_distance += part_data->particle.y * part_data->particle.y;
  part_data->particle.alive &= center_distance < part_data->aperture_sq;
  part_data->particle.passed_elements += part_data->particle.alive ? 1 : 0;
}

// Loads next instruction offset in the instruction buffer
inline void load_next_offset(__global const inst_t* inst, uint* inst_offset,
                             __local inst_t* inst_buf, uint* inst_current,
                             const uint inst_buf_size) {
  const uint lidx = get_local_id(0);
  const uint lsize = get_local_size(0);

  barrier(CLK_LOCAL_MEM_FENCE);

  for (uint i = lidx; i < inst_buf_size; i += lsize) {
    inst_buf[i] = inst[*inst_offset + i];
  }

  barrier(CLK_LOCAL_MEM_FENCE);

  *inst_offset += inst_buf_size;
  *inst_current = 0;
}

// Loads the first instruction offset in the instruction buffer
inline void load_first_offset(__global const inst_t* inst, uint* inst_offset,
                              __local inst_t* inst_buf, uint* inst_current,
                              const uint inst_buf_size) {
  *inst_offset = 0;
  load_next_offset(inst, inst_offset, inst_buf, inst_current, inst_buf_size);
}

// Dumps particle data in the global memory
inline void dump_particles(particle_work_t* part_data,
                           __global particle_t* output, uint* dump_offset,
                           const uint particles) {
  const uint idx = get_global_id(0);
  if (idx < particles) {
    output[idx + *dump_offset] = part_data->particle;
  }
  *dump_offset += particles;
}

__kernel void run(__global const particle_t* input, __global particle_t* output,
                  __global inst_t* inst, __local inst_t* inst_buf,
                  const uint particles, const uint inst_buf_size,
                  const uint turns) {
  particle_work_t part_data;
  uint inst_offset;
  uint inst_current;
  uint dump_offset = 0;

  const uint idx = get_global_id(0);

  if (turns == 0) {
    return;
  }

  // Particle load
  if (idx < particles) {
    part_data.particle = input[idx];
    update_oodppo(&part_data);
    part_data.aperture_sq = FLOAT_MAX;
  }

  uint turn = 0;
  load_first_offset(inst, &inst_offset, inst_buf, &inst_current, inst_buf_size);

  while (true) {
    // Instruction decoding
    const inst_t inst_decoded = inst_buf[inst_current];
    const op_t op = inst_decoded.op;
    const flags_t flags = inst_decoded.op;
    const intarg_t args0 = inst_decoded.args0;
    const intarg_t args1 = inst_decoded.args1;

    inst_current += 1;
    __local const float_t* args0_arr =
        (__local float_t*)(inst_buf + inst_current);
    inst_current += args0;
    __local const float_t* args1_arr =
        (__local float_t*)(inst_buf + inst_current);
    inst_current += args1;

    switch (op) {
      case OP_NEXT_OFFSET: {
        load_next_offset(inst, &inst_offset, inst_buf, &inst_current,
                         inst_buf_size);
        break;
      }
      case OP_DUMP: {
        dump_particles(&part_data, output, &dump_offset, particles);
        break;
      }
      case OP_ALIGN: {
        align(&part_data, flags, args0, args1, args0_arr, args1_arr);
        break;
      }
      case OP_DRIFT: {
        drift(&part_data, flags, args0, args1, args0_arr, args1_arr);
        break;
      }
      case OP_KICK: {
        kick(&part_data, flags, args0, args1, args0_arr, args1_arr);
        break;
      }
      case OP_TEAPOT: {
        teapot(&part_data, flags, args0, args1, args0_arr, args1_arr);
        break;
      }
      case OP_QUADRUPOLE: {
        quadrupole(&part_data, flags, args0, args1, args0_arr, args1_arr);
        break;
      }
      case OP_SBEND: {
        sbend(&part_data, flags, args0, args1, args0_arr, args1_arr);
        break;
      }
      case OP_EDGE: {
        edge(&part_data, flags, args0, args1, args0_arr, args1_arr);
        break;
      }
      case OP_WIRE: {
        wire(&part_data, flags, args0, args1, args0_arr, args1_arr);
        break;
      }
      case OP_CAVITY: {
        cavity(&part_data, flags, args0, args1, args0_arr, args1_arr);
        break;
      }
      case OP_TRAV_LINEAR: {
        trav_linear(&part_data, flags, args0, args1, args0_arr, args1_arr);
        break;
      }
      case OP_SET_APERTURE: {
        set_aperture(&part_data, flags, args0, args1, args0_arr, args1_arr);
        break;
      }
      default: {  // Case for OP_REWIND.
        turn++;
        if (turn == turns) {
          return;
        }
        load_first_offset(inst, &inst_offset, inst_buf, &inst_current,
                          inst_buf_size);
        break;
      }
    }
    update_passed_if_alive(&part_data);
  }
}
