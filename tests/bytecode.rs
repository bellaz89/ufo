use ufo::{
    Bytecode, Instruction, OP_CAVITY, OP_DUMP, OP_EDGE, OP_NEXT_OFFSET, OP_SET_APERTURE,
    OP_TRAN_LINEAR, PassFlags,
};

#[test]
fn rewind_encoding_matches_interpreter_layout() {
    let bytecode = Bytecode::from_instructions(vec![Instruction::rewind("rw")], false);
    assert_eq!(bytecode.emit_bytes().unwrap(), vec![255, 128, 0, 0]);
}

#[test]
fn simple_sequence_has_expected_word_count() {
    let bytecode = Bytecode::from_instructions(
        vec![
            Instruction::drift("d", 1.2, true),
            Instruction::kick("k", vec![0.0, 0.1], vec![], false, false),
            Instruction::rewind("rw"),
        ],
        false,
    );
    assert_eq!(bytecode.word_count(), 6);
    assert_eq!(bytecode.emit_bytes().unwrap().len(), 24);
}

#[test]
fn binds_remaining_kernel_opcodes() {
    let identity = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];
    let instructions = vec![
        Instruction::next_offset("next"),
        Instruction::dump("dump"),
        Instruction::edge("edge", 1.0, 0.1, 0.01, 0.0, false),
        Instruction::cavity("cav", 1.0, 2.0, 0.25),
        Instruction::tran_linear("tl", identity, [1.0, 0.0, 0.0, 0.0], true),
        Instruction::set_aperture("ap", 0.02),
    ];
    let ops = instructions.iter().map(|i| i.op).collect::<Vec<_>>();
    assert_eq!(
        ops,
        vec![
            OP_NEXT_OFFSET,
            OP_DUMP,
            OP_EDGE,
            OP_CAVITY,
            OP_TRAN_LINEAR,
            OP_SET_APERTURE
        ]
    );
    assert!(
        !Bytecode::from_instructions(instructions, false)
            .emit_u32_words()
            .unwrap()
            .is_empty()
    );
}

#[test]
fn double_precision_flag_selects_64_bit_bytecode() {
    let lattice = ufo::mad::parse_mad("d: DRIFT, L=1.0; ring: LINE=(d);").unwrap();
    let bytecode = ufo::compile_line(
        &lattice,
        lattice.line("ring").unwrap(),
        &ufo::compiler::CompileOptions {
            flags: PassFlags::DOUBLE_PRECISION,
            turns: 1,
            is_64bit: false,
            collapse_linear: false,
        },
    )
    .unwrap();
    assert!(bytecode.is_64bit());
}

#[test]
fn default_compile_options_keep_opencl_bytecode_single_precision() {
    let lattice = ufo::mad::parse_mad(
        r#"
d: DRIFT, L=1.0;
ring: LINE=(d);
"#,
    )
    .unwrap();

    let bytecode = ufo::compile_line(
        &lattice,
        lattice.line("ring").unwrap(),
        &ufo::compiler::CompileOptions::default(),
    )
    .unwrap();

    assert!(!bytecode.is_64bit());
    assert_eq!(bytecode.word_size(), 4);
}
