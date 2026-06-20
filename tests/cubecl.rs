#![cfg(feature = "cubecl-cpu")]

use std::collections::BTreeSet;

fn track_cpu(
    tracking: &ufo::TrackingBytecode,
    particles: &[ufo::Particle],
) -> ufo::Result<Vec<ufo::Particle>> {
    ufo::cubecl::track_with_options(
        tracking,
        particles,
        &ufo::cubecl::CubeClTrackRunOptions {
            backend: ufo::cubecl::CubeClBackend::Cpu,
            ..ufo::cubecl::CubeClTrackRunOptions::default()
        },
    )
}

fn compile_tracking(input: &str, line: &str, flags: ufo::PassFlags) -> ufo::TrackingBytecode {
    let lattice = ufo::mad::parse_mad(input).unwrap();
    ufo::compile_tracking_line(
        &lattice,
        lattice.line(line).unwrap(),
        &ufo::TrackCompileOptions {
            flags,
            where_: vec![-1.0],
            ..ufo::TrackCompileOptions::default()
        },
    )
    .unwrap()
}

fn opcodes(tracking: &ufo::TrackingBytecode) -> BTreeSet<u16> {
    tracking
        .bytecode
        .instructions()
        .iter()
        .map(|instruction| instruction.op)
        .collect()
}

fn assert_finite_particle(particle: &ufo::Particle) {
    for value in [
        particle.x,
        particle.px,
        particle.y,
        particle.py,
        particle.z,
        particle.dp,
    ] {
        assert!(
            value.is_finite(),
            "non-finite particle output: {particle:?}"
        );
    }
}

#[test]
fn tracks_one_drift_on_cubecl_cpu() {
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

    let output = ufo::cubecl::track_with_options(
        &tracking,
        &[ufo::Particle {
            px: 0.25,
            ..ufo::Particle::default()
        }],
        &ufo::cubecl::CubeClTrackRunOptions {
            backend: ufo::cubecl::CubeClBackend::Cpu,
            ..ufo::cubecl::CubeClTrackRunOptions::default()
        },
    )
    .unwrap();

    assert_eq!(output.len(), 1);
    assert!(
        (output[0].x - 0.5).abs() < 1.0e-5,
        "unexpected output: {:?}",
        output
    );
    assert_eq!(output[0].passed_elements, 1);
    assert!(output[0].alive);
}

#[test]
fn covers_core_element_instructions_on_cubecl_cpu() {
    let tracking = compile_tracking(
        r#"
q: QUADRUPOLE, L=0.3, K1=1.2;
b: SBEND, L=0.5, ANGLE=0.05, K1=0.1, E1=0.01, E2=-0.02, HGAP=0.01, FINT=0.5;
d: DRIFT, L=0.2;
m: MULTIPOLE, KNL={0.01, 0.02}, KSL={0.005};
w: WIRE, X=0.01, Y=-0.02, K=0.0001;
c: CAVITY, FIELD=0.001, OMEGA=0.2, LAG=0.25;
a: APERTURE, RADIUS=0.1;
ring: LINE=(a,d,m,q,b,w,c);
"#,
        "ring",
        ufo::PassFlags::empty(),
    );
    let ops = opcodes(&tracking);
    for op in [
        ufo::OP_SET_APERTURE,
        ufo::OP_DRIFT,
        ufo::OP_KICK,
        ufo::OP_QUADRUPOLE,
        ufo::OP_SBEND,
        ufo::OP_EDGE,
        ufo::OP_WIRE,
        ufo::OP_CAVITY,
        ufo::OP_DUMP,
        ufo::OP_REWIND,
    ] {
        assert!(ops.contains(&op), "missing opcode {op}; got {ops:?}");
    }

    let output = track_cpu(
        &tracking,
        &[ufo::Particle {
            x: 0.001,
            px: 0.0002,
            y: -0.0007,
            py: 0.0003,
            z: 0.1,
            ..ufo::Particle::default()
        }],
    )
    .unwrap();

    assert_eq!(output.len(), 1);
    assert_finite_particle(&output[0]);
    assert!(output[0].alive);
    assert!(output[0].passed_elements >= 7);
    assert_ne!(output[0].x, 0.001);
    assert_ne!(output[0].dp, 0.0);
}

#[test]
fn covers_teapot_instruction_on_cubecl_cpu() {
    let tracking = compile_tracking(
        "s: SEXTUPOLE, L=0.4, K2=3.0; ring: LINE=(s);",
        "ring",
        ufo::PassFlags::empty(),
    );
    let ops = opcodes(&tracking);
    assert!(ops.contains(&ufo::OP_TEAPOT), "got {ops:?}");

    let output = track_cpu(
        &tracking,
        &[ufo::Particle {
            x: 0.01,
            y: 0.002,
            ..ufo::Particle::default()
        }],
    )
    .unwrap();

    assert_eq!(output.len(), 1);
    assert_finite_particle(&output[0]);
    assert!(output[0].alive);
    assert_eq!(output[0].passed_elements, 1);
    assert_ne!(output[0].px, 0.0);
}

