#![cfg(feature = "opencl")]

#[test]
fn builds_interpreter_when_opencl_tests_are_enabled() {
    if std::env::var_os("UFO_RUN_OPENCL_TESTS").is_none() {
        eprintln!("set UFO_RUN_OPENCL_TESTS=1 to build interpreter.cl through OpenCL");
        return;
    }

    ufo::opencl::build_interpreter_for_first_device("-cl-fast-relaxed-math -cl-mad-enable")
        .unwrap();
}

#[test]
fn tracks_one_drift_when_opencl_tests_are_enabled() {
    if std::env::var_os("UFO_RUN_OPENCL_TESTS").is_none() {
        eprintln!("set UFO_RUN_OPENCL_TESTS=1 to run interpreter tracking through OpenCL");
        return;
    }

    let lattice = ufo::mad::parse_mad("d: DRIFT, L=2.0; ring: LINE=(d);").unwrap();
    let tracking = ufo::compile_tracking_line(
        &lattice,
        lattice.line("ring").unwrap(),
        &ufo::TrackCompileOptions {
            where_: vec![-1.0],
            ..ufo::TrackCompileOptions::default()
        },
    )
    .unwrap();
    let output = ufo::opencl::track_first_device(
        &tracking,
        &[ufo::Particle {
            px: 0.25,
            ..ufo::Particle::default()
        }],
        &ufo::opencl::TrackRunOptions::default(),
    )
    .unwrap();

    assert_eq!(output.len(), 1);
    assert!((output[0].x - 0.5).abs() < 1.0e-5);
    assert_eq!(output[0].passed_elements, 1);
    assert!(output[0].alive);
}

#[test]
fn tracks_one_drift_in_double_precision_when_enabled() {
    if std::env::var_os("UFO_RUN_OPENCL_TESTS").is_none() {
        eprintln!("set UFO_RUN_OPENCL_TESTS=1 to run double-precision tracking through OpenCL");
        return;
    }

    let lattice = ufo::mad::parse_mad("d: DRIFT, L=2.0; ring: LINE=(d);").unwrap();
    let tracking = ufo::compile_tracking_line(
        &lattice,
        lattice.line("ring").unwrap(),
        &ufo::TrackCompileOptions {
            flags: ufo::PassFlags::DOUBLE_PRECISION,
            where_: vec![-1.0],
            ..ufo::TrackCompileOptions::default()
        },
    )
    .unwrap();
    assert!(tracking.bytecode.is_64bit());

    let output = match ufo::opencl::track_first_device(
        &tracking,
        &[ufo::Particle {
            px: 0.25,
            ..ufo::Particle::default()
        }],
        &ufo::opencl::TrackRunOptions::default(),
    ) {
        Ok(output) => output,
        Err(error) if error.to_string().contains("double") => {
            eprintln!("OpenCL device does not support double precision: {error}");
            return;
        }
        Err(error) => panic!("{error}"),
    };

    assert_eq!(output.len(), 1);
    assert!((output[0].x - 0.5).abs() < 1.0e-12);
    assert_eq!(output[0].passed_elements, 1);
    assert!(output[0].alive);
}

#[test]
fn tracks_fractional_observation_inside_drift_when_enabled() {
    if std::env::var_os("UFO_RUN_OPENCL_TESTS").is_none() {
        eprintln!("set UFO_RUN_OPENCL_TESTS=1 to run fractional tracking through OpenCL");
        return;
    }

    let lattice = ufo::mad::parse_mad("d: DRIFT, L=2.0; ring: LINE=(d);").unwrap();
    let tracking = ufo::compile_tracking_line(
        &lattice,
        lattice.line("ring").unwrap(),
        &ufo::TrackCompileOptions {
            where_: vec![0.25, -1.0],
            ..ufo::TrackCompileOptions::default()
        },
    )
    .unwrap();
    let output = ufo::opencl::track_first_device(
        &tracking,
        &[ufo::Particle {
            px: 0.25,
            ..ufo::Particle::default()
        }],
        &ufo::opencl::TrackRunOptions::default(),
    )
    .unwrap();

    assert_eq!(output.len(), 2);
    assert!((output[0].x - 0.125).abs() < 1.0e-5);
    assert!((output[1].x - 0.5).abs() < 1.0e-5);
}

