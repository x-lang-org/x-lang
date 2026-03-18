// X-Web - X Language Web Framework
// 使用 Zig 标准库的 http 和 json

// ========================================
// Core
// ========================================

export use ./core/server.{Server}
export use ./core/config.{ServerConfig}

// ========================================
// HTTP Types
// ========================================

export use ./http/request.{Request, build_request}
export use ./http/response.{Response, IntoResponse, new_response, html, json, text, ok, created, no_content, bad_request, not_found, internal_server_error}
export use ./http/status.{Status, status_code}
export use ./http/method.{Method, parse_method, method_to_string}

// ========================================
// Routing
// ========================================

export use ./routing/router.{Router}
export use ./routing/route.{Route}

// ========================================
// Extractors
// ========================================

export use ./extract/query.{Query, get_query_param, get_query_param_or}
export use ./extract/path.{Path}

// ========================================
// JSON (直接使用 Zig std.json)
// ========================================

export use ./json/value.{JsonValue, parse_json, to_json, json_get, json_as_string, json_as_int, json_as_float, json_as_bool, json_is_null, json_null, json_bool, json_int, json_float, json_string}
