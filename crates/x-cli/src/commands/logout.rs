use crate::config::Credentials;
use crate::utils;

pub fn exec(registry: Option<&str>) -> Result<(), String> {
    let mut creds = Credentials::load();

    if creds.get_token(registry).is_none() {
        utils::warning(&format!(
            "未找到 {} 的登录凭证",
            registry.unwrap_or("default registry")
        ));
        return Ok(());
    }

    creds.remove_token(registry);
    creds.save()?;

    utils::status(
        "Logout",
        &format!(
            "token removed for {}",
            registry.unwrap_or("default registry")
        ),
    );

    Ok(())
}
