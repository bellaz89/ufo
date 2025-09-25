#include "instructions.cl"
#define __UFO_GENERATE_LOCAL_INSTRUCTIONS__
#include "instructions.cl"
#define OP_REWIND -1
#define OP_NEXT_OFFSET -2
#define OP_DUMP -3

#if 0
{{ endif }}
// custom instructions opcodes
{% for instruction in gen_instructions %}
#define OP_            \
  {                    \
    {                  \
      instruction.name \
    }                  \
  }                    \
  {                    \
    {                  \
      instruction op   \
    }                  \
  }
{% endfor %}
{{ if_0 }}
#endif

__constant const char* op_str(op_t op) {
  switch (op) {
    case OP_NEXT_OFFSET: {
      return "next_offset";
    }
    case OP_DUMP: {
      return "dump";
    }
    case OP_ALIGN: {
      return "align";
    }
    case OP_DRIFT: {
      return "drift";
    }
    case OP_KICK: {
      return "kick";
    }
    case OP_TEAPOT: {
      return "teapot";
    }
    case OP_QUADRUPOLE: {
      return "quadrupole";
    }
    case OP_SBEND: {
      return "sbend";
    }
    case OP_EDGE: {
      return "edge";
    }
    case OP_WIRE: {
      return "wire";
    }
    case OP_CAVITY: {
      return "cavity";
    }
    case OP_TRAV_LINEAR: {
      return "trav_linear";
    }
    case OP_SET_APERTURE: {
      return "set_aperture";
    }
    case OP_REWIND: {
      return "rewind";
    }
    // clang-format off
    #if 0
    {{ endif }}
    // custom instructions names
    {% for instruction in gen_instructions %}
    case OP_{{ instruction.name }} {
      return "{{ instruction.name }}"
    }
    {% endfor %}
    {{ if_0 }}
    #endif
    // clang-format on
    default: {
      return "unknown";
    }
  }
}

// Kills the particle if ouside aperture
// Updates the passed elements if the particle is alive
inline void update_passed_if_alive(particle_work_t* part_data,
                                   const bool check) {
  if (check) {
    const float_t x0 = part_data->particle.x;
    const float_t y0 = part_data->particle.y;
    const float_t center_distance = x0 * x0 + y0 * y0;
    part_data->particle.alive &= (center_distance < part_data->aperture_sq);
  }

  part_data->particle.passed_elements += part_data->particle.alive ? 1 : 0;
}

// clang-format off
#if 0
{{ endif }}

// custom instructions declarations
{% for instruction in gen_instructions %}

inline void local_{{ instruction.name }}(particle_work_t* part_data, const flags_t flags, const intarg_t args0, const intarg_t args1, __local const float_t* args0_arr, __local const float_t* args1_arr);
inline void {{ instruction.name }}(particle_work_t* part_data, const flags_t flags, const intarg_t args0, const intarg_t args1, const float_t* args0_arr, const float_t* args1_arr);
{% endfor %}
{{ if_0 }}
#endif

#if 0
{{ endif }}

{% macro instruction_body(instruction) %}
  UNUSED(flags)
  UNUSED(args0)
  UNUSED(args1)
  UNUSED(args0_arr)
  UNUSED(args1_arr)

  {% for inner in instruction.inner_instructions %}
  {
    // Instance of {{ repr(inner) }}
    const flags_t  _flags = {{ inner.flags }};
    const intarg_t _args0 = {{ inner.args0 }};
    const intarg_t _args1 = {{ inner.args1 }};
    const float_t _args0_arr[] = { {{ ", ".join(inner.args0_arr) }} };
    const float_t _args1_arr[] = { {{ ", ".join(inner.args1_arr) }} };
    {{ inner.type }}(part_data, _flags, _args0, _args1, _args0_arr, _args1_arr);
    update_passed_if_alive(part_data, !(inner.flags & FLAG_NO_APERTURE_CHECK));
  }
  {% endfor %}
{% endmacro %}

// custom instructions definitions
{% for instruction in gen_instructions %}

