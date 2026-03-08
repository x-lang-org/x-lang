//! JSON 模块
//! 提供JSON序列化和反序列化功能

/// 将值序列化为JSON字符串
/// 
/// # 参数
/// - `value`: 要序列化的值
/// 
/// # 返回值
/// 序列化后的JSON字符串
function to_json(value) {
    return x_to_json(value)
}

/// 解析JSON字符串为值
/// 
/// # 参数
/// - `json_str`: JSON字符串
/// 
/// # 返回值
/// 解析后的值（可能是map、array、string、number、bool或null）
function parse(json_str) {
    return x_json_parse(json_str)
}
