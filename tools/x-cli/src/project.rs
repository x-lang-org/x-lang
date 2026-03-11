use crate::manifest::Manifest;
use std::path::{Path, PathBuf};

pub struct Project {
    pub root: PathBuf,
    pub manifest: Manifest,
    pub manifest_path: PathBuf,
}

impl Project {
    pub fn find() -> Result<Self, String> {
        let cwd = std::env::current_dir().map_err(|e| format!("无法获取当前目录: {}", e))?;
        Self::find_from(&cwd)
    }

    pub fn find_from(start: &Path) -> Result<Self, String> {
        let manifest_path = Manifest::find_manifest_path(start).ok_or_else(|| {
            format!(
                "在 {} 及其父目录中找不到 x.toml\n\
                 提示: 使用 `x new <名称>` 创建新项目，或 `x init` 初始化当前目录",
                start.display()
            )
        })?;
        let root = manifest_path.parent().unwrap().to_path_buf();
        let manifest = Manifest::from_path(&manifest_path)?;
        Ok(Project {
            root,
            manifest,
            manifest_path,
        })
    }

    pub fn target_dir(&self) -> PathBuf {
        self.root.join("target")
    }

    pub fn source_dir(&self) -> PathBuf {
        self.root.join("src")
    }

    pub fn tests_dir(&self) -> PathBuf {
        self.root.join("tests")
    }

    pub fn benches_dir(&self) -> PathBuf {
        self.root.join("benches")
    }

    pub fn examples_dir(&self) -> PathBuf {
        self.root.join("examples")
    }

    pub fn name(&self) -> &str {
        self.manifest.package_name().unwrap_or("unknown")
    }

    pub fn version(&self) -> &str {
        self.manifest.package_version().unwrap_or("0.0.0")
    }

    pub fn source_files(&self) -> Vec<PathBuf> {
        collect_x_files(&self.source_dir())
    }

    pub fn main_file(&self) -> Option<PathBuf> {
        if !self.manifest.bins.is_empty() {
            if let Some(path) = &self.manifest.bins[0].path {
                let p = self.root.join(path);
                if p.exists() {
                    return Some(p);
                }
            }
        }
        let main = self.source_dir().join("main.x");
        if main.exists() {
            Some(main)
        } else {
            None
        }
    }

    pub fn lib_file(&self) -> Option<PathBuf> {
        if let Some(lib) = &self.manifest.lib {
            if let Some(path) = &lib.path {
                let p = self.root.join(path);
                if p.exists() {
                    return Some(p);
                }
            }
        }
        let lib = self.source_dir().join("lib.x");
        if lib.exists() {
            Some(lib)
        } else {
            None
        }
    }

    pub fn test_files(&self) -> Vec<PathBuf> {
        collect_x_files(&self.tests_dir())
    }

    pub fn bench_files(&self) -> Vec<PathBuf> {
        collect_x_files(&self.benches_dir())
    }

    pub fn example_files(&self) -> Vec<PathBuf> {
        collect_x_files(&self.examples_dir())
    }
}

fn collect_x_files(dir: &Path) -> Vec<PathBuf> {
    if !dir.exists() {
        return Vec::new();
    }
    walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "x"))
        .map(|e| e.path().to_path_buf())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_find_from_missing_manifest_has_hint() {
        let dir = tempfile::tempdir().expect("tempdir");
        let err = match Project::find_from(dir.path()) {
            Ok(_) => panic!("should error"),
            Err(e) => e,
        };
        assert!(err.contains("找不到 x.toml"), "{err}");
        assert!(err.contains("x new"), "{err}");
        assert!(err.contains("x init"), "{err}");
    }

    #[test]
    fn project_find_from_loads_manifest_and_root() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        std::fs::create_dir_all(root.join("src")).expect("mkdir src");
        std::fs::write(root.join("x.toml"), "package = { name = \"p\", version = \"0.2.0\" }\n")
            .expect("write manifest");
        std::fs::write(root.join("src").join("main.x"), "print(\"hi\")\n").expect("write main");

        let proj = Project::find_from(root).expect("should find project");
        assert_eq!(proj.name(), "p");
        assert_eq!(proj.version(), "0.2.0");
        assert_eq!(proj.root, root);
        assert!(proj.main_file().is_some());
    }
}
