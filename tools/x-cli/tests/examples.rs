//! CLI 冒烟测试：确保 `x check` 与 `x run` 的关键路径可用。
//!
//! 注意：仓库根 `examples/*.x` 可能包含“规范/未来语法”用例，
//! 不一定与当前 parser/typechecker 的能力完全对齐，因此这里使用最小自包含的源码做回归。

use std::path::PathBuf;
use std::process::Command;

fn x_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_x"))
}

fn write_program(dir: &tempfile::TempDir, name: &str, source: &str) -> PathBuf {
    let file = dir.path().join(name);
    std::fs::write(&file, source).expect("write");
    file
}

fn command_without_x_root(bin: &PathBuf) -> Command {
    let mut command = Command::new(bin);
    command.env_remove("X_ROOT");
    command
}

fn zig_available() -> bool {
    Command::new("zig")
        .arg("version")
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false)
}

#[test]
fn smoke_check() {
    let bin = x_bin();
    let dir = tempfile::tempdir().expect("tempdir");
    let file = dir.path().join("smoke.x");
    std::fs::write(&file, "function main() { println(\"hi\") }\n").expect("write");

    let out = Command::new(&bin)
        .arg("check")
        .arg(&file)
        .output()
        .expect("执行 x check 失败");
    if !out.status.success() {
        panic!(
            "x check failed.\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stdout.trim().is_empty(), "stdout was: {stdout}");
    assert!(stderr.contains("Finished"), "stderr was: {stderr}");
}

#[test]
fn smoke_run() {
    let bin = x_bin();
    let dir = tempfile::tempdir().expect("tempdir");
    let file = dir.path().join("smoke.x");
    std::fs::write(&file, "function main() { println(\"hi\") }\n").expect("write");

    let out = Command::new(&bin)
        .arg("run")
        .arg(&file)
        .output()
        .expect("执行 x run 失败");
    if !out.status.success() {
        panic!(
            "x run failed.\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(stdout.trim(), "hi", "stdout was: {stdout}");
    assert!(stderr.contains("Finished"), "stderr was: {stderr}");
}

#[test]
fn smoke_check_top_level_prelude_call() {
    let bin = x_bin();
    let dir = tempfile::tempdir().expect("tempdir");
    let file = dir.path().join("top_level.x");
    std::fs::write(&file, "println(\"hi\")\n").expect("write");

    let out = Command::new(&bin)
        .arg("check")
        .arg(&file)
        .output()
        .expect("执行 x check 失败");
    if !out.status.success() {
        panic!(
            "x check failed for top-level prelude call.\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

#[test]
fn smoke_compile_emit_ast_top_level_prelude_call() {
    let bin = x_bin();
    let dir = tempfile::tempdir().expect("tempdir");
    let file = dir.path().join("emit_ast.x");
    std::fs::write(&file, "println(\"hi\")\n").expect("write");

    let out = Command::new(&bin)
        .arg("compile")
        .arg(&file)
        .arg("--emit")
        .arg("ast")
        .output()
        .expect("执行 x compile --emit ast 失败");
    if !out.status.success() {
        panic!(
            "x compile --emit ast failed.\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stdout.contains("println"), "stdout was: {stdout}");
    assert!(stdout.contains("String("), "stdout was: {stdout}");
    assert!(stderr.trim().is_empty(), "stderr was: {stderr}");
}

#[test]
fn smoke_run_with_string_interpolation() {
    let bin = x_bin();
    let dir = tempfile::tempdir().expect("tempdir");
    let file = dir.path().join("interpolation.x");
    std::fs::write(
        &file,
        "function main() { let name = \"World\"; println(\"Hello, ${name}!\") }\n",
    )
    .expect("write");

    let out = Command::new(&bin)
        .arg("run")
        .arg(&file)
        .output()
        .expect("执行 x run 失败");
    if !out.status.success() {
        panic!(
            "x run with interpolation failed.\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Hello, World!"), "stdout was: {stdout}");
}

#[test]
fn smoke_run_top_level_prelude_call() {
    let bin = x_bin();
    let dir = tempfile::tempdir().expect("tempdir");
    let file = dir.path().join("run_top_level.x");
    std::fs::write(&file, "println(\"hi from run\")\n").expect("write");

    let out = Command::new(&bin)
        .arg("run")
        .arg(&file)
        .output()
        .expect("执行 x run 失败");
    if !out.status.success() {
        panic!(
            "x run failed for top-level prelude call.\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("hi from run"), "stdout was: {stdout}");
}

#[test]
fn smoke_compile_typescript_no_link_with_interpolation() {
    let bin = x_bin();
    let dir = tempfile::tempdir().expect("tempdir");
    let file = dir.path().join("compile_ts.x");
    let out_file = dir.path().join("compiled_ts");
    std::fs::write(
        &file,
        "function main() { let name = \"World\"; println(\"Hello, ${name}!\") }\n",
    )
    .expect("write");

    let out = Command::new(&bin)
        .arg("compile")
        .arg(&file)
        .arg("--target")
        .arg("ts")
        .arg("--no-link")
        .arg("-o")
        .arg(&out_file)
        .output()
        .expect("执行 x compile --target ts --no-link 失败");
    if !out.status.success() {
        panic!(
            "x compile --target ts --no-link failed.\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stdout.trim().is_empty(), "stdout was: {stdout}");
    assert!(stderr.contains("已生成TypeScript代码"), "stderr was: {stderr}");

    let generated = std::fs::read_to_string(out_file.with_extension("ts")).expect("read ts");
    assert!(!generated.is_empty(), "generated TypeScript should not be empty");
}

#[test]
fn smoke_compile_emit_ast_with_local_import() {
    let bin = x_bin();
    let dir = tempfile::tempdir().expect("tempdir");
    let module_file = dir.path().join("greetings.x");
    let main_file = dir.path().join("main.x");

    std::fs::write(
        &module_file,
        "function message() -> String { \"hello from module\" }\n",
    )
    .expect("write module");
    std::fs::write(
        &main_file,
        "import greetings.{message};\nprintln(message())\n",
    )
    .expect("write main");

    let out = Command::new(&bin)
        .arg("compile")
        .arg(&main_file)
        .arg("--emit")
        .arg("ast")
        .current_dir(dir.path())
        .output()
        .expect("执行 x compile --emit ast 失败");
    if !out.status.success() {
        panic!(
            "x compile --emit ast with import failed.\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("message"), "stdout was: {stdout}");
    assert!(stdout.contains("hello from module"), "stdout was: {stdout}");
}

#[test]
fn smoke_build_with_local_import() {
    if !zig_available() {
        return;
    }

    let bin = x_bin();
    let dir = tempfile::tempdir().expect("tempdir");
    let manifest = dir.path().join("x.toml");
    let src_dir = dir.path().join("src");
    std::fs::create_dir_all(&src_dir).expect("create src");

    std::fs::write(
        &manifest,
        "[package]\nname = \"smoke-build\"\nversion = \"0.1.0\"\n",
    )
    .expect("write manifest");
    std::fs::write(
        src_dir.join("greetings.x"),
        "function message() -> String { \"hello build\" }\n",
    )
    .expect("write module");
    std::fs::write(
        src_dir.join("main.x"),
        "import greetings.{message};\nfunction main() { println(message()) }\n",
    )
    .expect("write main");

    let out = Command::new(&bin)
        .arg("build")
        .current_dir(dir.path())
        .output()
        .expect("执行 x build 失败");
    if !out.status.success() {
        panic!(
            "x build failed.\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

#[test]
fn smoke_check_with_local_import() {
    let bin = x_bin();
    let dir = tempfile::tempdir().expect("tempdir");
    let module_file = dir.path().join("greetings.x");
    let main_file = dir.path().join("main_check.x");

    std::fs::write(
        &module_file,
        "function message() -> String { \"hello from module\" }\n",
    )
    .expect("write module");
    std::fs::write(
        &main_file,
        "import greetings.{message};\nprintln(message())\n",
    )
    .expect("write main");

    let out = Command::new(&bin)
        .arg("check")
        .arg(&main_file)
        .current_dir(dir.path())
        .output()
        .expect("执行 x check 失败");
    if !out.status.success() {
        panic!(
            "x check with import failed.\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

#[test]
fn smoke_run_with_local_import() {
    let bin = x_bin();
    let dir = tempfile::tempdir().expect("tempdir");
    let module_file = dir.path().join("greetings.x");
    let main_file = dir.path().join("main_run.x");

    std::fs::write(
        &module_file,
        "function message() -> String { \"hello from module\" }\n",
    )
    .expect("write module");
    std::fs::write(
        &main_file,
        "import greetings.{message};\nprintln(message())\n",
    )
    .expect("write main");

    let out = Command::new(&bin)
        .arg("run")
        .arg(&main_file)
        .current_dir(dir.path())
        .output()
        .expect("执行 x run 失败");
    if !out.status.success() {
        panic!(
            "x run with import failed.\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("hello from module"), "stdout was: {stdout}");
}

#[test]
fn quiet_suppresses_helper_status_output_for_check() {
    let bin = x_bin();
    let dir = tempfile::tempdir().expect("tempdir");
    let file = write_program(&dir, "quiet_check.x", "function main() { println(\"hi\") }\n");

    let out = Command::new(&bin)
        .arg("--quiet")
        .arg("check")
        .arg(&file)
        .output()
        .expect("执行 x --quiet check 失败");

    if !out.status.success() {
        panic!(
            "x --quiet check failed.\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!stdout.contains("Finished"), "stdout was: {stdout}");
    assert!(stderr.trim().is_empty(), "stderr was: {stderr}");
}

#[test]
fn quiet_preserves_program_stdout_for_run() {
    let bin = x_bin();
    let dir = tempfile::tempdir().expect("tempdir");
    let file = write_program(
        &dir,
        "quiet_run.x",
        "function main() { println(\"quiet run output\") }\n",
    );

    let out = Command::new(&bin)
        .arg("--quiet")
        .arg("run")
        .arg(&file)
        .output()
        .expect("执行 x --quiet run 失败");

    if !out.status.success() {
        panic!(
            "x --quiet run failed.\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stdout.contains("quiet run output"), "stdout was: {stdout}");
    assert!(!stdout.contains("Running"), "stdout was: {stdout}");
    assert!(!stdout.contains("Finished"), "stdout was: {stdout}");
    assert!(stderr.trim().is_empty(), "stderr was: {stderr}");
}

#[test]
fn verbose_does_not_break_machine_readable_metadata_output() {
    let bin = x_bin();
    let dir = tempfile::tempdir().expect("tempdir");

    std::fs::write(
        dir.path().join("x.toml"),
        "[package]\nname = \"smoke-meta\"\nversion = \"0.1.0\"\n",
    )
    .expect("write manifest");
    std::fs::create_dir_all(dir.path().join("src")).expect("create src");
    std::fs::write(
        dir.path().join("src/main.x"),
        "function main() { println(\"hi\") }\n",
    )
    .expect("write main");

    let out = Command::new(&bin)
        .arg("--verbose")
        .arg("metadata")
        .current_dir(dir.path())
        .output()
        .expect("执行 x --verbose metadata 失败");

    if !out.status.success() {
        panic!(
            "x --verbose metadata failed.\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }

    let stdout = String::from_utf8_lossy(&out.stdout);
    serde_json::from_str::<serde_json::Value>(&stdout).expect("metadata stdout should be valid JSON");
}

#[test]
fn locate_project_keeps_json_on_stdout() {
    let bin = x_bin();
    let dir = tempfile::tempdir().expect("tempdir");

    std::fs::write(
        dir.path().join("x.toml"),
        "[package]\nname = \"locate-me\"\nversion = \"0.1.0\"\n",
    )
    .expect("write manifest");

    let out = Command::new(&bin)
        .arg("locate-project")
        .current_dir(dir.path())
        .output()
        .expect("执行 x locate-project 失败");

    if !out.status.success() {
        panic!(
            "x locate-project failed.\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    serde_json::from_str::<serde_json::Value>(&stdout)
        .expect("locate-project stdout should be valid JSON");
    assert!(stderr.trim().is_empty(), "stderr was: {stderr}");
}

#[test]
fn verbose_emits_observability_for_check_to_stderr() {
    let bin = x_bin();
    let dir = tempfile::tempdir().expect("tempdir");
    let file = write_program(&dir, "verbose_check.x", "function main() { println(\"hi\") }\n");

    let out = Command::new(&bin)
        .arg("--verbose")
        .arg("check")
        .arg(&file)
        .output()
        .expect("执行 x --verbose check 失败");

    if !out.status.success() {
        panic!(
            "x --verbose check failed.\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("INFO"), "stderr was: {stderr}");
    assert!(stderr.contains("Finished: 检查通过（语法 + 类型）"), "stderr was: {stderr}");
}

#[test]
fn smoke_check_finds_stdlib_from_binary_when_run_outside_repo_without_x_root() {
    let bin = x_bin();
    let dir = tempfile::tempdir().expect("tempdir");
    let file = write_program(&dir, "outside_check.x", "println(\"hi outside\")\n");

    let out = command_without_x_root(&bin)
        .arg("check")
        .arg(&file)
        .current_dir(dir.path())
        .output()
        .expect("执行仓库外 x check 失败");

    if !out.status.success() {
        panic!(
            "x check outside repo without X_ROOT failed.\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

#[test]
fn smoke_run_finds_stdlib_from_binary_when_run_outside_repo_without_x_root() {
    let bin = x_bin();
    let dir = tempfile::tempdir().expect("tempdir");
    let file = write_program(&dir, "outside_run.x", "println(\"hi outside run\")\n");

    let out = command_without_x_root(&bin)
        .arg("run")
        .arg(&file)
        .current_dir(dir.path())
        .output()
        .expect("执行仓库外 x run 失败");

    if !out.status.success() {
        panic!(
            "x run outside repo without X_ROOT failed.\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stdout.contains("hi outside run"), "stdout was: {stdout}");
    assert!(!stdout.contains("Finished"), "stdout was: {stdout}");
    assert!(stderr.contains("Finished"), "stderr was: {stderr}");
}