#[test]
fn computes_periodic_optics_when_enabled() {
    if std::env::var_os("UFO_RUN_OPENCL_TESTS").is_none() {
        eprintln!("set UFO_RUN_OPENCL_TESTS=1 to run periodic optics through OpenCL");
        return;
    }

    let lattice = ufo::load_mad_file("optics/fodo.mad").unwrap();
    let optics = ufo::Optics::periodic(
        &lattice,
        lattice.line("RING").unwrap(),
        ufo::OpticsOptions {
            where_: vec![0.0, -1.0],
            ..ufo::OpticsOptions::default()
        },
    )
    .unwrap();

    assert_eq!(optics.points.len(), 2);
    assert!(optics.qx.is_finite());
    assert!(optics.qy.is_finite());
    assert!(optics.bx0 > 0.0);
    assert!(optics.by0 > 0.0);
    assert!(optics.points[0].bx > 0.0);
    assert!(optics.points[0].by > 0.0);
}

#[test]
fn propagates_optics_when_enabled() {
    if std::env::var_os("UFO_RUN_OPENCL_TESTS").is_none() {
        eprintln!("set UFO_RUN_OPENCL_TESTS=1 to run propagated optics through OpenCL");
        return;
    }

    let lattice = ufo::mad::parse_mad("d: DRIFT, L=2.0; ring: LINE=(d);").unwrap();
    let optics = ufo::Optics::propagate(
        &lattice,
        lattice.line("ring").unwrap(),
        ufo::InitialOptics {
            bx: 1.0,
            by: 1.0,
            ..ufo::InitialOptics::default()
        },
        ufo::OpticsOptions {
            where_: vec![-1.0],
            ..ufo::OpticsOptions::default()
        },
    )
    .unwrap();

    assert_eq!(optics.points.len(), 1);
    assert!((optics.points[0].bx - 5.0).abs() < 1.0e-5);
    assert!((optics.points[0].by - 5.0).abs() < 1.0e-5);
    assert!((optics.points[0].ax + 2.0).abs() < 1.0e-5);
    assert!((optics.points[0].ay + 2.0).abs() < 1.0e-5);
    assert!(optics.qx.is_nan());
    assert!(optics.qy.is_nan());
}

#[test]
fn computes_chromaticity_when_enabled() {
    if std::env::var_os("UFO_RUN_OPENCL_TESTS").is_none() {
        eprintln!("set UFO_RUN_OPENCL_TESTS=1 to run chromaticity through OpenCL");
        return;
    }

    let lattice = ufo::load_mad_file("optics/alba.mad").unwrap();
    let ring = lattice.line("RING").unwrap();
    let optics = ufo::Optics::periodic(
        &lattice,
        ring,
        ufo::OpticsOptions {
            where_: ufo::chromaticity_observations(&lattice, ring).unwrap(),
            ..ufo::OpticsOptions::default()
        },
    )
    .unwrap();
    let chromaticity = ufo::chromaticity(&lattice, ring, &optics).unwrap();

    assert!(chromaticity.natural[0].is_finite());
    assert!(chromaticity.natural[1].is_finite());
    assert!((chromaticity.total[0] - 1.752728632).abs() < 1.0e-2);
    assert!((chromaticity.total[1] - 3.763977279).abs() < 1.0e-2);
}

#[test]
fn computes_radiation_integrals_when_enabled() {
    if std::env::var_os("UFO_RUN_OPENCL_TESTS").is_none() {
        eprintln!("set UFO_RUN_OPENCL_TESTS=1 to run radiation integrals through OpenCL");
        return;
    }

    let lattice = ufo::load_mad_file("optics/alba.mad").unwrap();
    let ring = lattice.line("RING").unwrap();
    let optics = ufo::Optics::periodic(
        &lattice,
        ring,
        ufo::OpticsOptions {
            where_: ufo::radiation_observations(&lattice, ring).unwrap(),
            ..ufo::OpticsOptions::default()
        },
    )
    .unwrap();
    let radiation = ufo::emittance(&lattice, ring, &optics).unwrap();

    assert!((radiation.i1 - 0.289626312).abs() < 1.0e-2);
    assert!((radiation.i2 - 0.8979648783).abs() < 1.0e-2);
    assert!((radiation.i4 - -0.3327286365).abs() < 1.0e-2);
    assert!((radiation.i5 - 0.0005327915674).abs() < 1.0e-2);
    assert!(radiation.u0(&ufo::Beam::default()).is_finite());
    assert!(radiation.ex(&ufo::Beam::default()).is_finite());
}

