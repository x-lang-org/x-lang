use spec_runner::{SpecTestError, SpecTestRunner};
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn resolve_x_cli_path() -> Result<PathBuf, SpecTestError> {
    let project_root = repo_root();
    let candidates = [
        project_root.join("tools/target/debug/x.exe"),
        project_root.join("tools/target/release/x.exe"),
        project_root.join("tools/target/debug/x"),
        project_root.join("tools/target/release/x"),
    ];

    candidates
        .into_iter()
        .find(|path| path.exists())
        .ok_or_else(|| SpecTestError::TestFailed {
            name: "spec-runner bootstrap".to_string(),
            message: "Failed to locate x-cli. Tried tools/target/{debug,release}/x(.exe)".to_string(),
        })
}

fn main() -> Result<(), SpecTestError> {
    let args: Vec<String> = std::env::args().collect();
    let project_root = repo_root();

    let spec_dir = if args.len() > 1 {
        project_root.join(&args[1])
    } else {
        project_root.join("tests/spec")
    };

    let x_cli_path = resolve_x_cli_path()?;

    println!("🧪 X Language Specification Test Runner");
    println!("📁 Test directory: {}", spec_dir.display());
    println!("🔧 Using x-cli: {}\n", x_cli_path.display());

    let runner = SpecTestRunner::new(x_cli_path)?;
    let summary = runner.run_directory(&spec_dir)?;

    println!("\n{}", "=".repeat(60));
    println!("Test Summary:");
    println!("  ✅ Passed:  {}", summary.passed);
    println!("  ❌ Failed:  {}", summary.failed);
    println!("  ⏭️  Skipped: {}", summary.skipped);
    println!("  📊 Total:   {}", summary.total());
    println!("  📈 Success: {:.1}%", summary.success_rate());
    println!("{}", "=".repeat(60));

    if summary.failed > 0 {
        std::process::exit(1);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repo_root_contains_tests_directory() {
        let root = repo_root();
        assert!(root.join("tests").exists(), "repo root should contain tests/: {}", root.display());
    }

    #[test]
    fn resolve_x_cli_path_prefers_existing_debug_or_release_binary() {
        let resolved = resolve_x_cli_path().expect("should find workspace x-cli binary");
        assert!(resolved.exists(), "resolved path should exist: {}", resolved.display());
        assert_eq!(resolved.file_stem().and_then(|stem| stem.to_str()), Some("x"));
    }
}
