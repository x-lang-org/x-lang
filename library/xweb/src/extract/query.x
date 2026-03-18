// Query Extractor - 查询参数提取

use ../http/request.{Request}

/// Query 参数提取器
class Query {
    public var params: Map<string, string>

    new(params: Map<string, string>) {
        this.params = params
    }
}

/// 获取查询参数
function get_query_param(request: Request, name: string) -> Option<string> {
    let params = parse_query_string(request.query)
    map_get(params, name)
}

/// 获取查询参数（带默认值）
function get_query_param_or(request: Request, name: string, default: string) -> string {
    match get_query_param(request, name) {
        is Some(value) => value,
        is None => default
    }
}

/// 解析查询字符串
function parse_query_string(query: string) -> Map<string, string> {
    if string_length(query) == 0 {
        map_new()
    } else {
        parse_query_recursive(query, 0, map_new())
    }
}

function parse_query_recursive(
    query: string,
    start: integer,
    acc: Map<string, string>
) -> Map<string, string> {
    if start >= string_length(query) {
        acc
    } else {
        let amp_pos = find_char_from(query, '&', start)
        let end = if amp_pos < 0 { string_length(query) } else { amp_pos }

        let pair = substring(query, start, end)
        let eq_pos = find_char(pair, '=')

        let new_acc = if eq_pos < 0 {
            map_insert(acc, url_decode(pair), "")
        } else {
            let key = url_decode(substring(pair, 0, eq_pos))
            let value = url_decode(substring(pair, eq_pos + 1, string_length(pair)))
            map_insert(acc, key, value)
        }

        let next_start = if amp_pos < 0 { string_length(query) } else { amp_pos + 1 }
        parse_query_recursive(query, next_start, new_acc)
    }
}

extern function string_length(s: string) -> integer
extern function find_char_from(s: string, c: character, start: integer) -> integer
extern function substring(s: string, start: integer, end: integer) -> string
extern function find_char(s: string, c: character) -> integer
extern function url_decode(s: string) -> string
extern function map_new() -> Map<string, string>
extern function map_get(map: Map, key: string) -> Option<string>
extern function map_insert(map: Map, key: string, value: string) -> Map<string, string>
