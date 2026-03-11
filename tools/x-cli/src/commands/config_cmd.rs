use crate::utils;

fn load_raw_config() -> toml_edit::DocumentMut {
    let path = crate::config::config_path();
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(doc) = content.parse::<toml_edit::DocumentMut>() {
                return doc;
            }
        }
    }
    toml_edit::DocumentMut::new()
}

fn save_raw_config(doc: &toml_edit::DocumentMut) -> Result<(), String> {
    let path = crate::config::config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("无法创建目录: {}", e))?;
    }
    std::fs::write(&path, doc.to_string()).map_err(|e| format!("无法写入配置: {}", e))
}

fn get_value_at_key<'a>(doc: &'a toml_edit::DocumentMut, key: &str) -> Option<&'a toml_edit::Item> {
    let parts: Vec<&str> = key.split('.').collect();
    let mut current: &toml_edit::Item = doc.as_item();
    for part in &parts {
        match current.as_table_like() {
            Some(tbl) => match tbl.get(part) {
                Some(item) => current = item,
                None => return None,
            },
            None => return None,
        }
    }
    Some(current)
}

fn format_item(prefix: &str, item: &toml_edit::Item) {
    match item {
        toml_edit::Item::Value(v) => {
            println!("{} = {}", prefix, v);
        }
        toml_edit::Item::Table(tbl) => {
            for (k, v) in tbl.iter() {
                let full_key = if prefix.is_empty() {
                    k.to_string()
                } else {
                    format!("{}.{}", prefix, k)
                };
                format_item(&full_key, v);
            }
        }
        _ => {}
    }
}

fn parse_value(s: &str) -> toml_edit::Value {
    if let Ok(n) = s.parse::<i64>() {
        return toml_edit::value(n).into_value().unwrap();
    }
    if let Ok(f) = s.parse::<f64>() {
        if s.contains('.') {
            return toml_edit::value(f).into_value().unwrap();
        }
    }
    match s {
        "true" => toml_edit::value(true).into_value().unwrap(),
        "false" => toml_edit::value(false).into_value().unwrap(),
        _ => toml_edit::value(s).into_value().unwrap(),
    }
}

pub fn exec(action: &str, key: Option<&str>, value: Option<&str>) -> Result<(), String> {
    match action {
        "get" => {
            let doc = load_raw_config();
            if let Some(k) = key {
                match get_value_at_key(&doc, k) {
                    Some(item) => format_item(k, item),
                    None => utils::note(&format!("未找到配置键: {}", k)),
                }
            } else {
                format_item("", doc.as_item());
            }
            Ok(())
        }
        "set" => {
            let key = key.ok_or("请指定要设置的键")?;
            let value = value.ok_or("请指定要设置的值")?;
            let mut doc = load_raw_config();
            let parsed = parse_value(value);

            let parts: Vec<&str> = key.split('.').collect();
            match parts.len() {
                1 => {
                    doc[parts[0]] = toml_edit::Item::Value(parsed);
                }
                2 => {
                    if doc.get(parts[0]).is_none() {
                        doc[parts[0]] = toml_edit::Item::Table(toml_edit::Table::new());
                    }
                    doc[parts[0]][parts[1]] = toml_edit::Item::Value(parsed);
                }
                3 => {
                    if doc.get(parts[0]).is_none() {
                        doc[parts[0]] = toml_edit::Item::Table(toml_edit::Table::new());
                    }
                    if doc[parts[0]].get(parts[1]).is_none() {
                        doc[parts[0]][parts[1]] = toml_edit::Item::Table(toml_edit::Table::new());
                    }
                    doc[parts[0]][parts[1]][parts[2]] = toml_edit::Item::Value(parsed);
                }
                _ => return Err("键的层级过深（最多支持 3 层）".to_string()),
            }

            save_raw_config(&doc)?;
            utils::status("Set", &format!("{} = {}", key, value));
            Ok(())
        }
        "list" => {
            let doc = load_raw_config();
            format_item("", doc.as_item());
            Ok(())
        }
        _ => Err(format!(
            "未知的配置操作: {}（支持: get, set, list）",
            action
        )),
    }
}
