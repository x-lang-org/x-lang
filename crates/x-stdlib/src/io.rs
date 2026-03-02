// IO 模块
//
// 提供文件IO、标准输入输出等功能。

use super::{StdValue, StdError, option};
use std::fs;
use std::path::Path;

/// 读取文件内容
pub fn read_file(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.is_empty() {
        return Err(StdError::InvalidArgument("read_file需要一个参数: path".to_string()));
    }
    let path = args[0].as_string()?;
    match fs::read_to_string(path) {
        Ok(content) => Ok(StdValue::Result(Box::new(super::result::Result::Ok(StdValue::String(content))))),
        Err(e) => Ok(StdValue::Result(Box::new(super::result::Result::Err(StdValue::String(e.to_string()))))),
    }
}

/// 写入文件内容
pub fn write_file(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.len() < 2 {
        return Err(StdError::InvalidArgument("write_file需要两个参数: path, content".to_string()));
    }
    let path = args[0].as_string()?;
    let content = args[1].as_string()?;
    match fs::write(path, content) {
        Ok(_) => Ok(StdValue::Result(Box::new(super::result::Result::Ok(StdValue::Unit)))),
        Err(e) => Ok(StdValue::Result(Box::new(super::result::Result::Err(StdValue::String(e.to_string()))))),
    }
}

/// 检查文件是否存在
pub fn file_exists(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.is_empty() {
        return Err(StdError::InvalidArgument("file_exists需要一个参数: path".to_string()));
    }
    let path = args[0].as_string()?;
    Ok(StdValue::Boolean(Path::new(path).exists()))
}

/// 创建目录
pub fn create_dir(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.is_empty() {
        return Err(StdError::InvalidArgument("create_dir需要一个参数: path".to_string()));
    }
    let path = args[0].as_string()?;
    match fs::create_dir_all(path) {
        Ok(_) => Ok(StdValue::Result(Box::new(super::result::Result::Ok(StdValue::Unit)))),
        Err(e) => Ok(StdValue::Result(Box::new(super::result::Result::Err(StdValue::String(e.to_string()))))),
    }
}

/// 列出目录内容
pub fn list_dir(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.is_empty() {
        return Err(StdError::InvalidArgument("list_dir需要一个参数: path".to_string()));
    }
    let path = args[0].as_string()?;
    match fs::read_dir(path) {
        Ok(entries) => {
            let mut files = Vec::new();
            for entry in entries {
                if let Ok(entry) = entry {
                    if let Some(name) = entry.file_name().to_str() {
                        files.push(StdValue::String(name.to_string()));
                    }
                }
            }
            Ok(StdValue::Result(Box::new(super::result::Result::Ok(StdValue::List(files)))))
        }
        Err(e) => Ok(StdValue::Result(Box::new(super::result::Result::Err(StdValue::String(e.to_string()))))),
    }
}

/// 删除文件
pub fn remove_file(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.is_empty() {
        return Err(StdError::InvalidArgument("remove_file需要一个参数: path".to_string()));
    }
    let path = args[0].as_string()?;
    match fs::remove_file(path) {
        Ok(_) => Ok(StdValue::Result(Box::new(super::result::Result::Ok(StdValue::Unit)))),
        Err(e) => Ok(StdValue::Result(Box::new(super::result::Result::Err(StdValue::String(e.to_string()))))),
    }
}

/// 重命名文件
pub fn rename_file(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.len() < 2 {
        return Err(StdError::InvalidArgument("rename_file需要两个参数: from, to".to_string()));
    }
    let from = args[0].as_string()?;
    let to = args[1].as_string()?;
    match fs::rename(from, to) {
        Ok(_) => Ok(StdValue::Result(Box::new(super::result::Result::Ok(StdValue::Unit)))),
        Err(e) => Ok(StdValue::Result(Box::new(super::result::Result::Err(StdValue::String(e.to_string()))))),
    }
}

/// 复制文件
pub fn copy_file(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.len() < 2 {
        return Err(StdError::InvalidArgument("copy_file需要两个参数: from, to".to_string()));
    }
    let from = args[0].as_string()?;
    let to = args[1].as_string()?;
    match fs::copy(from, to) {
        Ok(_) => Ok(StdValue::Result(Box::new(super::result::Result::Ok(StdValue::Unit)))),
        Err(e) => Ok(StdValue::Result(Box::new(super::result::Result::Err(StdValue::String(e.to_string()))))),
    }
}

/// 获取文件元数据
pub fn file_metadata(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.is_empty() {
        return Err(StdError::InvalidArgument("file_metadata需要一个参数: path".to_string()));
    }
    let path = args[0].as_string()?;
    match fs::metadata(path) {
        Ok(meta) => {
            let mut map = std::collections::HashMap::new();
            map.insert("is_file".to_string(), StdValue::Boolean(meta.is_file()));
            map.insert("is_dir".to_string(), StdValue::Boolean(meta.is_dir()));
            map.insert("len".to_string(), StdValue::Integer(meta.len() as i64));
            Ok(StdValue::Result(Box::new(super::result::Result::Ok(StdValue::Map(map)))))
        }
        Err(e) => Ok(StdValue::Result(Box::new(super::result::Result::Err(StdValue::String(e.to_string()))))),
    }
}
