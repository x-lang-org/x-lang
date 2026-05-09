use std::path::PathBuf;
use std::process::Command;

fn x_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_x"))
}

fn write_registry_config(x_home: &std::path::Path, registry_name: &str, index_path: &std::path::Path) {
    std::fs::create_dir_all(x_home).expect("create X_HOME");
    let config = format!(
        "[registry]\ndefault = \"{registry_name}\"\n\n[registry.registries.{registry_name}]\nindex = '{}'\n",
        index_path.display()
    );
    std::fs::write(x_home.join("config.toml"), config).expect("write config");
}

fn write_package_json(index_dir: &std::path::Path, name: &str, version: &str, description: &str) {
    std::fs::create_dir_all(index_dir).expect("create index dir");
    let json = format!(
        "{{\n  \"name\": \"{name}\",\n  \"description\": \"{description}\",\n  \"max_version\": \"{version}\",\n  \"versions\": [{{\n    \"version\": \"{version}\",\n    \"yanked\": false,\n    \"checksum\": \"dummy\",\n    \"dependencies\": []\n  }}]\n}}\n"
    );
    std::fs::write(index_dir.join(format!("{name}.json")), json).expect("write package json");
}

fn write_project(dir: &std::path::Path, name: &str, description: &str) {
    std::fs::create_dir_all(dir.join("src")).expect("create src");
    std::fs::write(
        dir.join("x.toml"),
        format!(
            "[package]\nname = \"{name}\"\nversion = \"0.1.0\"\ndescription = \"{description}\"\n"
        ),
    )
    .expect("write manifest");
    std::fs::write(
        dir.join("src/main.x"),
        format!("function main() -> Unit {{ println(\"{name} works\") }}\n"),
    )
    .expect("write main");
}

fn write_versioned_project(dir: &std::path::Path, name: &str, version: &str, description: &str) {
    std::fs::create_dir_all(dir.join("src")).expect("create src");
    std::fs::write(
        dir.join("x.toml"),
        format!(
            "[package]\nname = \"{name}\"\nversion = \"{version}\"\ndescription = \"{description}\"\n"
        ),
    )
    .expect("write manifest");
    std::fs::write(
        dir.join("src/main.x"),
        format!("function main() -> Unit {{ println(\"{name} {version} works\") }}\n"),
    )
    .expect("write main");
}

#[test]
fn local_registry_search_returns_matching_packages() {
    let bin = x_bin();
    let dir = tempfile::tempdir().expect("tempdir");
    let x_home = dir.path().join("x-home");
    let registry_root = dir.path().join("local-registry");
    let index_dir = registry_root.join("index");

    write_registry_config(&x_home, "local", &registry_root);
    write_package_json(&index_dir, "hello-http", "1.2.3", "HTTP client for X");
    write_package_json(&index_dir, "world-db", "0.4.0", "Database helpers");

    let out = Command::new(&bin)
        .arg("search")
        .arg("http")
        .arg("--registry")
        .arg("local")
        .arg("--limit")
        .arg("5")
        .env("X_HOME", &x_home)
        .output()
        .expect("x search should run");

    assert!(
        out.status.success(),
        "x search failed.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stdout.contains("hello-http = \"1.2.3\"    # HTTP client for X"), "stdout was: {stdout}");
    assert!(!stdout.contains("world-db"), "stdout was: {stdout}");
    assert!(stderr.contains("Searching"), "stderr was: {stderr}");
}

#[test]
fn local_registry_search_reports_no_matches_cleanly() {
    let bin = x_bin();
    let dir = tempfile::tempdir().expect("tempdir");
    let x_home = dir.path().join("x-home");
    let registry_root = dir.path().join("local-registry");
    let index_dir = registry_root.join("index");

    write_registry_config(&x_home, "local", &registry_root);
    write_package_json(&index_dir, "hello-http", "1.2.3", "HTTP client for X");

    let out = Command::new(&bin)
        .arg("search")
        .arg("queue")
        .arg("--registry")
        .arg("local")
        .env("X_HOME", &x_home)
        .output()
        .expect("x search should run");

    assert!(out.status.success(), "x search should succeed without matches");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("未找到匹配的包"), "stdout was: {stdout}");
}