#[test]
fn computes_closed_orbit_when_enabled() {
    if std::env::var_os("UFO_RUN_OPENCL_TESTS").is_none() {
        eprintln!("set UFO_RUN_OPENCL_TESTS=1 to run closed orbit through OpenCL");
        return;
    }

    let lattice = ufo::load_mad_file("optics/fodo.mad").unwrap();
    let ring = lattice.line("RING").unwrap();
    let orbit = ufo::closed_orbit(
        &lattice,
        ring,
        ufo::ClosedOrbitOptions {
            iterations: 20,
            ..ufo::ClosedOrbitOptions::default()
        },
    )
    .unwrap();
    assert!(orbit.residual < 1.0e-12);

    let tracking = ufo::compile_tracking_line(
        &lattice,
        ring,
        &ufo::TrackCompileOptions {
            where_: vec![-1.0],
            ..ufo::TrackCompileOptions::default()
        },
    )
    .unwrap();
    let output = ufo::opencl::track_first_device(
        &tracking,
        &[ufo::Particle {
            x: orbit.orbit[0],
            px: orbit.orbit[1],
            y: orbit.orbit[2],
            py: orbit.orbit[3],
            ..ufo::Particle::default()
        }],
        &ufo::opencl::TrackRunOptions::default(),
    )
    .unwrap();
    assert!((output[0].x - orbit.orbit[0]).abs() < 1.0e-6);
    assert!((output[0].px - orbit.orbit[1]).abs() < 1.0e-6);
    assert!((output[0].y - orbit.orbit[2]).abs() < 1.0e-6);
    assert!((output[0].py - orbit.orbit[3]).abs() < 1.0e-6);
}

#[test]
fn computes_rdt_when_enabled() {
    if std::env::var_os("UFO_RUN_OPENCL_TESTS").is_none() {
        eprintln!("set UFO_RUN_OPENCL_TESTS=1 to run RDT through OpenCL");
        return;
    }

    let lattice = ufo::load_mad_file("optics/alba.mad").unwrap();
    let ring = lattice.line("RING").unwrap();
    let optics = ufo::Optics::periodic(
        &lattice,
        ring,
        ufo::OpticsOptions {
            where_: ufo::rdt_observations(&lattice, ring).unwrap(),
            ..ufo::OpticsOptions::default()
        },
    )
    .unwrap();
    let terms = ufo::rdt(&lattice, ring, &optics).unwrap();

    assert!(terms.f3000.abs().is_finite());
    assert!(terms.f1200.abs().is_finite());
    assert!(terms.f1020.abs().is_finite());
    assert!(terms.f0120.abs().is_finite());
    assert!(terms.f0111.abs().is_finite());
    assert!(terms.f3000.abs() > 0.0);
}

#[test]
fn computes_stable_aperture_when_enabled() {
    if std::env::var_os("UFO_RUN_OPENCL_TESTS").is_none() {
        eprintln!("set UFO_RUN_OPENCL_TESTS=1 to run stable aperture through OpenCL");
        return;
    }

    let lattice = ufo::mad::parse_mad("d: DRIFT, L=1.0; ring: LINE=(d);").unwrap();
    let result = ufo::stable_aperture(
        &lattice,
        lattice.line("ring").unwrap(),
        vec![
            ufo::Particle {
                x: 0.0,
                ..ufo::Particle::default()
            },
            ufo::Particle {
                x: 2.0,
                ..ufo::Particle::default()
            },
        ],
        ufo::StableApertureOptions {
            turns: 3,
            ..ufo::StableApertureOptions::default()
        },
    )
    .unwrap();

    assert_eq!(result.lost_turns, vec![3, 0]);
}

