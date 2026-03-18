use crate::registry::RegistryClient;
use crate::utils;

pub fn exec(query: &str, limit: usize, registry: Option<&str>) -> Result<(), String> {
    let client = RegistryClient::new(registry);

    utils::status("Searching", &format!("\"{}\"", query));

    let results = client.search(query, limit)?;

    if results.is_empty() {
        println!("未找到匹配的包");
        return Ok(());
    }

    for pkg in &results {
        let desc = pkg.description.as_deref().unwrap_or("");
        println!("{} = \"{}\"    # {}", pkg.name, pkg.max_version, desc);
    }

    Ok(())
}
