module std.string

import std.prelude;
import std.types;
import std.types { Option, Some, None };
import std.prelude { panic };

/// 字符串长度
extern function strlen(s: *character) -> unsigned 64-bit integer

/// 获取字符串长度
export function length(s: string) -> unsigned 64-bit integer {
    unsafe {
        strlen(s.as(*character))
    }
}

/// 字符串是否为空
export function is_empty(s: string) -> boolean {
    length(s) == 0
}

/// 字符串拼接
export function concat(a: string, b: string) -> string {
    a ++ b
}

/// 重复字符串
export function repeat(s: string, n: unsigned 64-bit integer) -> string {
    let mut result = "";
    let mut i = 0.as(unsigned 64-bit integer);
    
    while i < n {
        result = result ++ s;
        i = i + 1;
    }
    
    result
}

/// 字符串包含
extern function strstr(haystack: *character, needle: *character) -> *character

export function contains(s: string, substring: string) -> boolean {
    unsafe {
        let result = strstr(s.as(*character), substring.as(*character));
        not (result is Pointer.null())
    }
}

/// 字符串以指定前缀开始
export function starts_with(s: string, prefix: string) -> boolean {
    if prefix.length() > s.length() {
        return false;
    }
    
    let mut i = 0;
    while i < prefix.length().as(signed 64-bit integer) {
        if s[i] != prefix[i] {
            return false;
        }
        i = i + 1;
    }
    
    true
}

/// 字符串以指定后缀结束
export function ends_with(s: string, suffix: string) -> boolean {
    if suffix.length() > s.length() {
        return false;
    }
    
    let start = s.length().as(signed 64-bit integer) - suffix.length().as(signed 64-bit integer);
    let mut i = 0;
    
    while i < suffix.length().as(signed 64-bit integer) {
        if s[start + i] != suffix[i] {
            return false;
        }
        i = i + 1;
    }
    
    true
}

/// 查找子字符串位置
export function find(s: string, substring: string) -> Option<signed 64-bit integer> {
    if substring.length() == 0 {
        return Some(0);
    }
    
    if substring.length() > s.length() {
        return None;
    }
    
    let mut i = 0;
    let max_pos = (s.length() - substring.length()).as(signed 64-bit integer);
    
    while i <= max_pos {
        let mut match = true;
        let mut j = 0;
        
        while j < substring.length().as(signed 64-bit integer) {
            if s[i + j] != substring[j] {
                match = false;
                break;
            }
            j = j + 1;
        }
        
        if match {
            return Some(i);
        }
        
        i = i + 1;
    }
    
    None
}

/// 从右侧查找子字符串
export function rfind(s: string, substring: string) -> Option<signed 64-bit integer> {
    if substring.length() == 0 {
        return Some(s.length().as(signed 64-bit integer));
    }
    
    if substring.length() > s.length() {
        return None;
    }
    
    let mut i = (s.length() - substring.length()).as(signed 64-bit integer);
    
    while i >= 0 {
        let mut match = true;
        let mut j = 0;
        
        while j < substring.length().as(signed 64-bit integer) {
            if s[i + j] != substring[j] {
                match = false;
                break;
            }
            j = j + 1;
        }
        
        if match {
            return Some(i);
        }
        
        i = i - 1;
    }
    
    None
}

/// 子字符串
export function substring(s: string, start: signed 64-bit integer, end: signed 64-bit integer) -> string {
    if start < 0 or end > s.length().as(signed 64-bit integer) or start > end {
        return "";
    }
    
    let mut result = "";
    let mut i = start;
    
    while i < end {
        result = result ++ s[i].as(string);
        i = i + 1;
    }
    
    result
}

/// 去除左侧空白
export function trim_left(s: string) -> string {
    let mut i = 0;
    
    while i < s.length().as(signed 64-bit integer) {
        let c = s[i];
        if c != ' ' and c != '\t' and c != '\n' and c != '\r' {
            break;
        }
        i = i + 1;
    }
    
    substring(s, i, s.length().as(signed 64-bit integer))
}

/// 去除右侧空白
export function trim_right(s: string) -> string {
    let mut i = s.length().as(signed 64-bit integer) - 1;
    
    while i >= 0 {
        let c = s[i];
        if c != ' ' and c != '\t' and c != '\n' and c != '\r' {
            break;
        }
        i = i - 1;
    }
    
    substring(s, 0, i + 1)
}

