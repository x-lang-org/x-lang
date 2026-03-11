use crate::lockfile::{LockedPackage, Lockfile};
use crate::manifest::{Dependency, Manifest};

#[derive(Debug, Clone)]
pub struct ResolvedDep {
    pub name: String,
    pub version: String,
    pub source: DepSource,
}

#[derive(Debug, Clone)]
pub enum DepSource {
    Registry(String),
    Path(String),
    Git { url: String, rev: Option<String> },
}

pub struct Resolver {
    pub manifest: Manifest,
    pub lockfile: Option<Lockfile>,
}

impl Resolver {
    pub fn new(manifest: Manifest, lockfile: Option<Lockfile>) -> Self {
        Resolver { manifest, lockfile }
    }

    pub fn resolve(&self) -> Result<Vec<ResolvedDep>, String> {
        let mut resolved = Vec::new();
        for (name, dep) in &self.manifest.dependencies {
            resolved.push(self.resolve_dep(name, dep)?);
        }
        Ok(resolved)
    }

    pub fn resolve_dev(&self) -> Result<Vec<ResolvedDep>, String> {
        let mut resolved = Vec::new();
        for (name, dep) in &self.manifest.dev_dependencies {
            resolved.push(self.resolve_dep(name, dep)?);
        }
        Ok(resolved)
    }

    fn resolve_dep(&self, name: &str, dep: &Dependency) -> Result<ResolvedDep, String> {
        match dep {
            Dependency::Simple(version) => {
                if let Some(ref lockfile) = self.lockfile {
                    if let Some(locked) = lockfile.find_package(name) {
                        return Ok(ResolvedDep {
                            name: name.to_string(),
                            version: locked.version.clone(),
                            source: DepSource::Registry(locked.source.clone().unwrap_or_default()),
                        });
                    }
                }
                Ok(ResolvedDep {
                    name: name.to_string(),
                    version: version.clone(),
                    source: DepSource::Registry("default".to_string()),
                })
            }
            Dependency::Detailed(d) => {
                if let Some(ref path) = d.path {
                    return Ok(ResolvedDep {
                        name: name.to_string(),
                        version: d.version.clone().unwrap_or_else(|| "0.0.0".to_string()),
                        source: DepSource::Path(path.clone()),
                    });
                }
                if let Some(ref git) = d.git {
                    return Ok(ResolvedDep {
                        name: name.to_string(),
                        version: d.version.clone().unwrap_or_else(|| "0.0.0".to_string()),
                        source: DepSource::Git {
                            url: git.clone(),
                            rev: d.rev.clone().or_else(|| d.tag.clone()),
                        },
                    });
                }
                let version = d.version.clone().unwrap_or_else(|| "*".to_string());
                Ok(ResolvedDep {
                    name: name.to_string(),
                    version,
                    source: DepSource::Registry("default".to_string()),
                })
            }
        }
    }
}

pub fn generate_lockfile(manifest: &Manifest) -> Lockfile {
    let resolver = Resolver::new(manifest.clone(), None);
    let mut lockfile = Lockfile::new();

    if let Ok(deps) = resolver.resolve() {
        for dep in deps {
            lockfile.add_or_update(LockedPackage {
                name: dep.name,
                version: dep.version,
                source: match dep.source {
                    DepSource::Registry(r) => Some(format!("registry+{}", r)),
                    DepSource::Path(p) => Some(format!("path+{}", p)),
                    DepSource::Git { url, rev } => Some(format!(
                        "git+{}{}",
                        url,
                        rev.map(|r| format!("#{}", r)).unwrap_or_default()
                    )),
                },
                checksum: None,
                dependencies: Vec::new(),
            });
        }
    }

    lockfile
}