#[test]
fn tracks_multi_element_drift_line_when_enabled() {
    if std::env::var_os("UFO_RUN_OPENCL_TESTS").is_none() {
        eprintln!("set UFO_RUN_OPENCL_TESTS=1 to run multi-element tracking through OpenCL");
        return;
    }

    let lattice = ufo::load_mad_file("optics/drift.mad").unwrap();
    let tracking = ufo::compile_tracking_line(
        &lattice,
        lattice.line("RING").unwrap(),
        &ufo::TrackCompileOptions {
            where_: vec![-1.0],
            ..ufo::TrackCompileOptions::default()
        },
    )
    .unwrap();
    let output = ufo::opencl::track_first_device(
        &tracking,
        &[ufo::Particle {
            px: 0.25,
            ..ufo::Particle::default()
        }],
        &ufo::opencl::TrackRunOptions::default(),
    )
    .unwrap();

    assert_eq!(output.len(), 1);
    assert!((output[0].x - 2.5).abs() < 1.0e-5);
    assert_eq!(output[0].passed_elements, 4);
}

#[test]
fn tracks_with_chunked_instruction_buffer_when_enabled() {
    if std::env::var_os("UFO_RUN_OPENCL_TESTS").is_none() {
        eprintln!("set UFO_RUN_OPENCL_TESTS=1 to run chunked tracking through OpenCL");
        return;
    }

    let lattice = ufo::load_mad_file("optics/drift.mad").unwrap();
    let tracking = ufo::compile_tracking_line(
        &lattice,
        lattice.line("RING").unwrap(),
        &ufo::TrackCompileOptions {
            where_: vec![-1.0],
            ..ufo::TrackCompileOptions::default()
        },
    )
    .unwrap();
    let output = ufo::opencl::track_first_device(
        &tracking,
        &[ufo::Particle {
            px: 0.25,
            ..ufo::Particle::default()
        }],
        &ufo::opencl::TrackRunOptions {
            local_instruction_words: Some(4),
            ..ufo::opencl::TrackRunOptions::default()
        },
    )
    .unwrap();

    assert_eq!(output.len(), 1);
    assert!((output[0].x - 2.5).abs() < 1.0e-5);
    assert_eq!(output[0].passed_elements, 4);
}

#[test]
fn tracks_aligned_multipole_offset_when_enabled() {
    if std::env::var_os("UFO_RUN_OPENCL_TESTS").is_none() {
        eprintln!("set UFO_RUN_OPENCL_TESTS=1 to run aligned multipole tracking through OpenCL");
        return;
    }

    let lattice =
        ufo::mad::parse_mad("m: MULTIPOLE, KNL={0.0, 1.0}, DX=1.0; ring: LINE=(m);").unwrap();
    let tracking = ufo::compile_tracking_line(
        &lattice,
        lattice.line("ring").unwrap(),
        &ufo::TrackCompileOptions {
            flags: ufo::PassFlags::ACHROMATIC,
            where_: vec![-1.0],
            ..ufo::TrackCompileOptions::default()
        },
    )
    .unwrap();
    let output = ufo::opencl::track_first_device(
        &tracking,
        &[ufo::Particle::default()],
        &ufo::opencl::TrackRunOptions::default(),
    )
    .unwrap();

    assert_eq!(output.len(), 1);
    assert!((output[0].x - 0.0).abs() < 1.0e-6);
    assert!((output[0].px - 1.0).abs() < 1.0e-6);
    assert_eq!(output[0].passed_elements, 1);
}

#[test]
fn tracks_quadrupole_field_error_in_kick_mode_when_enabled() {
    if std::env::var_os("UFO_RUN_OPENCL_TESTS").is_none() {
        eprintln!("set UFO_RUN_OPENCL_TESTS=1 to run field-error tracking through OpenCL");
        return;
    }

    let lattice =
        ufo::mad::parse_mad("q: QUADRUPOLE, L=1.0, K1=0.0, DK1=1.0; ring: LINE=(q);").unwrap();
    let tracking = ufo::compile_tracking_line(
        &lattice,
        lattice.line("ring").unwrap(),
        &ufo::TrackCompileOptions {
            flags: ufo::PassFlags::KICK | ufo::PassFlags::ACHROMATIC,
            where_: vec![-1.0],
            ..ufo::TrackCompileOptions::default()
        },
    )
    .unwrap();
    let output = ufo::opencl::track_first_device(
        &tracking,
        &[ufo::Particle {
            x: 1.0,
            ..ufo::Particle::default()
        }],
        &ufo::opencl::TrackRunOptions::default(),
    )
    .unwrap();

    assert_eq!(output.len(), 1);
    assert!(output[0].px < -0.8);
    assert_eq!(output[0].passed_elements, 1);
}

