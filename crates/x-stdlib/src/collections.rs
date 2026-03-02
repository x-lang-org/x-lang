// 集合模块
//
// 提供 List、Map、Set 等集合数据结构。

use super::{StdValue, StdError, option, result};
use std::collections::HashMap;

// ============================================
// List 列表函数
// ============================================

/// 创建新列表
pub fn list_new(_args: &[StdValue]) -> Result<StdValue, StdError> {
    Ok(StdValue::List(Vec::new()))
}

/// 列表长度
pub fn list_len(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.is_empty() {
        return Err(StdError::InvalidArgument("list_len需要一个参数".to_string()));
    }
    let list = args[0].as_list()?;
    Ok(StdValue::Integer(list.len() as i64))
}

/// 检查列表是否为空
pub fn list_is_empty(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.is_empty() {
        return Err(StdError::InvalidArgument("list_is_empty需要一个参数".to_string()));
    }
    let list = args[0].as_list()?;
    Ok(StdValue::Boolean(list.is_empty()))
}

/// 获取列表元素
pub fn list_get(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.len() < 2 {
        return Err(StdError::InvalidArgument("list_get需要两个参数: list, index".to_string()));
    }
    let list = args[0].as_list()?;
    let index = args[1].as_integer()? as usize;
    if index >= list.len() {
        return Ok(StdValue::Option(Box::new(option::Option::None)));
    }
    Ok(StdValue::Option(Box::new(option::Option::Some(list[index].clone()))))
}

/// 获取第一个元素
pub fn list_first(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.is_empty() {
        return Err(StdError::InvalidArgument("list_first需要一个参数".to_string()));
    }
    let list = args[0].as_list()?;
    if list.is_empty() {
        return Ok(StdValue::Option(Box::new(option::Option::None)));
    }
    Ok(StdValue::Option(Box::new(option::Option::Some(list[0].clone()))))
}

/// 获取最后一个元素
pub fn list_last(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.is_empty() {
        return Err(StdError::InvalidArgument("list_last需要一个参数".to_string()));
    }
    let list = args[0].as_list()?;
    if list.is_empty() {
        return Ok(StdValue::Option(Box::new(option::Option::None)));
    }
    Ok(StdValue::Option(Box::new(option::Option::Some(list[list.len() - 1].clone()))))
}

/// 追加元素到列表末尾
pub fn list_push(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.len() < 2 {
        return Err(StdError::InvalidArgument("list_push需要两个参数: list, value".to_string()));
    }
    let mut list = args[0].clone();
    let list_vec = list.as_list_mut()?;
    list_vec.push(args[1].clone());
    Ok(list)
}

/// 移除并返回列表末尾元素
pub fn list_pop(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.is_empty() {
        return Err(StdError::InvalidArgument("list_pop需要一个参数".to_string()));
    }
    let mut list = args[0].clone();
    let list_vec = list.as_list_mut()?;
    match list_vec.pop() {
        Some(v) => Ok(StdValue::Option(Box::new(option::Option::Some(v))))),
        None => Ok(StdValue::Option(Box::new(option::Option::None))),
    }
}

/// 在指定位置插入元素
pub fn list_insert(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.len() < 3 {
        return Err(StdError::InvalidArgument("list_insert需要三个参数: list, index, value".to_string()));
    }
    let mut list = args[0].clone();
    let list_vec = list.as_list_mut()?;
    let index = args[1].as_integer()? as usize;
    if index > list_vec.len() {
        return Err(StdError::OutOfBounds(format!("索引 {} 超出列表长度 {}", index, list_vec.len())));
    }
    list_vec.insert(index, args[2].clone());
    Ok(list)
}

/// 移除指定位置的元素
pub fn list_remove(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.len() < 2 {
        return Err(StdError::InvalidArgument("list_remove需要两个参数: list, index".to_string()));
    }
    let mut list = args[0].clone();
    let list_vec = list.as_list_mut()?;
    let index = args[1].as_integer()? as usize;
    if index >= list_vec.len() {
        return Err(StdError::OutOfBounds(format!("索引 {} 超出列表长度 {}", index, list_vec.len())));
    }
    let removed = list_vec.remove(index);
    Ok(StdValue::Option(Box::new(option::Option::Some(removed))))
}

/// 追加两个列表
pub fn list_append(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.len() < 2 {
        return Err(StdError::InvalidArgument("list_append需要两个参数: list1, list2".to_string()));
    }
    let list1 = args[0].as_list()?;
    let list2 = args[1].as_list()?;
    let mut result = list1.to_vec();
    result.extend_from_slice(list2);
    Ok(StdValue::List(result))
}

/// 反转列表
pub fn list_reverse(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.is_empty() {
        return Err(StdError::InvalidArgument("list_reverse需要一个参数".to_string()));
    }
    let list = args[0].as_list()?;
    let mut result = list.to_vec();
    result.reverse();
    Ok(StdValue::List(result))
}

/// 检查列表是否包含元素
pub fn list_contains(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.len() < 2 {
        return Err(StdError::InvalidArgument("list_contains需要两个参数: list, value".to_string()));
    }
    let list = args[0].as_list()?;
    let value = &args[1];
    let contains = list.iter().any(|v| v == value);
    Ok(StdValue::Boolean(contains))
}

/// 映射列表
pub fn list_map(_args: &[StdValue]) -> Result<StdValue, StdError> {
    Err(StdError::RuntimeError("list_map尚未实现".to_string()))
}

/// 过滤列表
pub fn list_filter(_args: &[StdValue]) -> Result<StdValue, StdError> {
    Err(StdError::RuntimeError("list_filter尚未实现".to_string()))
}

