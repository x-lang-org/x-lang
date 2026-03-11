//! CLI 冒烟测试：确保 `x check` 与 `x run` 的关键路径可用。
//!
//! 注意：仓库根 `examples/*.x` 可能包含“规范/未来语法”用例，
//! 不一定与当前 parser/typechecker 的能力完全对齐，因此这里使用最小自包含的源码做回归。

use std::path::PathBuf;
use std::process::Command;

fn x_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_x"))
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
}