#[test]
fn track_api_runs_and_indexes_samples_when_enabled() {
    if std::env::var_os("UFO_RUN_OPENCL_TESTS").is_none() {
        eprintln!("set UFO_RUN_OPENCL_TESTS=1 to run Track API through OpenCL");
        return;
    }

    let lattice = ufo::mad::parse_mad("d: DRIFT, L=2.0; ring: LINE=(d);").unwrap();
    let mut track = ufo::Track::new(
        &lattice,
        lattice.line("ring").unwrap(),
        vec![ufo::Particle {
            px: 0.25,
            ..ufo::Particle::default()
        }],
        ufo::TrackOptions {
            turns: 1,
            where_: vec![0.25, -1.0],
            ..ufo::TrackOptions::default()
        },
    )
    .unwrap();
    track.run().unwrap();

    assert!((track.sample(0, 0, 0).unwrap().x - 0.125).abs() < 1.0e-5);
    assert!((track.sample(0, 1, 0).unwrap().x - 0.5).abs() < 1.0e-5);
}

#[test]
fn opencl3_particle_copy_kernel_matches_host_layout_when_enabled() {
    if std::env::var_os("UFO_RUN_OPENCL_TESTS").is_none() {
        eprintln!("set UFO_RUN_OPENCL_TESTS=1 to run particle copy through OpenCL");
        return;
    }

    use opencl3::{
        command_queue::{CL_BLOCKING, CommandQueue},
        context::Context,
        device::{CL_DEVICE_TYPE_ALL, Device},
        kernel::Kernel,
        memory::{Buffer, CL_MEM_READ_ONLY, CL_MEM_WRITE_ONLY, ClMem},
        platform::get_platforms,
        program::Program,
    };
    use std::ptr;

    let platform = get_platforms().unwrap().into_iter().next().unwrap();
    let device_id = platform
        .get_devices(CL_DEVICE_TYPE_ALL)
        .unwrap()
        .into_iter()
        .next()
        .unwrap();
    let device = Device::new(device_id);
    let context = Context::from_device(&device).unwrap();
    let queue = unsafe { CommandQueue::create(&context, device_id, 0) }.unwrap();
    let program = Program::create_and_build_from_source(
        &context,
        r#"
typedef struct {
  float x, y, z;
  float px, py, dp;
  uint passed_elements;
  uint alive;
} particle_t;
__kernel void copy(__global const particle_t *input, __global particle_t *output) {
  particle_t p = input[get_global_id(0)];
  p.x += p.px * 2.0f;
  p.passed_elements += 1;
  output[get_global_id(0)] = p;
}
"#,
        "",
    )
    .unwrap();
    let kernel = Kernel::create(&program, "copy").unwrap();
    let input = [ufo::Particle32 {
        px: 0.25,
        alive: 1,
        ..ufo::Particle32::default()
    }];
    let mut output = [ufo::Particle32::default()];
    let mut input_buffer = unsafe {
        Buffer::<ufo::Particle32>::create(&context, CL_MEM_READ_ONLY, 1, ptr::null_mut())
    }
    .unwrap();
    let output_buffer = unsafe {
        Buffer::<ufo::Particle32>::create(&context, CL_MEM_WRITE_ONLY, 1, ptr::null_mut())
    }
    .unwrap();

    unsafe {
        queue
            .enqueue_write_buffer(&mut input_buffer, CL_BLOCKING, 0, &input, &[])
            .unwrap();
        let input_mem = input_buffer.get();
        let output_mem = output_buffer.get();
        kernel.set_arg(0, &input_mem).unwrap();
        kernel.set_arg(1, &output_mem).unwrap();
        let global = [1usize];
        let local = [1usize];
        queue
            .enqueue_nd_range_kernel(
                kernel.get(),
                1,
                ptr::null(),
                global.as_ptr(),
                local.as_ptr(),
                &[],
            )
            .unwrap();
        queue
            .enqueue_read_buffer(&output_buffer, CL_BLOCKING, 0, &mut output, &[])
            .unwrap();
    }

    assert!((output[0].x - 0.5).abs() < 1.0e-6);
    assert_eq!(output[0].px, 0.25);
    assert_eq!(output[0].passed_elements, 1);
    assert_eq!(output[0].alive, 1);
}

