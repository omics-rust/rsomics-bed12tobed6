/// Compatibility tests: rsomics-bed12tobed6 output must match bedtools bed12tobed6.
use std::path::PathBuf;
use std::process::Command;

fn bin_path() -> &'static str {
    env!("CARGO_BIN_EXE_rsomics-bed12tobed6")
}

fn golden(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden")
        .join(name)
}

fn bedtools_available() -> bool {
    Command::new("bedtools")
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success())
}

fn run_ours(input: &str, extra_args: &[&str]) -> String {
    let mut cmd = Command::new(bin_path());
    cmd.arg("-i").arg(input);
    cmd.args(extra_args);
    let out = cmd.output().expect("failed to run rsomics-bed12tobed6");
    assert!(
        out.status.success(),
        "exit {:?}\nstderr: {}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

fn run_bedtools(input: &str, extra_args: &[&str]) -> String {
    let mut cmd = Command::new("bedtools");
    cmd.args(["bed12tobed6", "-i", input]);
    cmd.args(extra_args);
    let out = cmd.output().expect("failed to run bedtools bed12tobed6");
    assert!(
        out.status.success(),
        "bedtools exit {:?}\nstderr: {}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

#[test]
fn golden_output_matches() {
    let input = golden("input.bed12");
    let got = run_ours(input.to_str().unwrap(), &[]);
    let expected = std::fs::read_to_string(golden("expected.bed")).unwrap();
    assert_eq!(
        got, expected,
        "output diverged from golden\ngot:\n{got}\nexpected:\n{expected}"
    );
}

#[test]
fn output_matches_bedtools() {
    if !bedtools_available() {
        eprintln!("skip: bedtools not found");
        return;
    }
    let input = golden("input.bed12");
    let ours = run_ours(input.to_str().unwrap(), &[]);
    let bt = run_bedtools(input.to_str().unwrap(), &[]);
    assert_eq!(
        ours, bt,
        "output diverged from bedtools\nours:\n{ours}\nbedtools:\n{bt}"
    );
}

#[test]
fn block_num_score_matches_bedtools() {
    if !bedtools_available() {
        eprintln!("skip: bedtools not found");
        return;
    }
    let input = golden("input.bed12");
    let ours = run_ours(input.to_str().unwrap(), &["-n"]);
    let bt = run_bedtools(input.to_str().unwrap(), &["-n"]);
    assert_eq!(
        ours, bt,
        "block-num-score output diverged from bedtools\nours:\n{ours}\nbedtools:\n{bt}"
    );
}