#[test]
fn remote_registry_search_remains_explicitly_unimplemented() {
    let bin = x_bin();
    let dir = tempfile::tempdir().expect("tempdir");
    let x_home = dir.path().join("x-home");

    std::fs::create_dir_all(&x_home).expect("create X_HOME");
    let config = "[registry]\ndefault = \"remote\"\n\n[registry.registries.remote]\nindex = 'https://registry.x-lang.org'\n";
    std::fs::write(x_home.join("config.toml"), config).expect("write config");

    let out = Command::new(&bin)
        .arg("search")
        .arg("http")
        .arg("--registry")
        .arg("remote")
        .env("X_HOME", &x_home)
        .output()
        .expect("x search should run");

    assert!(!out.status.success(), "remote registry search should stay unimplemented");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("当前仅支持本地文件注册表索引目录搜索"), "stderr was: {stderr}");
}

#[test]
fn explicit_registry_name_requires_active_config_entry() {
    let bin = x_bin();
    let dir = tempfile::tempdir().expect("tempdir");
    let missing_x_home = dir.path().join("missing-x-home");

    let out = Command::new(&bin)
        .arg("search")
        .arg("http")
        .arg("--registry")
        .arg("local")
        .env("X_HOME", &missing_x_home)
        .output()
        .expect("x search should run");

    assert!(!out.status.success(), "search should fail when registry config is missing");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("配置中未定义注册表 `local`"), "stderr was: {stderr}");
    assert!(stderr.contains("请设置 X_HOME"), "stderr was: {stderr}");
}

#[test]
fn missing_default_registry_config_fails_install_cleanly() {
    let bin = x_bin();
    let dir = tempfile::tempdir().expect("tempdir");
    let missing_x_home = dir.path().join("missing-x-home");
    let install_dir = dir.path().join("bin");

    let out = Command::new(&bin)
        .arg("install")
        .arg("offline-tool")
        .arg("--root")
        .arg(&install_dir)
        .env("X_HOME", &missing_x_home)
        .output()
        .expect("x install should run");

    assert!(!out.status.success(), "install should fail when default registry config is missing");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("当前未配置默认注册表"), "stderr was: {stderr}");
    assert!(stderr.contains("请设置 X_HOME"), "stderr was: {stderr}");
}

#[test]
fn broken_default_registry_entry_fails_cleanly() {
    let bin = x_bin();
    let dir = tempfile::tempdir().expect("tempdir");
    let x_home = dir.path().join("x-home");

    std::fs::create_dir_all(&x_home).expect("create X_HOME");
    let config = "[registry]\ndefault = \"local\"\n";
    std::fs::write(x_home.join("config.toml"), config).expect("write config");

    let out = Command::new(&bin)
        .arg("search")
        .arg("http")
        .env("X_HOME", &x_home)
        .output()
        .expect("x search should run");

    assert!(!out.status.success(), "search should fail when default registry entry is missing");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("配置中未定义注册表 `local`"), "stderr was: {stderr}");
}

#[test]
fn malformed_config_toml_fails_search_with_config_path_and_parse_reason() {
    let bin = x_bin();
    let dir = tempfile::tempdir().expect("tempdir");
    let x_home = dir.path().join("x-home");

    std::fs::create_dir_all(&x_home).expect("create X_HOME");
    std::fs::write(x_home.join("config.toml"), "[registry\ndefault = \"local\"\n")
        .expect("write malformed config");

    let out = Command::new(&bin)
        .arg("search")
        .arg("http")
        .env("X_HOME", &x_home)
        .output()
        .expect("x search should run");

    assert!(!out.status.success(), "search should fail on malformed config");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("无法解析全局配置"), "stderr was: {stderr}");
    assert!(stderr.contains("config.toml"), "stderr was: {stderr}");
    assert!(stderr.contains("invalid table header"), "stderr was: {stderr}");
}

#[test]
fn invalid_registry_config_shape_fails_install_with_config_path_and_reason() {
    let bin = x_bin();
    let dir = tempfile::tempdir().expect("tempdir");
    let x_home = dir.path().join("x-home");
    let install_dir = dir.path().join("bin");

    std::fs::create_dir_all(&x_home).expect("create X_HOME");
    std::fs::write(
        x_home.join("config.toml"),
        "[registry]\ndefault = 42\n\n[registry.registries.local]\nindex = 'C:/tmp/local-registry'\n",
    )
    .expect("write invalid config");

    let out = Command::new(&bin)
        .arg("install")
        .arg("offline-tool")
        .arg("--root")
        .arg(&install_dir)
        .env("X_HOME", &x_home)
        .output()
        .expect("x install should run");

    assert!(!out.status.success(), "install should fail on invalid config shape");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("无法解析全局配置"), "stderr was: {stderr}");
    assert!(stderr.contains("config.toml"), "stderr was: {stderr}");
    assert!(stderr.contains("invalid type"), "stderr was: {stderr}");
}