#[test]
fn covers_tran_linear_instruction_on_cubecl_cpu() {
    let tracking = compile_tracking(
        "m: MULTIPOLE, KNL={0.0, 0.2}, DX=0.01, DY=-0.02; ring: LINE=(m);",
        "ring",
        ufo::PassFlags::LINEAR,
    );
    let ops = opcodes(&tracking);
    assert!(ops.contains(&ufo::OP_TRAN_LINEAR), "got {ops:?}");

    let output = track_cpu(
        &tracking,
        &[ufo::Particle {
            x: 0.001,
            y: -0.002,
            ..ufo::Particle::default()
        }],
    )
    .unwrap();

    assert_eq!(output.len(), 1);
    assert_finite_particle(&output[0]);
    assert!(output[0].alive);
    assert_eq!(output[0].passed_elements, 1);
    assert_ne!(output[0].px, 0.0);
}

#[test]
fn covers_full_lattice_fixture_on_cubecl_cpu() {
    let lattice = ufo::load_mad_file("optics/alba.mad").unwrap();
    let tracking = ufo::compile_tracking_line(
        &lattice,
        lattice.line("RING").unwrap(),
        &ufo::TrackCompileOptions {
            where_: vec![-1.0],
            ..ufo::TrackCompileOptions::default()
        },
    )
    .unwrap();
    let ops = opcodes(&tracking);
    for op in [
        ufo::OP_DRIFT,
        ufo::OP_QUADRUPOLE,
        ufo::OP_SBEND,
        ufo::OP_EDGE,
        ufo::OP_TEAPOT,
        ufo::OP_DUMP,
        ufo::OP_REWIND,
    ] {
        assert!(ops.contains(&op), "missing opcode {op}; got {ops:?}");
    }

    let output = track_cpu(
        &tracking,
        &[ufo::Particle {
            x: 1.0e-4,
            px: 2.0e-5,
            y: -1.0e-4,
            py: 1.0e-5,
            ..ufo::Particle::default()
        }],
    )
    .unwrap();

    assert_eq!(output.len(), 1);
    assert_finite_particle(&output[0]);
    assert!(output[0].alive);
    let flat_elements = lattice
        .line("RING")
        .unwrap()
        .flatten(&lattice)
        .unwrap()
        .len() as u32;
    assert!(output[0].passed_elements >= flat_elements);
}

#[test]
fn tracks_fractional_observation_inside_drift_on_cubecl_cpu() {
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

    let output = ufo::cubecl::track_with_options(
        &tracking,
        &[ufo::Particle {
            px: 0.25,
            ..ufo::Particle::default()
        }],
        &ufo::cubecl::CubeClTrackRunOptions {
            backend: ufo::cubecl::CubeClBackend::Cpu,
            ..ufo::cubecl::CubeClTrackRunOptions::default()
        },
    )
    .unwrap();

    assert_eq!(output.len(), 2);
    assert!(
        (output[0].x - 0.125).abs() < 1.0e-5,
        "unexpected output: {:?}",
        output
    );
    assert!(
        (output[1].x - 0.5).abs() < 1.0e-5,
        "unexpected output: {:?}",
        output
    );
}

#[test]
fn tracks_multiple_particles_on_cubecl_cpu() {
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

    let output = track_cpu(
        &tracking,
        &[
            ufo::Particle {
                px: 0.25,
                ..ufo::Particle::default()
            },
            ufo::Particle {
                px: -0.5,
                ..ufo::Particle::default()
            },
        ],
    )
    .unwrap();

    assert_eq!(output.len(), 2);
    assert!((output[0].x - 0.5).abs() < 1.0e-5);
    assert!((output[1].x + 1.0).abs() < 1.0e-5);
}

#[test]
fn tracks_one_drift_in_double_precision_on_cubecl_cpu() {
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

    let output = track_cpu(
        &tracking,
        &[ufo::Particle {
            px: 0.25,
            ..ufo::Particle::default()
        }],
    )
    .unwrap();

    assert!(tracking.bytecode.is_64bit());
    assert_eq!(output.len(), 1);
    assert!((output[0].x - 0.5).abs() < 1.0e-12);
}

#[test]
fn applies_aperture_check_on_cubecl_cpu() {
    let lattice =
        ufo::mad::parse_mad("a: APERTURE, RADIUS=0.1; d: DRIFT, L=1.0; ring: LINE=(a,d);").unwrap();
    let tracking = ufo::compile_tracking_line(
        &lattice,
        lattice.line("ring").unwrap(),
        &ufo::TrackCompileOptions {
            where_: vec![-1.0],
            ..ufo::TrackCompileOptions::default()
        },
    )
    .unwrap();

    let output = track_cpu(
        &tracking,
        &[ufo::Particle {
            x: 0.2,
            ..ufo::Particle::default()
        }],
    )
    .unwrap();

    assert_eq!(output.len(), 1);
    assert!(!output[0].alive);
    assert_eq!(output[0].passed_elements, 1);
}

#[test]
fn applies_multipole_kick_on_cubecl_cpu() {
    let lattice = ufo::mad::parse_mad("m: MULTIPOLE, KNL={0.1}; ring: LINE=(m);").unwrap();
    let tracking = ufo::compile_tracking_line(
        &lattice,
        lattice.line("ring").unwrap(),
        &ufo::TrackCompileOptions {
            where_: vec![-1.0],
            ..ufo::TrackCompileOptions::default()
        },
    )
    .unwrap();

    let output = track_cpu(&tracking, &[ufo::Particle::default()]).unwrap();

    assert_eq!(output.len(), 1);
    assert!((output[0].px + 0.1).abs() < 1.0e-6);
    assert_eq!(output[0].passed_elements, 1);
}
