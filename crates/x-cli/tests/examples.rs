//! 示例程序测试集：确保 examples/*.x 能通过 check 并成功 run。
//!
//! 运行: `cargo test -p x-cli --no-default-features --test examples`
//!
//! 建议使用单独目标目录，避免测试 spawn 的 x 与 target/debug 冲突导致「x.exe 被占用」:
//!   CARGO_TARGET_DIR=target_examples_test cargo test -p x-cli --no-default-features --test examples

use std::path::PathBuf;
use std::process::Command;

fn examples_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples")
}

fn x_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_x"))
}

/// 列出 examples 目录下所有 .x 文件
fn example_files() -> Vec<PathBuf> {
    let dir = examples_dir();
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .expect("examples 目录应存在")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map_or(false, |e| e == "x"))
        .collect();
    files.sort();
    files
}

#[test]
fn examples_dir_exists() {
    let dir = examples_dir();
    assert!(dir.is_dir(), "examples 目录应存在: {}", dir.display());
}

#[test]
fn at_least_one_example() {
    let files = example_files();
    assert!(!files.is_empty(), "examples 下应至少有一个 .x 文件");
}

/// 对每个 .x 执行 `x check <file>`，要求全部通过
#[test]
fn examples_check() {
    let bin = x_bin();
    let files = example_files();
    let mut failed = Vec::new();
    for path in &files {
        let status = Command::new(&bin)
            .arg("check")
            .arg(path)
            .output()
            .expect("执行 x check 失败");
        if !status.status.success() {
            let stderr = String::from_utf8_lossy(&status.stderr);
            let stdout = String::from_utf8_lossy(&status.stdout);
            failed.push((path.clone(), status.status, stdout.to_string(), stderr.to_string()));
        }
    }
    if !failed.is_empty() {
        let msg: String = failed
            .iter()
            .map(|(p, _s, out, err)| format!("{}:\nstdout:\n{}\nstderr:\n{}\n", p.display(), out, err))
            .collect();
        panic!("以下示例 check 未通过:\n{}", msg);
    }
}

/// 对每个 .x 执行 `x run <file>`，要求全部成功运行
#[test]
fn examples_run() {
    let bin = x_bin();
    let files = example_files();
    let mut failed = Vec::new();
    for path in &files {
        let status = Command::new(&bin)
            .arg("run")
            .arg(path)
            .output()
            .expect("执行 x run 失败");
        if !status.status.success() {
            let stderr = String::from_utf8_lossy(&status.stderr);
            let stdout = String::from_utf8_lossy(&status.stdout);
            failed.push((path.clone(), status.status, stdout.to_string(), stderr.to_string()));
        }
    }
    if !failed.is_empty() {
        let msg: String = failed
            .iter()
            .map(|(p, _s, out, err)| format!("{}:\nstdout:\n{}\nstderr:\n{}\n", p.display(), out, err))
            .collect();
        panic!("以下示例 run 未通过:\n{}", msg);
    }
}
