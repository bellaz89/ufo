#include "instructions.cl"
#define OP_REWIND 255
#define OP_NEXT_OFFSET 254
#define OP_DUMP 253

// Kills the particle if outside aperture.
// Updates the passed elements if the particle is alive
inline void update_passed_if_alive(particle_work_t *part_data,
                                   const bool check) {
  if (check) {
    const float_t x0 = part_data->particle.x;
    const float_t y0 = part_data->particle.y;
    const float_t center_distance = x0 * x0 + y0 * y0;
    part_data->particle.alive &= (center_distance < part_data->aperture_sq);
  }

  part_data->particle.passed_elements += part_data->particle.alive ? 1 : 0;
}

// Loads next instruction offset in the instruction buffer
inline void load_next_offset(__global const inst_t *inst, uint *inst_offset,
                             __local inst_t *inst_buf, uint *inst_current,
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
inline void load_first_offset(__global const inst_t *inst, uint *inst_offset,
                              __local inst_t *inst_buf, uint *inst_current,
                              const uint inst_buf_size) {
  // If there is only one offset and is already loaded, do not reload it
  if (*inst_offset == inst_buf_size && *inst_current != 0) {
    *inst_current = 0;
  } else {
    *inst_offset = 0;
    load_next_offset(inst, inst_offset, inst_buf, inst_current, inst_buf_size);
    UFO_ASSERT(*inst_offset == inst_buf_size,
               "expected inst_offset to be equal to %d, instead it is %d",
               inst_buf_size, *inst_offset)
  }
}

// Dumps particle data in the global memory
inline void dump_particles(particle_work_t *part_data,
                           __global particle_t *output, uint *output_offset,
                           const uint particles) {
  const uint idx = get_global_id(0);
  if (idx < particles) {
    output[idx + *output_offset] = part_data->particle;
  }
  *output_offset += particles;
}

__kernel void run(__global const particle_t *input, __global particle_t *output,
                  __global inst_t *inst, __local inst_t *inst_buf,
                  const uint particles, const uint inst_buf_size,
                  const uint turns, const uint output_size,
                  const uint instructions) {
  UNUSED(output_size)
  UNUSED(instructions)

  particle_work_t part_data;
  uint inst_offset = 0;
  uint inst_current = 0;
  uint output_offset = 0;
  uint instructions_done = 0;

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
    const flags_t flags = inst_decoded.flags;
    const intarg_t aux = inst_decoded.aux;
    const intarg_t argc = inst_decoded.argc;
    intarg_t args0 = argc;
    intarg_t args1 = 0;

    inst_current += 1;
    __local const float_t *args0_arr =
        (__local float_t *)(inst_buf + inst_current);
    __local const float_t *args1_arr = args0_arr + argc;
    if (op == OP_KICK) {
      args0 = aux;
      args1 = argc - aux;
      args1_arr = args0_arr + aux;
    } else if (op == OP_TEAPOT) {
      args0 = 4 + aux;
      args1 = argc - args0;
      args1_arr = args0_arr + args0;
    }
    inst_current += argc;

    UFO_DEBUG(
        "executing instruction op %d (id %d), flags %x, aux %d, argc %d", op,
        instructions_done, flags, aux, argc)
    instructions_done++;
    UFO_ASSERT(instructions_done <= instructions,
               "executing over %d instructions", instructions);

    for (uint i = 0; i < argc; i++) {
      UNUSED(i)
      UFO_ASSERT(!isnan(args0_arr[i]), "value %d of args is a NAN", i)
    }

    switch (op) {
    case OP_NEXT_OFFSET: {
      load_next_offset(inst, &inst_offset, inst_buf, &inst_current,
                       inst_buf_size);
      break;
    }
    case OP_DUMP: {
      dump_particles(&part_data, output, &output_offset, particles);
      UFO_ASSERT(output_offset <= output_size,
                 "output offset should be <= of %d but it is %d", output_size,
                 output_offset)
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
    case OP_TRAN_LINEAR: {
      tran_linear(&part_data, flags, args0, args1, args0_arr, args1_arr);
      break;
    }
    case OP_SET_APERTURE: {
      set_aperture(&part_data, flags, args0, args1, args0_arr, args1_arr);
      break;
    }
    case OP_REWIND: {
      turn++;
      if (turn == turns) {
        UFO_ASSERT(instructions_done == instructions,
                   "exiting the interpreter with %d executed "
                   "instructions out of %d",
                   instructions_done, instructions)
        UFO_ASSERT(output_offset == output_size,
                   "on exit output offset should be equal to %d but "
                   "it is %d",
                   output_size, output_offset)
        return;
      }
      load_first_offset(inst, &inst_offset, inst_buf, &inst_current,
                        inst_buf_size);
      break;
    }
    default: {
      UFO_ASSERT(0, "unknown op %d found at offset %d, cache offset %d", op,
                 inst_offset - 1, inst_current)
    }
    }
    if (op != OP_REWIND && op != OP_NEXT_OFFSET && op != OP_DUMP &&
        !(op == OP_TRAN_LINEAR && (flags & FLAG_TRAN_LINEAR_NO_PASS))) {
      update_passed_if_alive(&part_data, !(flags & FLAG_NO_APERTURE_CHECK));
    }
  }
}