inline void local_{{ instruction.name }}(particle_work_t* part_data, const flags_t flags, const intarg_t args0, const intarg_t args1, __local const float_t* args0_arr, __local const float_t* args1_arr) {
  {{ instruction_body(instruction) }}
}

inline void {{ instruction.name }}(particle_work_t* part_data, const flags_t flags, const intarg_t args0, const intarg_t args1, const float_t* args0_arr, const float_t* args1_arr); {
  {{ instruction_body(instruction) }}
}

{% endfor %}
{{ if_0 }}
#endif
// clang-format on

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
inline void dump_particles(particle_work_t* part_data,
                           __global particle_t* output, uint* output_offset,
                           const uint particles) {
  const uint idx = get_global_id(0);
  if (idx < particles) {
    output[idx + *output_offset] = part_data->particle;
  }
  *output_offset += particles;
}

__kernel void run(__global const particle_t* input, __global particle_t* output,
                  __global inst_t* inst, __local inst_t* inst_buf,
                  const uint particles, const uint inst_buf_size,
                  const uint turns, const uint output_size,
                  const uint instructions) {
  UNUSED(output_size)
  UNUSED(instructions)

  particle_work_t part_data;
  uint inst_offset;
  uint inst_current;
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

    UFO_DEBUG(
        "executing instruction %s (id %d), flags %x, args0 %d, args1 "
        "%d",
        op_str(op), instructions_done, flags, args0, args1)
    instructions_done++;
    UFO_ASSERT(instructions_done <= instructions,
               "executing over %d instructions", instructions);

    for (uint i = 0; i < args0; i++) {
      UNUSED(i)
      UFO_ASSERT(!isnan(args0_arr[i]), "value %d of args0 is a NAN", i)
    }

    for (uint i = 0; i < args0; i++) {
      UNUSED(i)
      UFO_ASSERT(!isnan(args0_arr[i]), "value %d of args1 is a NAN", i)
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
      case OP_ALIGN: {
        __INST(align)(&part_data, flags, args0, args1, args0_arr, args1_arr);
        break;
      }
      case OP_DRIFT: {
        __INST(drift)(&part_data, flags, args0, args1, args0_arr, args1_arr);
        break;
      }
      case OP_KICK: {
        __INST(kick)(&part_data, flags, args0, args1, args0_arr, args1_arr);
        break;
      }
      case OP_TEAPOT: {
        __INST(teapot)(&part_data, flags, args0, args1, args0_arr, args1_arr);
        break;
      }
      case OP_QUADRUPOLE: {
        __INST(quadrupole)(&part_data, flags, args0, args1, args0_arr,
                           args1_arr);
        break;
      }
      case OP_SBEND: {
        __INST(sbend)(&part_data, flags, args0, args1, args0_arr, args1_arr);
        break;
      }
      case OP_EDGE: {
        __INST(edge)(&part_data, flags, args0, args1, args0_arr, args1_arr);
        break;
      }
      case OP_WIRE: {
        __INST(wire)(&part_data, flags, args0, args1, args0_arr, args1_arr);
        break;
      }
      case OP_CAVITY: {
        __INST(cavity)(&part_data, flags, args0, args1, args0_arr, args1_arr);
        break;
      }
      case OP_TRAV_LINEAR: {
        __INST(trav_linear)(&part_data, flags, args0, args1, args0_arr,
                            args1_arr);
        break;
      }
      case OP_SET_APERTURE: {
        __INST(set_aperture)(&part_data, flags, args0, args1, args0_arr,
                             args1_arr);
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
      // clang-format off
      #if 0
      {{ endif }}
      {% for instruction in gen_instructions %}
      case OP_{{ instruction.name }}: {
        local_{{ instruction.name }}(&part_data, flags, args0, args1, args0_arr, args1_arr);
        break;
      }
      {% endfor %}
      {{ if_0 }}
      #endif
      // clang-format on
      default: {
        UFO_ASSERT(0, "unknown op %d found at offset %d, cache offset %d", op,
                   inst_offset - 1, inst_current)
      }
    }
    update_passed_if_alive(&part_data, !(flags & FLAG_NO_APERTURE_CHECK));
  }
}
