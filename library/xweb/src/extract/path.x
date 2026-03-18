// Path Extractor - 路径参数提取

use ../http/request.{Request}

/// Path 参数提取器
class Path {
    public var value: string

    new(value: string) {
        this.value = value
    }
}

/// 提取路径参数
function extract_path_param(request: Request, pattern: string, param_name: string) -> Option<string> {
    let pattern_parts = split_path(pattern)
    let path_parts = split_path(request.path)

    extract_param_recursive(pattern_parts, path_parts, param_name, 0)
}

function extract_param_recursive(
    pattern: List<string>,
    path: List<string>,
    param_name: string,
    index: integer
) -> Option<string> {
    if index >= list_length(pattern) || index >= list_length(path) {
        None
    } else {
        let p = list_get_string(pattern, index)
        let v = list_get_string(path, index)

        if starts_with(p, ":") && substring(p, 1, string_length(p)) == param_name {
            Some(v)
        } else {
            extract_param_recursive(pattern, path, param_name, index + 1)
        }
    }
}

extern function split_path(path: string) -> List<string>
extern function list_length(list: List) -> integer
extern function list_get_string(list: List, index: integer) -> string
extern function starts_with(s: string, prefix: string) -> boolean
extern function substring(s: string, start: integer, end: integer) -> string
extern function string_length(s: string) -> integer
