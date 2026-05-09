use std::path::PathBuf;
use std::process::Command;

fn x_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_x"))
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

#[test]
fn documented_integration_category_form_runs_requested_category_only() {
    let out = Command::new(x_bin())
        .arg("test")
        .arg("integration")
        .arg("basic")
        .current_dir(repo_root())
        .output()
        .expect("x test integration basic should run");

    assert!(
        out.status.success(),
        "documented integration-category form failed.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("basic tests:"), "stdout was: {stdout}");
    assert!(!stdout.contains("functions tests:"), "stdout was: {stdout}");
    assert!(!stdout.contains("types tests:"), "stdout was: {stdout}");
    assert!(stdout.contains("5 passed; 0 failed; 0 skipped; 5 total"), "stdout was: {stdout}");
}

#[test]
fn non_integration_extra_positional_is_rejected() {
    let out = Command::new(x_bin())
        .arg("test")
        .arg("basic")
        .arg("extra")
        .current_dir(repo_root())
        .output()
        .expect("x test basic extra should run");

    assert!(
        !out.status.success(),
        "non-integration extra positional should fail.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("额外的测试分类 `extra` 仅支持与 `x test integration <category>` 一起使用"), "stderr was: {stderr}");
}

#[test]
fn stdlib_integration_suite_reports_stdin_io_case() {
    let out = Command::new(x_bin())
        .arg("test")
        .arg("integration")
        .arg("stdlib")
        .current_dir(repo_root())
        .output()
        .expect("x test integration stdlib should run");

    assert!(
        out.status.success(),
        "stdlib integration suite failed.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("stdlib tests:"), "stdout was: {stdout}");
    assert!(stdout.contains("io_test ... ok"), "stdout was: {stdout}");
    assert!(stdout.contains("io_unicode_test ... ok"), "stdout was: {stdout}");
    assert!(stdout.contains("io_whitespace_test ... ok"), "stdout was: {stdout}");
    assert!(stdout.contains("6 passed; 0 failed; 0 skipped; 6 total"), "stdout was: {stdout}");
}