/// 去除两侧空白
export function trim(s: string) -> string {
    trim_right(trim_left(s))
}

/// 转换为大写
export function to_upper(s: string) -> string {
    let mut result = "";
    let mut i = 0;
    
    while i < s.length().as(signed 64-bit integer) {
        let c = s[i];
        if c >= 'a' and c <= 'z' {
            result = result ++ (c.as(signed 32-bit integer) - 32).as(character).as(string);
        } else {
            result = result ++ c.as(string);
        }
        i = i + 1;
    }
    
    result
}

/// 转换为小写
export function to_lower(s: string) -> string {
    let mut result = "";
    let mut i = 0;
    
    while i < s.length().as(signed 64-bit integer) {
        let c = s[i];
        if c >= 'A' and c <= 'Z' {
            result = result ++ (c.as(signed 32-bit integer) + 32).as(character).as(string);
        } else {
            result = result ++ c.as(string);
        }
        i = i + 1;
    }
    
    result
}

/// 替换子字符串
export function replace(s: string, from: string, to: string) -> string {
    if from.length() == 0 {
        return s;
    }
    
    let mut result = "";
    let mut i = 0;
    
    while i < s.length().as(signed 64-bit integer) {
        if i + from.length().as(signed 64-bit integer) <= s.length().as(signed 64-bit integer) {
            let mut match = true;
            let mut j = 0;
            
            while j < from.length().as(signed 64-bit integer) {
                if s[i + j] != from[j] {
                    match = false;
                    break;
                }
                j = j + 1;
            }
            
            if match {
                result = result ++ to;
                i = i + from.length().as(signed 64-bit integer);
                continue;
            }
        }
        
        result = result ++ s[i].as(string);
        i = i + 1;
    }
    
    result
}

/// 分割字符串
export function split(s: string, delimiter: string) -> [string] {
    if delimiter.length() == 0 {
        return [s];
    }
    
    let mut result: [string] = [];
    let mut start = 0;
    let mut i = 0;
    
    while i < s.length().as(signed 64-bit integer) {
        if i + delimiter.length().as(signed 64-bit integer) <= s.length().as(signed 64-bit integer) {
            let mut match = true;
            let mut j = 0;
            
            while j < delimiter.length().as(signed 64-bit integer) {
                if s[i + j] != delimiter[j] {
                    match = false;
                    break;
                }
                j = j + 1;
            }
            
            if match {
                result = result ++ [substring(s, start, i)];
                start = i + delimiter.length().as(signed 64-bit integer);
                i = start;
                continue;
            }
        }
        
        i = i + 1;
    }
    
    result = result ++ [substring(s, start, s.length().as(signed 64-bit integer))];
    result
}

/// 用分隔符连接字符串数组
export function join(parts: [string], delimiter: string) -> string {
    if parts.length() == 0 {
        return "";
    }
    
    let mut result = parts[0];
    let mut i = 1;
    
    while i < parts.length().as(signed 64-bit integer) {
        result = result ++ delimiter ++ parts[i];
        i = i + 1;
    }
    
    result
}

/// 将整数转换为字符串
extern function snprintf(buffer: *character, size: unsigned 64-bit integer, format: *character, ...) -> signed 32-bit integer

export function from_int(n: signed 64-bit integer) -> string {
    unsafe {
        let buffer: [character; 32] = ['\0'; 32];
        snprintf(buffer.as(*character), 32, "%lld".as(*character), n);
        buffer_to_string(buffer)
    }
}

/// 将浮点数转换为字符串
export function from_float(f: 64-bit float) -> string {
    unsafe {
        let buffer: [character; 64] = ['\0'; 64];
        snprintf(buffer.as(*character), 64, "%f".as(*character), f);
        buffer_to_string(buffer)
    }
}

/// 将布尔值转换为字符串
export function from_bool(b: boolean) -> string {
    if b { "true" } else { "false" }
}

/// 缓冲区转字符串
function buffer_to_string(buffer: [character; 32]) -> string {
    let mut s = "";
    let mut i = 0;
    
    while i < 32 and buffer[i] != '\0' {
        s = s ++ buffer[i].as(string);
        i = i + 1;
    }
    
    s
}
