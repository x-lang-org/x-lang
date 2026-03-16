// X语言标准库 - JSON 模块
//
// 提供JSON序列化和反序列化功能（纯 X 语言实现）

import "string"
import "collections"

// ==========================================
// JSON 解析器（纯 X 实现）
// ==========================================

/// JSON 解析器状态
type JsonParser {
    input: String,
    pos: Int,
    len: Int,
}

/// 创建 JSON 解析器
function json_parser_new(input: String): JsonParser {
    {
        input: input,
        pos: 0,
        len: len(input),
    }
}

/// 跳过空白字符
function json_skip_whitespace(mut parser: JsonParser): JsonParser {
    while parser.pos < parser.len {
        let c = str_substring(parser.input, parser.pos, parser.pos + 1)
        if c == " " || c == "\t" || c == "\n" || c == "\r" {
            parser.pos = parser.pos + 1
        } else {
            break
        }
    }
    parser
}

/// 解析 JSON 值
function json_parse_value(mut parser: JsonParser): (JsonParser, Option<String>) {
    parser = json_skip_whitespace(parser)

    if parser.pos >= parser.len {
        return (parser, None)
    }

    let c = str_substring(parser.input, parser.pos, parser.pos + 1)

    // 解析 null
    if c == "n" {
        let remaining = str_substring(parser.input, parser.pos, parser.pos + 4)
        if remaining == "null" {
            parser.pos = parser.pos + 4
            return (parser, Some("null"))
        }
    }

    // 解析布尔值
    if c == "t" {
        let remaining = str_substring(parser.input, parser.pos, parser.pos + 4)
        if remaining == "true" {
            parser.pos = parser.pos + 4
            return (parser, Some("true"))
        }
    }
    if c == "f" {
        let remaining = str_substring(parser.input, parser.pos, parser.pos + 5)
        if remaining == "false" {
            parser.pos = parser.pos + 5
            return (parser, Some("false"))
        }
    }

    // 解析字符串
    if c == "\"" {
        parser.pos = parser.pos + 1
        let mut result = "\""
        while parser.pos < parser.len {
            let ch = str_substring(parser.input, parser.pos, parser.pos + 1)
            parser.pos = parser.pos + 1
            result = result + ch
            if ch == "\"" {
                break
            }
            if ch == "\\" && parser.pos < parser.len {
                let escaped = str_substring(parser.input, parser.pos, parser.pos + 1)
                parser.pos = parser.pos + 1
                result = result + escaped
            }
        }
        return (parser, Some(result))
    }

    // 解析数字
    if c == "-" || (c >= "0" && c <= "9") {
        let mut result = ""
        while parser.pos < parser.len {
            let ch = str_substring(parser.input, parser.pos, parser.pos + 1)
            if ch >= "0" && ch <= "9" || ch == "-" || ch == "." || ch == "e" || ch == "E" || ch == "+" {
                result = result + ch
                parser.pos = parser.pos + 1
            } else {
                break
            }
        }
        return (parser, Some(result))
    }

    // 解析数组
    if c == "[" {
        parser.pos = parser.pos + 1
        let mut result = "["
        parser = json_skip_whitespace(parser)

        if parser.pos < parser.len {
            let next = str_substring(parser.input, parser.pos, parser.pos + 1)
            if next == "]" {
                parser.pos = parser.pos + 1
                return (parser, Some("[]"))
            }

            while parser.pos < parser.len {
                let parse_result = json_parse_value(parser)
                parser = parse_result.0
                given parse_result.1 {
                    is Some(v) => {
                        if len(result) > 1 {
                            result = result + ","
                        }
                        result = result + v
                    }
                    is None => break
                }

                parser = json_skip_whitespace(parser)
                if parser.pos >= parser.len {
                    break
                }

                let ch = str_substring(parser.input, parser.pos, parser.pos + 1)
                if ch == "]" {
                    parser.pos = parser.pos + 1
                    break
                }
                if ch == "," {
                    parser.pos = parser.pos + 1
                }
            }
        }
        result = result + "]"
        return (parser, Some(result))
    }

    // 解析对象
    if c == "{" {
        parser.pos = parser.pos + 1
        let mut result = "{"
        parser = json_skip_whitespace(parser)

        if parser.pos < parser.len {
            let next = str_substring(parser.input, parser.pos, parser.pos + 1)
            if next == "}" {
                parser.pos = parser.pos + 1
                return (parser, Some("{}"))
            }

            while parser.pos < parser.len {
                parser = json_skip_whitespace(parser)
                if parser.pos >= parser.len {
                    break
                }

                // 解析键
                let key_result = json_parse_value(parser)
                parser = key_result.0
                given key_result.1 {
                    is Some(k) => {
                        if len(result) > 1 {
                            result = result + ","
                        }
                        result = result + k
                    }
                    is None => break
                }

                parser = json_skip_whitespace(parser)
                if parser.pos >= parser.len {
                    break
                }

                // 期望冒号
                let colon = str_substring(parser.input, parser.pos, parser.pos + 1)
                if colon != ":" {
                    break
                }
                parser.pos = parser.pos + 1
                result = result + ":"

                // 解析值
                let val_result = json_parse_value(parser)
                parser = val_result.0
                given val_result.1 {
                    is Some(v) => result = result + v
                    is None => break
                }

                parser = json_skip_whitespace(parser)
                if parser.pos >= parser.len {
                    break
                }

                let ch = str_substring(parser.input, parser.pos, parser.pos + 1)
                if ch == "}" {
                    parser.pos = parser.pos + 1
                    break
                }
                if ch == "," {
                    parser.pos = parser.pos + 1
                }
            }
        }
        result = result + "}"
        return (parser, Some(result))
    }

    (parser, None)
}

