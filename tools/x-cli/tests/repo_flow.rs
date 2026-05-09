use std::path::{Path, PathBuf};
use std::process::Command;

fn x_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_x"))
}

fn git_available() -> bool {
    Command::new("git")
        .arg("--version")
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false)
}

fn git_env(command: &mut Command) -> &mut Command {
    command
        .env("GIT_AUTHOR_NAME", "Sisyphus")
        .env("GIT_AUTHOR_EMAIL", "sisyphus@example.com")
        .env("GIT_COMMITTER_NAME", "Sisyphus")
        .env("GIT_COMMITTER_EMAIL", "sisyphus@example.com")
}

fn init_git_repo(repo_dir: &Path) {
    let init = git_env(Command::new("git").current_dir(repo_dir))
        .arg("init")
        .output()
        .expect("git init should run");
    assert!(
        init.status.success(),
        "git init failed.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&init.stdout),
        String::from_utf8_lossy(&init.stderr)
    );

    let add = git_env(Command::new("git").current_dir(repo_dir))
        .args(["add", "."])
        .output()
        .expect("git add should run");
    assert!(
        add.status.success(),
        "git add failed.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&add.stdout),
        String::from_utf8_lossy(&add.stderr)
    );

    let commit = git_env(Command::new("git").current_dir(repo_dir))
        .args(["commit", "-m", "init"])
        .output()
        .expect("git commit should run");
    assert!(
        commit.status.success(),
        "git commit failed.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&commit.stdout),
        String::from_utf8_lossy(&commit.stderr)
    );
}

fn wrapper_path(install_dir: &Path, package: &str) -> PathBuf {
    if cfg!(windows) {
        install_dir.join(format!("{}.cmd", package))
    } else {
        install_dir.join(package)
    }
}

fn write_project(repo_dir: &Path, name: &str, version: &str, body: &str) {
    std::fs::create_dir_all(repo_dir.join("src")).expect("mkdir src");
    std::fs::write(
        repo_dir.join("x.toml"),
        format!("[package]\nname = \"{name}\"\nversion = \"{version}\"\n"),
    )
    .expect("write manifest");
    std::fs::write(repo_dir.join("src/main.x"), body).expect("write main");
}

#[test]
fn local_git_repo_flow_and_cleanup() {
    if !git_available() {
        return;
    }

    let bin = x_bin();
    let temp = tempfile::tempdir().expect("tempdir");
    let x_home = temp.path().join("x-home");
    let install_dir = temp.path().join("bin");
    let repo_dir = temp.path().join("hello-git");
    write_project(
        &repo_dir,
        "hello-git",
        "0.1.0",
        "function main() { println(\"installed from git\") }\n",
    );
    init_git_repo(&repo_dir);

    let install = Command::new(&bin)
        .arg("install")
        .arg("--git")
        .arg(&repo_dir)
        .arg("--root")
        .arg(&install_dir)
        .env("X_HOME", &x_home)
        .output()
        .expect("x install --git should run");
    assert!(
        install.status.success(),
        "x install --git failed.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&install.stdout),
        String::from_utf8_lossy(&install.stderr)
    );

    let wrapper = wrapper_path(&install_dir, "hello-git");
    assert!(wrapper.exists(), "wrapper should exist at {}", wrapper.display());
    assert!(
        x_home.join("toolchain").join("stdlib").join("prelude.x").exists(),
        "managed stdlib should be provisioned into X_HOME"
    );

    let list = Command::new(&bin)
        .arg("install")
        .arg("--list")
        .arg("--root")
        .arg(&install_dir)
        .env("X_HOME", &x_home)
        .output()
        .expect("x install --list should run");
    assert!(list.status.success(), "x install --list should succeed");
    let list_stdout = String::from_utf8_lossy(&list.stdout);
    assert!(list_stdout.contains("hello-git"), "stdout was: {list_stdout}");

    let run_wrapper = Command::new(&wrapper)
        .current_dir(temp.path())
        .env("X_HOME", &x_home)
        .env_remove("X_ROOT")
        .output()
        .expect("installed wrapper should run");
    assert!(
        run_wrapper.status.success(),
        "installed wrapper failed.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run_wrapper.stdout),
        String::from_utf8_lossy(&run_wrapper.stderr)
    );
    let wrapper_stdout = String::from_utf8_lossy(&run_wrapper.stdout);
    assert!(
        wrapper_stdout.contains("installed from git"),
        "stdout was: {wrapper_stdout}"
    );

    let managed_checkout = x_home.join("git").join("hello-git");
    assert!(managed_checkout.exists(), "managed checkout should exist");

    let uninstall = Command::new(&bin)
        .arg("uninstall")
        .arg("hello-git")
        .arg("--root")
        .arg(&install_dir)
        .env("X_HOME", &x_home)
        .output()
        .expect("x uninstall should run");
    assert!(
        uninstall.status.success(),
        "x uninstall failed.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&uninstall.stdout),
        String::from_utf8_lossy(&uninstall.stderr)
    );

    assert!(!wrapper.exists(), "wrapper should be removed");
    assert!(!managed_checkout.exists(), "managed checkout should be removed");
}

