use std::path::PathBuf;
use std::process::Command;

fn x_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_x"))
}

fn repo_root() -> &'static str {
    "C:\\Users\\Administrator\\Documents\\x-lang"
}

#[test]
fn new_binary_scaffold_uses_current_syntax_and_checks() {
    let bin = x_bin();
    let dir = tempfile::tempdir().expect("tempdir");

    let create = Command::new(&bin)
        .arg("new")
        .arg("hello-app")
        .arg("--vcs")
        .arg("none")
        .current_dir(dir.path())
        .output()
        .expect("x new should run");
    assert!(
        create.status.success(),
        "x new failed.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&create.stdout),
        String::from_utf8_lossy(&create.stderr)
    );

    let main_file = dir.path().join("hello-app").join("src").join("main.x");
    let main_source = std::fs::read_to_string(&main_file).expect("read main.x");
    assert!(main_source.contains("function main() -> Unit"), "main.x was: {main_source}");
    assert!(main_source.contains("println(\"Hello, X!\")"), "main.x was: {main_source}");
    assert!(!main_source.contains("fun main"), "main.x was: {main_source}");
    assert!(!main_source.contains("print(\"Hello, world!\")"), "main.x was: {main_source}");

    let check = Command::new(&bin)
        .arg("check")
        .arg(&main_file)
        .env("X_ROOT", repo_root())
        .output()
        .expect("x check should run");
    assert!(
        check.status.success(),
        "x check failed.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );
}

#[test]
fn init_binary_scaffold_uses_current_syntax_and_checks() {
    let bin = x_bin();
    let dir = tempfile::tempdir().expect("tempdir");

    let init = Command::new(&bin)
        .arg("init")
        .arg("--vcs")
        .arg("none")
        .current_dir(dir.path())
        .output()
        .expect("x init should run");
    assert!(
        init.status.success(),
        "x init failed.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&init.stdout),
        String::from_utf8_lossy(&init.stderr)
    );

    let main_file = dir.path().join("src").join("main.x");
    let main_source = std::fs::read_to_string(&main_file).expect("read main.x");
    assert!(main_source.contains("function main() -> Unit"), "main.x was: {main_source}");
    assert!(main_source.contains("println(\"Hello, X!\")"), "main.x was: {main_source}");
    assert!(!main_source.contains("fun main"), "main.x was: {main_source}");

    let check = Command::new(&bin)
        .arg("check")
        .arg(&main_file)
        .env("X_ROOT", repo_root())
        .output()
        .expect("x check should run");
    assert!(
        check.status.success(),
        "x check failed.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );
}

#[test]
fn new_library_scaffold_uses_current_type_names() {
    let bin = x_bin();
    let dir = tempfile::tempdir().expect("tempdir");

    let create = Command::new(&bin)
        .arg("new")
        .arg("hello-lib")
        .arg("--lib")
        .arg("--vcs")
        .arg("none")
        .current_dir(dir.path())
        .output()
        .expect("x new --lib should run");
    assert!(
        create.status.success(),
        "x new --lib failed.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&create.stdout),
        String::from_utf8_lossy(&create.stderr)
    );

    let lib_file = dir.path().join("hello-lib").join("src").join("lib.x");
    let lib_source = std::fs::read_to_string(&lib_file).expect("read lib.x");
    assert!(
        lib_source.contains("function add(a: integer, b: integer) -> integer"),
        "lib.x was: {lib_source}"
    );
    assert!(!lib_source.contains("fun add"), "lib.x was: {lib_source}");
    assert!(!lib_source.contains("Int"), "lib.x was: {lib_source}");
}
