use ufo::{
    Element, OP_CAVITY, OP_DRIFT, OP_DUMP, OP_EDGE, OP_KICK, OP_QUADRUPOLE, OP_SBEND,
    OP_SET_APERTURE, OP_TRAN_LINEAR, PassFlags, compile_line, compile_tracking_line, load_mad_file,
    to_mad_string,
};

#[test]
fn loads_all_current_optics_fixtures() {
    for path in [
        "optics/align.mad",
        "optics/alba.mad",
        "optics/bend.mad",
        "optics/drift.mad",
        "optics/elements.mad",
        "optics/fodo.mad",
        "optics/fodo2.mad",
        "optics/multipole.mad",
        "optics/oct.mad",
        "optics/triplet.mad",
    ] {
        let lattice = load_mad_file(path).unwrap_or_else(|err| panic!("{path}: {err}"));
        assert!(
            !lattice.elements.is_empty() || !lattice.lines.is_empty(),
            "{path} parsed to an empty lattice"
        );
    }
}

#[test]
fn load_mad_file_expands_call_file_includes() {
    let lattice = load_mad_file("tests/twiss.mad").unwrap();
    let ring = lattice.line("RING").unwrap();
    assert!(ring.flatten(&lattice).unwrap().len() > 100);
    assert!(ring.length(&lattice).unwrap() > 0.0);
}

#[test]
fn load_mad_file_ignores_comments_while_expanding_includes() {
    let path = std::env::temp_dir().join(format!("ufo-comments-{}.mad", std::process::id()));
    let fixture = std::env::current_dir().unwrap().join("optics/drift.mad");
    std::fs::write(
        &path,
        format!(
            r#"
! comment with a semicolon ; should not split statements
CALL, FILE="{}"; ! include drift fixture
BEAM, SEQUENCE=RING;
"#,
            fixture.display()
        ),
    )
    .unwrap();

    let lattice = load_mad_file(&path).unwrap();
    assert!(!lattice.elements.is_empty() || !lattice.lines.is_empty());
    std::fs::remove_file(path).unwrap();
}

#[test]
fn alba_ring_flattens_and_has_positive_length() {
    let lattice = load_mad_file("optics/alba.mad").unwrap();
    let ring = lattice.line("RING").unwrap();
    let flat = ring.flatten(&lattice).unwrap();
    assert!(flat.len() > 100);
    assert!(ring.length(&lattice).unwrap() > 0.0);
}

