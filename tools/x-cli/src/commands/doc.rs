use crate::project::Project;
use crate::utils;
use std::time::Instant;

#[allow(unused_variables)]
pub fn exec(open: bool, no_deps: bool, document_private_items: bool) -> Result<(), String> {
    let project = Project::find()?;
    let start = Instant::now();

    utils::status(
        "Documenting",
        &format!("{} v{}", project.name(), project.version()),
    );

    let doc_dir = project.target_dir().join("doc");
    std::fs::create_dir_all(&doc_dir).map_err(|e| format!("无法创建文档目录: {}", e))?;

    let source_files = project.source_files();
    let mut doc_content = String::new();
    doc_content.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
    doc_content.push_str(&format!("<title>{} - X语言文档</title>\n", project.name()));
    doc_content.push_str("<meta charset=\"utf-8\">\n");
    doc_content.push_str(STYLE);
    doc_content.push_str("</head>\n<body>\n");
    doc_content.push_str(&format!(
        "<h1>{} <small>v{}</small></h1>\n",
        project.name(),
        project.version()
    ));

    if let Some(pkg) = &project.manifest.package {
        if let Some(desc) = &pkg.description {
            doc_content.push_str(&format!("<p class=\"desc\">{}</p>\n", html_escape(desc)));
        }
    }

    for path in &source_files {
        let rel = path.strip_prefix(&project.root).unwrap_or(path);
        let content = std::fs::read_to_string(path).unwrap_or_default();
        doc_content.push_str("<div class=\"file\">\n");
        doc_content.push_str(&format!("<h2>{}</h2>\n", rel.display()));
        doc_content.push_str(&format!(
            "<pre><code>{}</code></pre>\n",
            html_escape(&content)
        ));
        doc_content.push_str("</div>\n");
    }

    doc_content.push_str("</body>\n</html>");

    let index_path = doc_dir.join("index.html");
    std::fs::write(&index_path, &doc_content).map_err(|e| format!("无法写入文档: {}", e))?;

    let elapsed = start.elapsed();
    utils::status(
        "Finished",
        &format!("documentation in {}", utils::elapsed_str(elapsed)),
    );
    utils::status("Generated", &format!("{}", index_path.display()));

    if open {
        open_browser(&index_path.display().to_string());
    }

    Ok(())
}

const STYLE: &str = "<style>
body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; max-width: 900px; margin: 0 auto; padding: 20px; color: #1a1a2e; }
h1 { border-bottom: 2px solid #1a1a2e; padding-bottom: 12px; }
h1 small { color: #666; font-weight: normal; }
h2 { color: #16213e; border-bottom: 1px solid #eee; padding-bottom: 8px; }
pre { background: #f6f8fa; padding: 16px; border-radius: 6px; overflow-x: auto; border: 1px solid #e1e4e8; }
code { font-family: 'Fira Code', 'Cascadia Code', monospace; font-size: 14px; }
.file { margin: 24px 0; }
.desc { color: #555; font-size: 1.1em; }
</style>\n";

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn open_browser(url: &str) {
    #[cfg(target_os = "windows")]
    let _ = std::process::Command::new("cmd")
        .args(["/C", "start", url])
        .spawn();

    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open").arg(url).spawn();

    #[cfg(target_os = "linux")]
    let _ = std::process::Command::new("xdg-open").arg(url).spawn();
}
