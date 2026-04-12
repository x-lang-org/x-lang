module std.encoding
import std.prelude
import std.types




// === HEX 编码 ===

/// 十六进制字符表（小写）
const HEX_CHARS_LOWER: [character] = [
    '0', '1', '2', '3', '4', '5', '6', '7',
    '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
];

/// 十六进制字符表（大写）
const HEX_CHARS_UPPER: [character] = [
    '0', '1', '2', '3', '4', '5', '6', '7',
    '8', '9', 'A', 'B', 'C', 'D', 'E', 'F',
];

/// 字节编码为十六进制字符串（小写）
export fn hex_encode(bytes: [u8]) -> string {
    let mut result = "";
    for b in bytes {
        let high = ((b >> 4) & 0xF) as Int;
        let low = (b & 0xF) as Int;
        result = result ++ HEX_CHARS_LOWER[high];
        result = result ++ HEX_CHARS_LOWER[low];
    }
    result
}

/// 字节编码为十六进制字符串（大写）
export fn hex_encode_upper(bytes: [u8]) -> string {
    let mut result = "";
    for b in bytes {
        let high = ((b >> 4) & 0xF) as Int;
        let low = (b & 0xF) as Int;
        result = result ++ HEX_CHARS_UPPER[high];
        result = result ++ HEX_CHARS_UPPER[low];
    }
    result
}

/// 解析十六进制字符为数值
private fn hex_char_value(c: character) -> Option<u8> {
    when c >= '0' && c <= '9' {
        Some((c as u8) - ('0' as u8))
    } else when c >= 'a' && c <= 'f' {
        Some(10 + ((c as u8) - ('a' as u8)))
    } else when c >= 'A' && c <= 'F' {
        Some(10 + ((c as u8) - ('A' as u8)))
    } else {
        None
    }
}

/// 十六进制字符串解码为字节数组
export fn hex_decode(hex: string) -> Result<[u8], string> {
    let mut bytes: [u8] = [];
    let mut i = 0;
    let len = hex.len();
    // 处理奇数长度
    when len % 2 != 0 {
        Err("hex string has odd length");
    } else {
        while i < len {
            let c1 = hex[i];
            let c2 = hex[i + 1];
            let v1 = hex_char_value(c1);
            let v2 = hex_char_value(c2);
            match v1 {
                None => return Err("invalid hex character: '" ++ c1.to_string() ++ "'"),
                Some(v1) => match v2 {
                    None => return Err("invalid hex character: '" ++ c2.to_string() ++ "'"),
                    Some(v2) => {
                        let byte = ((v1 << 4) | v2);
                        bytes.push(byte);
                    },
                },
            };
            i = i + 2;
        }
        Ok(bytes)
    }
}

// === Base64 编码 ===

/// Base64 标准字符表
const BASE64_CHARS: [character] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M',
    'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm',
    'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '+', '/',
];

/// Base64 URL 安全字符表
const BASE64_URL_CHARS: [character] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M',
    'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm',
    'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '-', '_',
];

/// 查找 Base64 字符值
private fn base64_char_value(c: character) -> Option<u8> {
    when c >= 'A' && c <= 'Z' {
        Some((c as u8) - ('A' as u8))
    } else when c >= 'a' && c <= 'z' {
        Some(26 + ((c as u8) - ('a' as u8)))
    } else when c >= '0' && c <= '9' {
        Some(52 + ((c as u8) - ('0' as u8)))
    } else when c == '+' || c == '-' {
        Some(62)
    } else when c == '/' || c == '_' {
        Some(63)
    } else when c == '=' {
        // 填充字符
        Some(0)
    } else {
        None
    }
}

/// 字节数组编码为标准 Base64 字符串
export fn base64_encode(bytes: [u8]) -> string {
    let mut result = "";
    let n = bytes.len();
    let mut i = 0;
    while i < n {
        // 每 3 字节 = 4 字符
        let mut chunk: u32 = 0;
        let remaining = n - i;

        chunk = chunk | ((bytes[i] as u32) << 16);
        when remaining > 1 {
            chunk = chunk | ((bytes[i + 1] as u32) << 8);
        }
        when remaining > 2 {
            chunk = chunk | (bytes[i + 2] as u32);
        }

        let c1 = BASE64_CHARS[((chunk >> 18) & 0x3F) as Int];
        let c2 = BASE64_CHARS[((chunk >> 12) & 0x3F) as Int];
        result = result ++ c1 ++ c2;

        when remaining > 1 {
            let c3 = BASE64_CHARS[((chunk >> 6) & 0x3F) as Int];
            result = result ++ c3;
        }
        when remaining > 2 {
            let c4 = BASE64_CHARS[((chunk >> 0) & 0x3F) as Int];
            result = result ++ c4;
        }

        i = i + 3;
    }

    // 添加填充
    let padding = (4 - (result.len() % 4)) % 4;
    for _ in 0..padding {
        result = result ++ '=';
    }

    result
}