#[test]
fn malformed_config_toml_fails_publish_before_registry_resolution() {
    let bin = x_bin();
    let dir = tempfile::tempdir().expect("tempdir");
    let x_home = dir.path().join("x-home");
    let project_dir = dir.path().join("publisher");

    std::fs::create_dir_all(&x_home).expect("create X_HOME");
    std::fs::write(x_home.join("config.toml"), "[registry\ndefault = \"local\"\n")
        .expect("write malformed config");
    write_project(&project_dir, "offline-tool", "Offline local registry tool");

    let out = Command::new(&bin)
        .arg("publish")
        .arg("--allow-dirty")
        .current_dir(&project_dir)
        .env("X_HOME", &x_home)
        .output()
        .expect("x publish should run");

    assert!(!out.status.success(), "publish should fail on malformed config");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("无法解析全局配置"), "stderr was: {stderr}");
    assert!(stderr.contains("config.toml"), "stderr was: {stderr}");
    assert!(stderr.contains("invalid table header"), "stderr was: {stderr}");
}

#[test]
fn local_registry_publish_and_install_flow_works_end_to_end() {
    let bin = x_bin();
    let dir = tempfile::tempdir().expect("tempdir");
    let x_home = dir.path().join("x-home");
    let registry_root = dir.path().join("local-registry");
    let install_dir = dir.path().join("bin");
    let project_dir = dir.path().join("publisher");

    write_registry_config(&x_home, "local", &registry_root);
    write_project(&project_dir, "offline-tool", "Offline local registry tool");

    let publish = Command::new(&bin)
        .arg("publish")
        .arg("--registry")
        .arg("local")
        .arg("--allow-dirty")
        .current_dir(&project_dir)
        .env("X_HOME", &x_home)
        .output()
        .expect("x publish should run");
    assert!(
        publish.status.success(),
        "x publish failed.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&publish.stdout),
        String::from_utf8_lossy(&publish.stderr)
    );

    let search = Command::new(&bin)
        .arg("search")
        .arg("offline")
        .arg("--registry")
        .arg("local")
        .env("X_HOME", &x_home)
        .output()
        .expect("x search should run");
    assert!(search.status.success(), "x search should succeed");
    let search_stdout = String::from_utf8_lossy(&search.stdout);
    assert!(search_stdout.contains("offline-tool = \"0.1.0\""), "stdout was: {search_stdout}");

    let install = Command::new(&bin)
        .arg("install")
        .arg("offline-tool")
        .arg("--root")
        .arg(&install_dir)
        .env("X_HOME", &x_home)
        .output()
        .expect("x install should run");
    assert!(
        install.status.success(),
        "x install failed.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&install.stdout),
        String::from_utf8_lossy(&install.stderr)
    );

    let wrapper = if cfg!(windows) {
        install_dir.join("offline-tool.cmd")
    } else {
        install_dir.join("offline-tool")
    };
    assert!(wrapper.exists(), "wrapper should exist at {}", wrapper.display());

    let run = Command::new(&wrapper)
        .current_dir(dir.path())
        .env("X_HOME", &x_home)
        .env_remove("X_ROOT")
        .output()
        .expect("installed wrapper should run");
    assert!(run.status.success(), "wrapper should run successfully");
    let run_stdout = String::from_utf8_lossy(&run.stdout);
    assert!(run_stdout.contains("offline-tool works"), "stdout was: {run_stdout}");
    assert!(
        x_home.join("toolchain").join("stdlib").join("prelude.x").exists(),
        "managed stdlib should be provisioned into X_HOME"
    );
}

