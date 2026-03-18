// JSON - 直接使用 Zig std.json

use zig::std.json.{Value, parseFromSlice, stringify, stringifyAlloc}

/// JSON 值类型 (直接使用 Zig 的)
typealias JsonValue = zig::std.json.Value

/// 解析 JSON 字符串
function parse_json(json_string: string) -> Result<JsonValue, string> {
    parseFromSlice(allocator, json_string, .{})
}

/// 将值序列化为 JSON 字符串
function to_json(value: anytype) -> string {
    stringifyAlloc(allocator, value, .{})
}

/// 获取对象字段
function json_get(obj: JsonValue, key: string) -> ?JsonValue {
    obj.object.get(key)
}

/// 获取数组元素
function json_get_index(arr: JsonValue, index: integer) -> ?JsonValue {
    if index < arr.array.items.len {
        arr.array.items[index]
    } else {
        null
    }
}

/// 获取字符串值
function json_as_string(value: JsonValue) -> ?string {
    if value == .string {
        value.string
    } else {
        null
    }
}

/// 获取整数值
function json_as_int(value: JsonValue) -> ?integer {
    if value == .integer {
        value.integer
    } else {
        null
    }
}

/// 获取浮点数值
function json_as_float(value: JsonValue) -> ?float {
    if value == .float {
        value.float
    } else if value == .integer {
        @floatFromInt(value.integer)
    } else {
        null
    }
}

/// 获取布尔值
function json_as_bool(value: JsonValue) -> ?boolean {
    if value == .bool {
        value.bool
    } else {
        null
    }
}

/// 检查是否为 null
function json_is_null(value: JsonValue) -> boolean {
    value == .null
}

// ========================================
// JSON 构造器
// ========================================

/// 创建 null 值
function json_null() -> JsonValue {
    .null
}

/// 创建布尔值
function json_bool(b: boolean) -> JsonValue {
    .{ .bool = b }
}

/// 创建整数值
function json_int(n: integer) -> JsonValue {
    .{ .integer = n }
}

/// 创建浮点数值
function json_float(n: float) -> JsonValue {
    .{ .float = n }
}

/// 创建字符串值
function json_string(s: string) -> JsonValue {
    .{ .string = s }
}

/// 创建数组值
function json_array(items: List<JsonValue>) -> JsonValue {
    .{ .array = .{ .items = items } }
}

/// 创建对象值
function json_object(fields: Map<string, JsonValue>) -> JsonValue {
    .{ .object = fields }
}