/// 字节数组编码为 URL 安全 Base64 字符串（无填充）
export fn base64url_encode(bytes: [u8]) -> string {
    let mut result = "";
    let n = bytes.len();
    let mut i = 0;
    while i < n {
        let mut chunk: u32 = 0;
        let remaining = n - i;

        chunk = chunk | ((bytes[i] as u32) << 16);
        when remaining > 1 {
            chunk = chunk | ((bytes[i + 1] as u32) << 8);
        }
        when remaining > 2 {
            chunk = chunk | (bytes[i + 2] as u32);
        }

        let c1 = BASE64_URL_CHARS[((chunk >> 18) & 0x3F) as Int];
        let c2 = BASE64_URL_CHARS[((chunk >> 12) & 0x3F) as Int];
        result = result ++ c1 ++ c2;

        when remaining > 1 {
            let c3 = BASE64_URL_CHARS[((chunk >> 6) & 0x3F) as Int];
            result = result ++ c3;
        }
        when remaining > 2 {
            let c4 = BASE64_URL_CHARS[((chunk >> 0) & 0x3F) as Int];
            result = result ++ c4;
        }

        i = i + 3;
    }

    // URL 编码通常不填充
    result
}

/// Base64 字符串解码为字节数组
export fn base64_decode(encoded: string) -> Result<[u8], string> {
    let mut bytes: [u8] = [];
    let len = encoded.len();
    if len % 4 != 0 {
        // 允许没有 padding，有些实现不添加
        // 我们继续尝试解码
    }

    let mut acc: u32 = 0;
    let mut bits = 0;

    let mut i = 0;
    while i < len {
        let c = encoded[i];
        when c == '=' {
            // 填充结束，停止
            break;
        }
        when c == ' ' || c == '\n' || c == '\r' || c == '\t' {
            // 忽略空白字符
            i = i + 1;
            continue;
        }
        let val = base64_char_value(c);
        match val {
            None => return Err("invalid base64 character: '" ++ c.to_string() ++ "'"),
            Some(v) => {
                acc = (acc << 6) | (v as u32);
                bits = bits + 6;
                when bits >= 8 {
                    bits = bits - 8;
                    let byte = (acc >> bits) as u8;
                    bytes.push(byte);
                }
            },
        };
        i = i + 1;
    }

    Ok(bytes)
}

/// URL 安全 Base64 解码（和标准解码相同）
export fn base64url_decode(encoded: string) -> Result<[u8], string> {
    base64_decode(encoded)
}

// === UTF-8 编码辅助 ===

/// 验证字节数组是否为合法 UTF-8
export fn validate_utf8(bytes: [u8]) -> Bool {
    let mut i = 0;
    let n = bytes.len();
    while i < n {
        let b = bytes[i];
        when (b & 0x80) == 0 {
            // 1 字节
            i = i + 1;
        } else when (b & 0xE0) == 0xC0 {
            // 2 字节
            if i + 1 >= n {
                false
            }
            if (bytes[i + 1] & 0xC0) != 0x80 {
                false
            }
            i = i + 2;
        } else when (b & 0xF0) == 0xE0 {
            // 3 字节
            if i + 2 >= n {
                false
            }
            if (bytes[i + 1] & 0xC0) != 0x80 || (bytes[i + 2] & 0xC0) != 0x80 {
                false
            }
            i = i + 3;
        } else when (b & 0xF8) == 0xF0 {
            // 4 字节
            if i + 3 >= n {
                false
            }
            if (bytes[i + 1] & 0xC0) != 0x80 || (bytes[i + 2] & 0xC0) != 0x80 || (bytes[i + 3] & 0xC0) != 0x80 {
                false
            }
            i = i + 4;
        } else {
            // 错误的起始字节
            false
        }
    }
    true
}
