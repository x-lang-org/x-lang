use colored::*;
use log::{debug, error as log_error, info, warn};
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};

static QUIET_MODE: AtomicBool = AtomicBool::new(false);

pub fn init_logging(verbose: bool, quiet: bool) {
    QUIET_MODE.store(quiet, Ordering::Relaxed);

    let level = if quiet {
        log::LevelFilter::Error
    } else if verbose {
        log::LevelFilter::Info
    } else {
        log::LevelFilter::Off
    };

    let mut builder = env_logger::Builder::new();
    builder
        .target(env_logger::Target::Stderr)
        .filter_level(level)
        .format(|buf, record| {
            writeln!(
                buf,
                "[{} {}] {}",
                buf.timestamp_millis(),
                record.level(),
                record.args()
            )
        });

    let _ = builder.try_init();

    debug!("cli logging initialized");
}

fn quiet_mode() -> bool {
    QUIET_MODE.load(Ordering::Relaxed)
}

pub fn status(label: &str, message: &str) {
    info!(target: "x_cli::status", "{}: {}", label, message);
    if !quiet_mode() {
        eprintln!("{:>12} {}", label.green().bold(), message);
    }
}

pub fn status_stderr(label: &str, message: &str) {
    status(label, message);
}

pub fn warning(message: &str) {
    warn!(target: "x_cli::warning", "{}", message);
    if !quiet_mode() {
        eprintln!("{}: {}", "warning".yellow().bold(), message);
    }
}

pub fn error(message: &str) {
    log_error!(target: "x_cli::error", "{}", message);
    eprintln!("{}: {}", "error".red().bold(), message);
}

pub fn note(message: &str) {
    info!(target: "x_cli::note", "{}", message);
    if !quiet_mode() {
        eprintln!("{}: {}", "note".cyan().bold(), message);
    }
}

pub fn validate_package_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("包名不能为空".to_string());
    }
    if name.len() > 64 {
        return Err("包名长度不能超过 64 个字符".to_string());
    }
    if !name.chars().next().unwrap().is_alphanumeric() {
        return Err("包名必须以字母或数字开头".to_string());
    }
    for c in name.chars() {
        if !c.is_alphanumeric() && c != '-' && c != '_' {
            return Err(format!("包名包含非法字符: '{}'", c));
        }
    }
    let reserved = [
        "test", "main", "lib", "src", "std", "core", "self", "super", "crate", "pub", "mod", "use",
        "as", "if", "else", "for", "while", "loop", "break", "continue", "return", "fn", "fun",
        "val", "var", "type", "struct", "enum", "impl", "trait",
    ];
    if reserved.contains(&name) {
        return Err(format!("'{}' 是保留名称，不能作为包名", name));
    }
    Ok(())
}

pub fn sanitize_package_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

pub fn elapsed_str(elapsed: std::time::Duration) -> String {
    let secs = elapsed.as_secs_f64();
    if secs < 1.0 {
        format!("{:.2}ms", secs * 1000.0)
    } else if secs < 60.0 {
        format!("{:.2}s", secs)
    } else {
        let mins = (secs / 60.0).floor() as u64;
        let remaining = secs - (mins as f64 * 60.0);
        format!("{}m {:.2}s", mins, remaining)
    }
}

pub fn confirm(prompt: &str) -> bool {
    print!("{} [y/N] ", prompt);
    let _ = std::io::Write::flush(&mut std::io::stdout());
    let mut input = String::new();
    if std::io::stdin().read_line(&mut input).is_ok() {
        let trimmed = input.trim().to_lowercase();
        trimmed == "y" || trimmed == "yes"
    } else {
        false
    }
}