#[test]
fn opencl3_interpreter_signature_copy_kernel_when_enabled() {
    if std::env::var_os("UFO_RUN_OPENCL_TESTS").is_none() {
        eprintln!("set UFO_RUN_OPENCL_TESTS=1 to run signature copy through OpenCL");
        return;
    }

    use opencl3::{
        command_queue::{CL_BLOCKING, CommandQueue},
        context::Context,
        device::{CL_DEVICE_TYPE_ALL, Device},
        kernel::Kernel,
        memory::{Buffer, CL_MEM_READ_ONLY, CL_MEM_READ_WRITE, CL_MEM_WRITE_ONLY, ClMem},
        platform::get_platforms,
        program::Program,
    };
    use std::ptr;

    let platform = get_platforms().unwrap().into_iter().next().unwrap();
    let device_id = platform
        .get_devices(CL_DEVICE_TYPE_ALL)
        .unwrap()
        .into_iter()
        .next()
        .unwrap();
    let device = Device::new(device_id);
    let context = Context::from_device(&device).unwrap();
    let queue = unsafe { CommandQueue::create(&context, device_id, 0) }.unwrap();
    let program = Program::create_and_build_from_source(
        &context,
        r#"
typedef struct { uchar op; uchar flags; uchar aux; uchar argc; } inst_t;
typedef struct {
  float x, y, z;
  float px, py, dp;
  uint passed_elements;
  uint alive;
} particle_t;
__kernel void run(__global const particle_t *input, __global particle_t *output,
                  __global inst_t *inst, __local inst_t *inst_buf,
                  const uint particles, const uint inst_buf_size,
                  const uint turns, const uint output_size,
                  const uint instructions) {
  (void)inst;
  (void)inst_buf;
  (void)inst_buf_size;
  (void)turns;
  (void)output_size;
  (void)instructions;
  uint idx = get_global_id(0);
  if (idx < particles) {
    particle_t p = input[idx];
    p.x += p.px * 2.0f;
    p.passed_elements += 1;
    output[idx] = p;
  }
}
"#,
        "",
    )
    .unwrap();
    let kernel = Kernel::create(&program, "run").unwrap();
    let input = [ufo::Particle32 {
        px: 0.25,
        alive: 1,
        ..ufo::Particle32::default()
    }];
    let mut output = [ufo::Particle32::default()];
    let inst = [0u8; 4];
    let mut input_buffer = unsafe {
        Buffer::<ufo::Particle32>::create(&context, CL_MEM_READ_ONLY, 1, ptr::null_mut())
    }
    .unwrap();
    let output_buffer = unsafe {
        Buffer::<ufo::Particle32>::create(&context, CL_MEM_WRITE_ONLY, 1, ptr::null_mut())
    }
    .unwrap();
    let mut inst_buffer =
        unsafe { Buffer::<u8>::create(&context, CL_MEM_READ_WRITE, inst.len(), ptr::null_mut()) }
            .unwrap();

    unsafe {
        queue
            .enqueue_write_buffer(&mut input_buffer, CL_BLOCKING, 0, &input, &[])
            .unwrap();
        queue
            .enqueue_write_buffer(&mut inst_buffer, CL_BLOCKING, 0, &inst, &[])
            .unwrap();
        let input_mem = input_buffer.get();
        let output_mem = output_buffer.get();
        let inst_mem = inst_buffer.get();
        let one = 1u32;
        kernel.set_arg(0, &input_mem).unwrap();
        kernel.set_arg(1, &output_mem).unwrap();
        kernel.set_arg(2, &inst_mem).unwrap();
        kernel.set_arg_local_buffer(3, 4).unwrap();
        kernel.set_arg(4, &one).unwrap();
        kernel.set_arg(5, &one).unwrap();
        kernel.set_arg(6, &one).unwrap();
        kernel.set_arg(7, &one).unwrap();
        kernel.set_arg(8, &one).unwrap();
        let global = [1usize];
        let local = [1usize];
        queue
            .enqueue_nd_range_kernel(
                kernel.get(),
                1,
                ptr::null(),
                global.as_ptr(),
                local.as_ptr(),
                &[],
            )
            .unwrap();
        queue
            .enqueue_read_buffer(&output_buffer, CL_BLOCKING, 0, &mut output, &[])
            .unwrap();
    }

    assert!((output[0].x - 0.5).abs() < 1.0e-6);
    assert_eq!(output[0].px, 0.25);
    assert_eq!(output[0].passed_elements, 1);
    assert_eq!(output[0].alive, 1);
}
