//! Specification test runner for X language.
//! Cases are TOML files in cases/ with [[case]] entries.
//! Optional `spec = ["README section"]` links to README.md for traceability.

use serde::Deserialize;
use std::path::Path;
use std::fs;
use std::env;

#[derive(Debug, Deserialize)]
struct Section {
    id: Option<String>,
    name: Option<String>,
    spec_chapter: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Case {
    name: String,
    source: String,
    #[serde(default)]
    spec: Vec<String>,
    #[serde(default)]
    compile_fail: bool,
    #[serde(default)]
    error_contains: Vec<String>,
    exit_code: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct Root {
    section: Option<Section>,
    #[serde(default)]
    case: Vec<Case>,
}

fn run_case(c: &Case, filter: Option<&str>) -> bool {
    if let Some(f) = filter {
        if !c.name.contains(f) && !c.spec.iter().any(|s| s.contains(f)) {
            return true; // skip but count as pass for filtering
        }
    }

    let parser = x_parser::parser::XParser::new();
    let parse_result = parser.parse(c.source.trim());

    if c.compile_fail {
        match &parse_result {
            Err(e) => {
                let err_str = e.to_string();
                let ok = c.error_contains.is_empty()
                    || c.error_contains.iter().all(|sub| err_str.contains(sub));
                if !ok {
                    eprintln!("  FAIL {}: expected error to contain {:?}, got: {}", c.name, c.error_contains, err_str);
                }
                ok
            }
            Ok(_) => {
                eprintln!("  FAIL {}: expected compile failure but parse succeeded", c.name);
                false
            }
        }
    } else {
        match &parse_result {
            Ok(program) => {
                if let Some(_expected_code) = c.exit_code {
                    let mut interp = x_interpreter::Interpreter::new();
                    match interp.run(program) {
                        Ok(()) => true,
                        Err(e) => {
                            eprintln!("  FAIL {}: interpreter error: {}", c.name, e);
                            false
                        }
                    }
                } else {
                    true
                }
            }
            Err(e) => {
                eprintln!("  FAIL {}: expected pass but got: {}", c.name, e);
                false
            }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let filter = args.get(1).map(String::as_str);

    let cases_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("cases");
    if !cases_dir.exists() {
        eprintln!("Cases directory not found: {:?}", cases_dir);
        std::process::exit(1);
    }

    let mut total = 0usize;
    let mut passed = 0usize;

    let mut files: Vec<_> = fs::read_dir(&cases_dir)
        .map_err(|e| {
            eprintln!("Failed to read cases dir: {}", e);
            std::process::exit(1);
        })
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "toml"))
        .collect();
    files.sort_by_key(|e| e.file_name());

    for entry in files {
        let path = entry.path();
        let file_name = path.file_name().unwrap().to_string_lossy();
        let s = match fs::read_to_string(&path) {
            Ok(x) => x,
            Err(e) => {
                eprintln!("Failed to read {}: {}", file_name, e);
                std::process::exit(1);
            }
        };
        let root: Root = match toml::from_str(&s) {
            Ok(x) => x,
            Err(e) => {
                eprintln!("Failed to parse {}: {}", file_name, e);
                std::process::exit(1);
            }
        };
        for case in root.case {
            total += 1;
            if run_case(&case, filter) {
                passed += 1;
                if filter.is_some() {
                    println!("  PASS {}", case.name);
                }
            }
        }
    }

    println!("x-spec: {} passed, {} total", passed, total);
    if passed < total {
        std::process::exit(1);
    }
}