#[test]
fn installed_wrapper_prefers_managed_stdlib_over_cwd_shadow_copy() {
    if !git_available() {
        return;
    }

    let bin = x_bin();
    let temp = tempfile::tempdir().expect("tempdir");
    let x_home = temp.path().join("x-home");
    let install_dir = temp.path().join("bin");
    let repo_dir = temp.path().join("shadow-git");
    let hostile_cwd = temp.path().join("hostile-cwd");
    let hostile_stdlib = hostile_cwd.join("library").join("stdlib");

    write_project(
        &repo_dir,
        "shadow-git",
        "0.1.0",
        "function main() { println(\"managed stdlib won\") }\n",
    );
    init_git_repo(&repo_dir);

    let install = Command::new(&bin)
        .arg("install")
        .arg("--git")
        .arg(&repo_dir)
        .arg("--root")
        .arg(&install_dir)
        .env("X_HOME", &x_home)
        .output()
        .expect("x install --git should run");
    assert!(
        install.status.success(),
        "x install --git failed.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&install.stdout),
        String::from_utf8_lossy(&install.stderr)
    );

    std::fs::create_dir_all(&hostile_stdlib).expect("mkdir hostile stdlib");
    std::fs::write(
        hostile_stdlib.join("prelude.x"),
        "export function definitely_not_real() -> unit {}\n",
    )
    .expect("write hostile prelude");

    let wrapper = wrapper_path(&install_dir, "shadow-git");
    let run_wrapper = Command::new(&wrapper)
        .current_dir(&hostile_cwd)
        .env("X_HOME", &x_home)
        .env_remove("X_ROOT")
        .output()
        .expect("installed wrapper should run");

    assert!(
        run_wrapper.status.success(),
        "installed wrapper should ignore hostile cwd stdlib.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run_wrapper.stdout),
        String::from_utf8_lossy(&run_wrapper.stderr)
    );
    let stdout = String::from_utf8_lossy(&run_wrapper.stdout);
    assert!(stdout.contains("managed stdlib won"), "stdout was: {stdout}");
}

#[test]
fn install_time_managed_stdlib_seed_ignores_hostile_cwd_shadow() {
    if !git_available() {
        return;
    }

    let bin = x_bin();
    let temp = tempfile::tempdir().expect("tempdir");
    let x_home = temp.path().join("x-home");
    let install_dir = temp.path().join("bin");
    let repo_dir = temp.path().join("seed-git");
    let hostile_cwd = temp.path().join("hostile-install-cwd");
    let hostile_stdlib = hostile_cwd.join("library").join("stdlib");

    write_project(
        &repo_dir,
        "seed-git",
        "0.1.0",
        "function main() { println(\"trusted seed won\") }\n",
    );
    init_git_repo(&repo_dir);

    std::fs::create_dir_all(&hostile_stdlib).expect("mkdir hostile stdlib");
    std::fs::write(
        hostile_stdlib.join("prelude.x"),
        "export function definitely_not_real() -> unit {}\n",
    )
    .expect("write hostile prelude");

    let install = Command::new(&bin)
        .arg("install")
        .arg("--git")
        .arg(&repo_dir)
        .arg("--root")
        .arg(&install_dir)
        .current_dir(&hostile_cwd)
        .env("X_HOME", &x_home)
        .env_remove("X_ROOT")
        .output()
        .expect("x install --git should run");
    assert!(
        install.status.success(),
        "install should ignore hostile cwd stdlib during managed seed.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&install.stdout),
        String::from_utf8_lossy(&install.stderr)
    );

    let managed_prelude = std::fs::read_to_string(
        x_home.join("toolchain").join("stdlib").join("prelude.x"),
    )
    .expect("managed prelude should exist");
    assert!(
        !managed_prelude.contains("definitely_not_real"),
        "managed stdlib should not be copied from hostile cwd"
    );

    let wrapper = wrapper_path(&install_dir, "seed-git");
    let run_wrapper = Command::new(&wrapper)
        .current_dir(&hostile_cwd)
        .env("X_HOME", &x_home)
        .env_remove("X_ROOT")
        .output()
        .expect("installed wrapper should run");
    assert!(
        run_wrapper.status.success(),
        "wrapper should still run after trusted stdlib seeding.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run_wrapper.stdout),
        String::from_utf8_lossy(&run_wrapper.stderr)
    );
    let stdout = String::from_utf8_lossy(&run_wrapper.stdout);
    assert!(stdout.contains("trusted seed won"), "stdout was: {stdout}");
}

#[test]
fn git_mode_rejects_version_flag() {
    if !git_available() {
        return;
    }

    let bin = x_bin();
    let temp = tempfile::tempdir().expect("tempdir");
    let repo_dir = temp.path().join("hello-git");
    write_project(
        &repo_dir,
        "hello-git",
        "0.1.0",
        "function main() { println(\"installed from git\") }\n",
    );
    init_git_repo(&repo_dir);

    let out = Command::new(&bin)
        .arg("install")
        .arg("--git")
        .arg(&repo_dir)
        .arg("--version")
        .arg("main")
        .env("X_HOME", temp.path().join("x-home"))
        .output()
        .expect("x install --git --version should run");

    assert!(!out.status.success(), "command should fail");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("--version 暂不支持与 --git 一起使用"), "stderr was: {stderr}");
}
