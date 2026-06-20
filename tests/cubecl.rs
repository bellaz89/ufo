#![cfg(feature = "cubecl-cpu")]

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