/// 解析 JSON 字符串
function json_parse(input: String): Result<String, String> {
    let parser = json_parser_new(input)
    let result = json_parse_value(parser)
    given result.1 {
        is Some(v) => Ok(v)
        is None => Err("Failed to parse JSON")
    }
}

// ==========================================
// JSON 序列化
// ==========================================

/// 将整数序列化为 JSON
function json_from_int(n: Int): String {
    to_string(n)
}

/// 将浮点数序列化为 JSON
function json_from_float(f: Float): String {
    to_string(f)
}

/// 将布尔值序列化为 JSON
function json_from_bool(b: Bool): String {
    if b { "true" } else { "false" }
}

/// 将字符串序列化为 JSON
function json_from_string(s: String): String {
    let mut result = "\""
    let mut i = 0
    let length = len(s)
    while i < length {
        let c = str_substring(s, i, i + 1)
        if c == "\"" {
            result = result + "\\\""
        } else if c == "\\" {
            result = result + "\\\\"
        } else if c == "\n" {
            result = result + "\\n"
        } else if c == "\t" {
            result = result + "\\t"
        } else {
            result = result + c
        }
        i = i + 1
    }
    result + "\""
}

/// 将 null 序列化为 JSON
function json_null(): String {
    "null"
}

/// 创建 JSON 数组
function json_array(items: [String]): String {
    let mut result = "["
    let mut i = 0
    while i < len(items) {
        if i > 0 {
            result = result + ","
        }
        result = result + items[i]
        i = i + 1
    }
    result + "]"
}

/// 创建 JSON 对象
function json_object(entries: [(String, String)]): String {
    let mut result = "{"
    let mut i = 0
    while i < len(entries) {
        if i > 0 {
            result = result + ","
        }
        let entry = entries[i]
        result = result + json_from_string(entry.0)
        result = result + ":"
        result = result + entry.1
        i = i + 1
    }
    result + "}"
}

// ==========================================
// 辅助函数
// ==========================================

/// 获取 JSON 值的类型
function json_type(json: String): String {
    let trimmed = str_trim(json)
    if trimmed == "null" { return "null" }
    if trimmed == "true" || trimmed == "false" { return "boolean" }
    if str_starts_with(trimmed, "\"") { return "string" }
    if str_starts_with(trimmed, "[") { return "array" }
    if str_starts_with(trimmed, "{") { return "object" }
    "number"
}