#[test]
fn parses_multipole_vectors() {
    let lattice = load_mad_file("optics/multipole.mad").unwrap();
    let multipoles = lattice
        .elements
        .values()
        .filter_map(|element| match element {
            Element::Multipole(v) => Some(v),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(!multipoles.is_empty());
    assert!(
        multipoles
            .iter()
            .any(|m| !m.knl.is_empty() || !m.ksl.is_empty())
    );
}

#[test]
fn compiles_fodo_ring_to_bytecode() {
    let lattice = load_mad_file("optics/fodo.mad").unwrap();
    let ring = lattice.line("RING").unwrap();
    let bytecode = compile_line(
        &lattice,
        ring,
        &ufo::compiler::CompileOptions {
            flags: PassFlags::LINEAR | PassFlags::ACHROMATIC,
            turns: 1,
            is_64bit: false,
            collapse_linear: false,
        },
    )
    .unwrap();
    assert!(bytecode.word_count() > 1);
    assert!(!bytecode.emit_u32_words().unwrap().is_empty());
}

#[test]
fn compiler_emits_newly_bound_kernel_instructions() {
    let lattice = ufo::mad::parse_mad(
        r#"
b: SBEND, L=1.0, ANGLE=0.1, K1=0.2, E1=0.01, E2=0.02, HGAP=0.03, FINT=0.7;
c: CAVITY, FIELD=1.0, OMEGA=2.0, LAG=0.5;
a: APERTURE, RADIUS=0.01;
ring: LINE=(b, c, a);
"#,
    )
    .unwrap();
    let bytecode = compile_line(
        &lattice,
        lattice.line("ring").unwrap(),
        &ufo::compiler::CompileOptions::default(),
    )
    .unwrap();
    let ops = bytecode
        .instructions()
        .iter()
        .map(|instruction| instruction.op)
        .collect::<Vec<_>>();
    assert!(ops.iter().filter(|op| **op == OP_EDGE).count() >= 2);
    assert!(ops.contains(&OP_CAVITY));
    assert!(ops.contains(&OP_SET_APERTURE));
}

#[test]
fn compiler_emits_alignment_transforms_for_offsets() {
    let lattice = ufo::mad::parse_mad(
        r#"
m: MULTIPOLE, KNL={0.0, 1.0}, DX=0.1, DY=-0.2;
ring: LINE=(m);
"#,
    )
    .unwrap();
    let bytecode = compile_line(
        &lattice,
        lattice.line("ring").unwrap(),
        &ufo::compiler::CompileOptions::default(),
    )
    .unwrap();
    let transforms = bytecode
        .instructions()
        .iter()
        .filter(|instruction| instruction.op == OP_TRAN_LINEAR)
        .count();
    assert_eq!(transforms, 2);
}

#[test]
fn compiler_emits_linear_multipole_kick_as_tran_linear() {
    let lattice = ufo::mad::parse_mad(
        r#"
m: MULTIPOLE, KNL={0.1, 0.2}, KSL={0.3, 0.4};
ring: LINE=(m);
"#,
    )
    .unwrap();

    let nonlinear = compile_line(
        &lattice,
        lattice.line("ring").unwrap(),
        &ufo::compiler::CompileOptions::default(),
    )
    .unwrap();
    assert!(
        nonlinear
            .instructions()
            .iter()
            .any(|instruction| instruction.op == OP_KICK)
    );

    let linear = compile_line(
        &lattice,
        lattice.line("ring").unwrap(),
        &ufo::compiler::CompileOptions {
            flags: PassFlags::LINEAR,
            ..ufo::compiler::CompileOptions::default()
        },
    )
    .unwrap();
    assert!(
        linear
            .instructions()
            .iter()
            .any(|instruction| instruction.op == OP_TRAN_LINEAR)
    );
    assert!(
        !linear
            .instructions()
            .iter()
            .any(|instruction| instruction.op == OP_KICK)
    );
}

#[test]
fn compiler_collapses_affine_linear_transforms_with_offsets() {
    let lattice = ufo::mad::parse_mad(
        r#"
m: MULTIPOLE, KNL={0.0, 1.0}, DX=1.0;
ring: LINE=(m);
"#,
    )
    .unwrap();

    let bytecode = compile_line(
        &lattice,
        lattice.line("ring").unwrap(),
        &ufo::compiler::CompileOptions {
            flags: PassFlags::LINEAR,
            collapse_linear: true,
            ..ufo::compiler::CompileOptions::default()
        },
    )
    .unwrap();
    let instructions = bytecode.instructions();
    assert_eq!(instructions.len(), 2);
    assert_eq!(instructions[0].op, OP_TRAN_LINEAR);

    // entry x -= 1, kick px -= x, exit x += 1 collapses to px' = px - x + 1.
    assert!((instructions[0].args[2] - 1.0).abs() < 1.0e-12);
    assert!((instructions[0].args[4] + 1.0).abs() < 1.0e-12);
}

#[test]
fn compiler_collapses_linear_element_opcodes_to_tran_linear() {
    let lattice = ufo::mad::parse_mad(
        r#"
d: DRIFT, L=0.5;
q: QUADRUPOLE, L=0.4, K1=0.8;
b: SBEND, L=0.7, ANGLE=0.05, K1=0.1, E1=0.01, E2=0.02;
ring: LINE=(d,q,b);
"#,
    )
    .unwrap();

    let bytecode = compile_line(
        &lattice,
        lattice.line("ring").unwrap(),
        &ufo::compiler::CompileOptions {
            flags: PassFlags::LINEAR | PassFlags::ACHROMATIC,
            collapse_linear: true,
            ..ufo::compiler::CompileOptions::default()
        },
    )
    .unwrap();
    let ops = bytecode
        .instructions()
        .iter()
        .map(|instruction| instruction.op)
        .collect::<Vec<_>>();

    assert!(ops.iter().any(|op| *op == OP_TRAN_LINEAR));
    assert!(
        !ops.iter()
            .any(|op| matches!(*op, OP_DRIFT | OP_QUADRUPOLE | OP_SBEND | OP_EDGE))
    );
}

#[test]
fn parses_field_error_vectors_and_scalar_aliases() {
    let lattice = ufo::mad::parse_mad(
        r#"
q: QUADRUPOLE, L=1.0, K1=0.2, DKN={0.1, 0.2}, DKS={0.3}, DK3=0.4, DK2S=0.5;
s: SEXTUPOLE, L=1.0, K2=0.1, DX=0.01, DY=-0.02, DK1=0.7;
ring: LINE=(q, s);
"#,
    )
    .unwrap();

    let q = match lattice.elements.get("q").unwrap() {
        Element::Quadrupole(v) => v,
        other => panic!("unexpected element: {other:?}"),
    };
    assert_eq!(q.dkn, vec![0.1, 0.2, 0.0, 0.4]);
    assert_eq!(q.dks, vec![0.3, 0.0, 0.5]);

    let s = match lattice.elements.get("s").unwrap() {
        Element::Sextupole(v) => v,
        other => panic!("unexpected element: {other:?}"),
    };
    assert_eq!(s.dkn, vec![0.0, 0.7]);
    assert_eq!(s.dx, 0.01);
    assert_eq!(s.dy, -0.02);
}

#[test]
fn dumps_lattice_to_mad_string_that_can_be_reparsed() {
    let lattice = ufo::mad::parse_mad(
        r#"
d: DRIFT, L=1.5;
q: QUADRUPOLE, L=2.0, K1=0.3, DX=0.01, DKN={0.0, 0.2};
ring: LINE=(d, q);
"#,
    )
    .unwrap();

    let output = to_mad_string(&lattice);
    assert!(output.contains("d: DRIFT, L=1.5;"));
    assert!(output.contains("q: QUADRUPOLE"));
    assert!(output.contains("DX=0.01"));
    assert!(output.contains("DKN={0, 0.2}"));

    let reparsed = ufo::mad::parse_mad(&output).unwrap();
    assert_eq!(
        reparsed
            .line("ring")
            .unwrap()
            .flatten(&reparsed)
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn dumps_lattice_to_elegant_and_reports_unsupported_styles() {
    let lattice = ufo::mad::parse_mad(
        r#"
d: DRIFT, L=1.5;
q: QUADRUPOLE, L=2.0, K1=0.3;
ring: LINE=(d, q);
"#,
    )
    .unwrap();

    let elegant = ufo::to_lattice_string(&lattice, ufo::DumpStyle::Elegant).unwrap();
    assert!(elegant.contains("d: DRIFT, L=1.5;"));
    assert!(elegant.contains("ring: LINE=(d, q)"));

    let with_multipole = ufo::mad::parse_mad(
        r#"
m: MULTIPOLE, KNL={0, 1};
ring: LINE=(m);
"#,
    )
    .unwrap();
    let error = ufo::to_lattice_string(&with_multipole, ufo::DumpStyle::At).unwrap_err();
    assert!(error.to_string().contains("at export"));
}

#[test]
fn dumps_rbend_to_opa_bending() {
    let lattice = ufo::mad::parse_mad(
        r#"
b: RBEND, L=1.2, ANGLE=0.1, K1=0.2, E1=0.01, E2=0.02;
ring: LINE=(b);
"#,
    )
    .unwrap();

    let opa = ufo::to_lattice_string(&lattice, ufo::DumpStyle::Opa).unwrap();
    assert!(opa.contains("b : bending, l = 1.2"));
    assert!(opa.contains("k = 0.2"));
    assert!(opa.contains("t = 5.729577951308232"));
    assert!(opa.contains("t1 = 0.5729577951308232"));
    assert!(opa.contains("t2 = 1.1459155902616465"));
}

#[test]
fn line_find_and_locate_match_legacy_index_semantics() {
    let lattice = ufo::mad::parse_mad(
        r#"
d1: DRIFT, L=1.0;
q: QUADRUPOLE, L=2.0, K1=0.3;
d2: DRIFT, L=3.0;
ring: LINE=(d1, q, d2);
"#,
    )
    .unwrap();
    let ring = lattice.line("ring").unwrap();

    let drifts = ring
        .find(&lattice, |element| matches!(element, Element::Drift(_)))
        .unwrap();
    assert_eq!(drifts, vec![0, 2]);
    assert_eq!(ring.locate(&lattice, 0.0).unwrap(), Some(0.0));
    assert_eq!(ring.locate(&lattice, 1.5).unwrap(), Some(2.0));
    assert_eq!(ring.locate(&lattice, -1.0).unwrap(), Some(6.0));
    assert_eq!(ring.locate(&lattice, 3.0).unwrap(), None);
}

#[test]
fn line_count_and_survey_follow_flattened_elements() {
    let lattice = ufo::mad::parse_mad(
        r#"
d: DRIFT, L=2.0;
b: SBEND, L=1.0, ANGLE=1.5707963267948966;
ring: LINE=(d, b);
"#,
    )
    .unwrap();
    let ring = lattice.line("ring").unwrap();

    assert_eq!(ring.count(&lattice).unwrap(), 2);
    let survey = ring.survey(&lattice, [0.0, 0.0], 0.0).unwrap();
    assert_eq!(survey.len(), 2);
    assert!((survey[0].position[0] - 2.0).abs() < 1e-12);
    assert!(survey[0].position[1].abs() < 1e-12);
    assert!((survey[1].position[0] - 2.0).abs() < 1e-12);
    assert!((survey[1].position[1] - 1.0).abs() < 1e-12);
    assert!((survey[1].alpha - std::f64::consts::FRAC_PI_2).abs() < 1e-12);
}

#[test]
fn tracking_compiler_emits_boundary_and_end_dumps() {
    let lattice = load_mad_file("optics/fodo.mad").unwrap();
    let ring = lattice.line("RING").unwrap();
    let tracking = compile_tracking_line(
        &lattice,
        ring,
        &ufo::TrackCompileOptions {
            flags: PassFlags::LINEAR | PassFlags::ACHROMATIC,
            turns: 2,
            is_64bit: false,
            where_: vec![1.0, -1.0],
            collapse_linear: false,
        },
    )
    .unwrap();
    let dump_count = tracking
        .bytecode
        .instructions()
        .iter()
        .filter(|instruction| instruction.op == OP_DUMP)
        .count();
    assert_eq!(tracking.dumps_per_turn, 2);
    assert_eq!(dump_count, 2);
}

#[test]
fn tracking_compiler_splits_length_elements_for_fractional_observations() {
    let lattice = load_mad_file("optics/fodo.mad").unwrap();
    let ring = lattice.line("RING").unwrap();
    let tracking = compile_tracking_line(
        &lattice,
        ring,
        &ufo::TrackCompileOptions {
            where_: vec![0.5],
            ..ufo::TrackCompileOptions::default()
        },
    )
    .unwrap();
    assert_eq!(tracking.dumps_per_turn, 1);
    assert!(
        tracking
            .bytecode
            .instructions()
            .iter()
            .any(|instruction| instruction.op == OP_DUMP)
    );
}