/// 折叠列表
pub fn list_fold(_args: &[StdValue]) -> Result<StdValue, StdError> {
    Err(StdError::RuntimeError("list_fold尚未实现".to_string()))
}

/// 列表求和
pub fn list_sum(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.is_empty() {
        return Err(StdError::InvalidArgument("list_sum需要一个参数".to_string()));
    }
    let list = args[0].as_list()?;
    if list.is_empty() {
        return Ok(StdValue::Integer(0));
    }

    let first = &list[0];
    match first {
        StdValue::Integer(_) => {
            let mut sum = 0i64;
            for item in list {
                sum += item.as_integer()?;
            }
            Ok(StdValue::Integer(sum))
        }
        StdValue::Float(_) => {
            let mut sum = 0f64;
            for item in list {
                sum += item.as_float()?;
            }
            Ok(StdValue::Float(sum))
        }
        _ => Err(StdError::TypeError("list_sum需要数字列表".to_string()))
    }
}

/// 创建范围列表
pub fn list_range(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.len() < 2 {
        return Err(StdError::InvalidArgument("list_range需要两个参数: start, end".to_string()));
    }
    let start = args[0].as_integer()?;
    let end = args[1].as_integer()?;

    let mut result = Vec::new();
    if start <= end {
        for i in start..end {
            result.push(StdValue::Integer(i));
        }
    } else {
        for i in (end+1..=start).rev() {
            result.push(StdValue::Integer(i));
        }
    }
    Ok(StdValue::List(result))
}

// ============================================
// Map 字典函数
// ============================================

/// 创建新字典
pub fn map_new(_args: &[StdValue]) -> Result<StdValue, StdError> {
    Ok(StdValue::Map(HashMap::new()))
}

/// 字典长度
pub fn map_len(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.is_empty() {
        return Err(StdError::InvalidArgument("map_len需要一个参数".to_string()));
    }
    match &args[0] {
        StdValue::Map(map) => Ok(StdValue::Integer(map.len() as i64)),
        _ => Err(StdError::TypeError("map_len需要Map类型".to_string()))
    }
}

/// 检查字典是否为空
pub fn map_is_empty(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.is_empty() {
        return Err(StdError::InvalidArgument("map_is_empty需要一个参数".to_string()));
    }
    match &args[0] {
        StdValue::Map(map) => Ok(StdValue::Boolean(map.is_empty())),
        _ => Err(StdError::TypeError("map_is_empty需要Map类型".to_string()))
    }
}

/// 获取字典值
pub fn map_get(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.len() < 2 {
        return Err(StdError::InvalidArgument("map_get需要两个参数: map, key".to_string()));
    }
    match &args[0] {
        StdValue::Map(map) => {
            let key = args[1].as_string()?;
            match map.get(key) {
                Some(v) => Ok(StdValue::Option(Box::new(option::Option::Some(v.clone()))))),
                None => Ok(StdValue::Option(Box::new(option::Option::None))),
            }
        }
        _ => Err(StdError::TypeError("map_get需要Map类型".to_string()))
    }
}

/// 插入键值对
pub fn map_insert(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.len() < 3 {
        return Err(StdError::InvalidArgument("map_insert需要三个参数: map, key, value".to_string()));
    }
    let mut result = args[0].clone();
    match &mut result {
        StdValue::Map(map) => {
            let key = args[1].as_string()?;
            map.insert(key.to_string(), args[2].clone());
            Ok(result)
        }
        _ => Err(StdError::TypeError("map_insert需要Map类型".to_string()))
    }
}

/// 移除键
pub fn map_remove(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.len() < 2 {
        return Err(StdError::InvalidArgument("map_remove需要两个参数: map, key".to_string()));
    }
    let mut result = args[0].clone();
    match &mut result {
        StdValue::Map(map) => {
            let key = args[1].as_string()?;
            match map.remove(key) {
                Some(v) => Ok(StdValue::Option(Box::new(option::Option::Some(v))))),
                None => Ok(StdValue::Option(Box::new(option::Option::None))),
            }
        }
        _ => Err(StdError::TypeError("map_remove需要Map类型".to_string()))
    }
}

/// 检查是否包含键
pub fn map_contains_key(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.len() < 2 {
        return Err(StdError::InvalidArgument("map_contains_key需要两个参数: map, key".to_string()));
    }
    match &args[0] {
        StdValue::Map(map) => {
            let key = args[1].as_string()?;
            Ok(StdValue::Boolean(map.contains_key(key)))
        }
        _ => Err(StdError::TypeError("map_contains_key需要Map类型".to_string()))
    }
}

/// 获取所有键
pub fn map_keys(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.is_empty() {
        return Err(StdError::InvalidArgument("map_keys需要一个参数".to_string()));
    }
    match &args[0] {
        StdValue::Map(map) => {
            let keys: Vec<StdValue> = map.keys().map(|k| StdValue::String(k.clone())).collect();
            Ok(StdValue::List(keys))
        }
        _ => Err(StdError::TypeError("map_keys需要Map类型".to_string()))
    }
}

/// 获取所有值
pub fn map_values(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.is_empty() {
        return Err(StdError::InvalidArgument("map_values需要一个参数".to_string()));
    }
    match &args[0] {
        StdValue::Map(map) => {
            let values: Vec<StdValue> = map.values().cloned().collect();
            Ok(StdValue::List(values))
        }
        _ => Err(StdError::TypeError("map_values需要Map类型".to_string()))
    }
}
