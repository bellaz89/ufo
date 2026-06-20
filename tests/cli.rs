use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn cli_dumps_lattice_to_mad_file() {
    let output = std::env::temp_dir().join(format!("ufo-dump-{}.mad", std::process::id()));
    let _ = std::fs::remove_file(&output);

    let mut cmd = Command::cargo_bin("ufo").unwrap();
    cmd.args(["dump", "optics/fodo.mad", output.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("wrote:"));

    let dumped = std::fs::read_to_string(&output).unwrap();
    assert!(dumped.contains("RING: LINE="));
    std::fs::remove_file(output).unwrap();
}

#[test]
fn cli_dumps_lattice_to_elegant_file() {
    let output = std::env::temp_dir().join(format!("ufo-dump-{}.lte", std::process::id()));
    let _ = std::fs::remove_file(&output);

    let mut cmd = Command::cargo_bin("ufo").unwrap();
    cmd.args([
        "dump",
        "optics/fodo.mad",
        output.to_str().unwrap(),
        "--style",
        "elegant",
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("wrote:"));

    let dumped = std::fs::read_to_string(&output).unwrap();
    assert!(dumped.contains("RING: LINE="));
    std::fs::remove_file(output).unwrap();
}

#[test]
fn cli_loads_lattice_summary() {
    let mut cmd = Command::cargo_bin("ufo").unwrap();
    cmd.args(["load", "optics/fodo.mad"])
        .assert()
        .success()
        .stdout(predicate::str::contains("elements:"))
        .stdout(predicate::str::contains("line: RING"));
}

#[test]
fn cli_compile_reports_bytecode_metadata() {
    let mut cmd = Command::cargo_bin("ufo").unwrap();
    cmd.args([
        "compile",
        "optics/fodo.mad",
        "--flag",
        "linear",
        "--flag",
        "achromatic",
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("instructions:"))
    .stdout(predicate::str::contains("words:"));
}

#[test]
fn cli_compile_prints_decoded_instructions() {
    let mut cmd = Command::cargo_bin("ufo").unwrap();
    cmd.args(["compile", "optics/fodo.mad", "--instructions"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "index,opcode,name,kind,flags,aux,args",
        ))
        .stdout(predicate::str::contains(",drift,"))
        .stdout(predicate::str::contains(",rewind,"));
}

#[test]
fn cli_compile_help_exposes_instruction_listing() {
    let mut cmd = Command::cargo_bin("ufo").unwrap();
    cmd.args(["compile", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--instructions"))
        .stdout(predicate::str::contains("--hex"));
}

#[test]
#[cfg(feature = "cubecl")]
fn cli_accepts_list_devices_spelling() {
    let mut cmd = Command::cargo_bin("ufo").unwrap();
    cmd.arg("list_devices")
        .assert()
        .success()
        .stdout(predicate::str::contains(":").or(predicate::str::is_empty()));
}

#[test]
#[cfg(feature = "cubecl")]
fn cli_exposes_track_command() {
    let mut cmd = Command::cargo_bin("ufo").unwrap();
    cmd.args(["track", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--turns"))
        .stdout(predicate::str::contains("--where"))
        .stdout(predicate::str::contains("--particle"))
        .stdout(predicate::str::contains("--particles-file"))
        .stdout(predicate::str::contains("--random"))
        .stdout(predicate::str::contains("--grid"))
        .stdout(predicate::str::contains("--backend"))
        .stdout(predicate::str::contains("--device"));
}

#[test]
#[cfg(feature = "cubecl")]
fn cli_exposes_optics_command() {
    let mut cmd = Command::cargo_bin("ufo").unwrap();
    cmd.args(["optics", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--propagate"))
        .stdout(predicate::str::contains("--where"))
        .stdout(predicate::str::contains("--backend"))
        .stdout(predicate::str::contains("--device"));
}

#[test]
#[cfg(feature = "cubecl")]
fn cli_exposes_chromaticity_command() {
    let mut cmd = Command::cargo_bin("ufo").unwrap();
    cmd.args(["chromaticity", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--backend"))
        .stdout(predicate::str::contains("--flag"));
}

#[test]
#[cfg(feature = "cubecl")]
fn cli_exposes_radiation_command() {
    let mut cmd = Command::cargo_bin("ufo").unwrap();
    cmd.args(["radiation", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--backend"))
        .stdout(predicate::str::contains("--flag"));
}

#[test]
#[cfg(feature = "cubecl")]
fn cli_exposes_closed_orbit_command() {
    let mut cmd = Command::cargo_bin("ufo").unwrap();
    cmd.args(["closed-orbit", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--iterations"))
        .stdout(predicate::str::contains("--step"));
}

#[test]
#[cfg(feature = "cubecl")]
fn cli_exposes_rdt_command() {
    let mut cmd = Command::cargo_bin("ufo").unwrap();
    cmd.args(["rdt", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--backend"))
        .stdout(predicate::str::contains("--flag"));
}

#[test]
#[cfg(feature = "cubecl")]
fn cli_exposes_stable_aperture_command() {
    let mut cmd = Command::cargo_bin("ufo").unwrap();
    cmd.args(["stable-aperture", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--x-count"))
        .stdout(predicate::str::contains("--y-count"))
        .stdout(predicate::str::contains("--device"));
}
