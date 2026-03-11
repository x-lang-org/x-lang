use crate::config::Credentials;
use crate::utils;

pub fn exec(token: Option<&str>, registry: Option<&str>) -> Result<(), String> {
    let token = match token {
        Some(t) => t.to_string(),
        None => {
            println!("请输入 API token（可从注册表网站获取）:");
            print!("> ");
            let _ = std::io::Write::flush(&mut std::io::stdout());
            let mut input = String::new();
            std::io::stdin()
                .read_line(&mut input)
                .map_err(|e| format!("读取输入失败: {}", e))?;
            input.trim().to_string()
        }
    };

    if token.is_empty() {
        return Err("token 不能为空".to_string());
    }

    let mut creds = Credentials::load();
    creds.set_token(registry, token);
    creds.save()?;

    utils::status(
        "Login",
        &format!("token saved for {}", registry.unwrap_or("default registry")),
    );

    Ok(())
}