#[test]
fn local_registry_keeps_highest_version_when_published_out_of_order() {
    let bin = x_bin();
    let dir = tempfile::tempdir().expect("tempdir");
    let x_home = dir.path().join("x-home");
    let registry_root = dir.path().join("local-registry");
    let install_dir = dir.path().join("bin");
    let project_dir = dir.path().join("publisher");

    write_registry_config(&x_home, "local", &registry_root);
    write_versioned_project(&project_dir, "offline-tool", "1.0.0", "Offline local registry tool");

    let publish_v1 = Command::new(&bin)
        .arg("publish")
        .arg("--registry")
        .arg("local")
        .arg("--allow-dirty")
        .current_dir(&project_dir)
        .env("X_HOME", &x_home)
        .output()
        .expect("x publish should run");
    assert!(publish_v1.status.success(), "first publish should succeed");

    write_versioned_project(&project_dir, "offline-tool", "0.9.0", "Offline local registry tool");
    let publish_v09 = Command::new(&bin)
        .arg("publish")
        .arg("--registry")
        .arg("local")
        .arg("--allow-dirty")
        .current_dir(&project_dir)
        .env("X_HOME", &x_home)
        .output()
        .expect("x publish should run");
    assert!(publish_v09.status.success(), "second publish should succeed");

    let index = std::fs::read_to_string(registry_root.join("index").join("offline-tool.json"))
        .expect("read index json");
    assert!(index.contains("\"max_version\": \"1.0.0\""), "index was: {index}");

    let install = Command::new(&bin)
        .arg("install")
        .arg("offline-tool")
        .arg("--root")
        .arg(&install_dir)
        .env("X_HOME", &x_home)
        .output()
        .expect("x install should run");
    assert!(
        install.status.success(),
        "x install failed.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&install.stdout),
        String::from_utf8_lossy(&install.stderr)
    );

    let wrapper = if cfg!(windows) {
        install_dir.join("offline-tool.cmd")
    } else {
        install_dir.join("offline-tool")
    };
    let run = Command::new(&wrapper)
        .current_dir(dir.path())
        .env("X_HOME", &x_home)
        .env_remove("X_ROOT")
        .output()
        .expect("installed wrapper should run");
    assert!(run.status.success(), "wrapper should run successfully");
    let run_stdout = String::from_utf8_lossy(&run.stdout);
    assert!(run_stdout.contains("offline-tool 1.0.0 works"), "stdout was: {run_stdout}");
}

#[test]
fn local_registry_yank_and_unyank_gate_installability() {
    let bin = x_bin();
    let dir = tempfile::tempdir().expect("tempdir");
    let x_home = dir.path().join("x-home");
    let registry_root = dir.path().join("local-registry");
    let install_dir = dir.path().join("bin");
    let project_dir = dir.path().join("publisher");

    write_registry_config(&x_home, "local", &registry_root);
    write_project(&project_dir, "offline-tool", "Offline local registry tool");
    std::fs::write(
        x_home.join("credentials.toml"),
        "[registries.local]\ntoken = \"dev-token\"\n",
    )
    .expect("write credentials");

    let publish = Command::new(&bin)
        .arg("publish")
        .arg("--registry")
        .arg("local")
        .arg("--allow-dirty")
        .current_dir(&project_dir)
        .env("X_HOME", &x_home)
        .output()
        .expect("x publish should run");
    assert!(publish.status.success(), "publish should succeed");

    let yank = Command::new(&bin)
        .arg("yank")
        .arg("offline-tool")
        .arg("--version")
        .arg("0.1.0")
        .arg("--registry")
        .arg("local")
        .env("X_HOME", &x_home)
        .output()
        .expect("x yank should run");
    assert!(
        yank.status.success(),
        "x yank failed.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&yank.stdout),
        String::from_utf8_lossy(&yank.stderr)
    );

    let index_after_yank = std::fs::read_to_string(registry_root.join("index").join("offline-tool.json"))
        .expect("read index after yank");
    assert!(index_after_yank.contains("\"yanked\": true"), "index was: {index_after_yank}");

    let install_yanked = Command::new(&bin)
        .arg("install")
        .arg("offline-tool")
        .arg("--root")
        .arg(&install_dir)
        .env("X_HOME", &x_home)
        .output()
        .expect("x install should run");
    assert!(!install_yanked.status.success(), "install should fail for yanked version");
    let install_yanked_stderr = String::from_utf8_lossy(&install_yanked.stderr);
    assert!(install_yanked_stderr.contains("offline-tool@0.1.0 已被撤回，无法安装"), "stderr was: {install_yanked_stderr}");

    let unyank = Command::new(&bin)
        .arg("yank")
        .arg("offline-tool")
        .arg("--version")
        .arg("0.1.0")
        .arg("--undo")
        .arg("--registry")
        .arg("local")
        .env("X_HOME", &x_home)
        .output()
        .expect("x yank --undo should run");
    assert!(
        unyank.status.success(),
        "x yank --undo failed.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&unyank.stdout),
        String::from_utf8_lossy(&unyank.stderr)
    );

    let index_after_unyank = std::fs::read_to_string(registry_root.join("index").join("offline-tool.json"))
        .expect("read index after unyank");
    assert!(index_after_unyank.contains("\"yanked\": false"), "index was: {index_after_unyank}");

    let install = Command::new(&bin)
        .arg("install")
        .arg("offline-tool")
        .arg("--root")
        .arg(&install_dir)
        .env("X_HOME", &x_home)
        .output()
        .expect("x install should run after unyank");
    assert!(
        install.status.success(),
        "x install failed after unyank.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&install.stdout),
        String::from_utf8_lossy(&install.stderr)
    );
}
