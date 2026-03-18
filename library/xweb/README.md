# X-Web Framework

A minimal web framework for X language, directly using Zig's stdlib (`std.http`, `std.json`).

## Features

- **Zig stdlib integration**: Uses `zig::std.http` and `zig::std.json` directly
- **Async-first**: Built on Zig's async I/O
- **Axum-style routing**: Simple route definitions

## Quick Start

```x
use xweb.{Server, Router, Response, Request}
use xweb.{json, html}

function home(request: Request) -> Response {
    html("<h1>Hello X-Web!</h1>")
}

function api(request: Request) -> Response {
    json("{\"status\": \"ok\"}")
}

async function main() {
    let router = Router.new()
        .get("/", home)
        .get("/api", api)

    let server = Server.new()
        .port(3000)
        .router(router)

    wait server.start()
}
```

## Zig stdlib Usage

X-Web directly imports Zig standard library:

```x
use zig::std.http.{Server, Request, Response, Method}
use zig::std.json.{parseFromSlice, Value, stringify}
use zig::std.net.{Address}
```

The Zig backend converts `zig::` imports to:

```zig
const http = @import("std").http;
const json = @import("std").json;
const net = @import("std").net;
```

## API

### Server

```x
let server = Server.new()
    .port(3000)
    .host("127.0.0.1")
    .router(router)

wait server.start()
```

### Router

```x
let router = Router.new()
    .get("/", home_handler)
    .post("/users", create_user)
    .put("/users/:id", update_user)
    .delete("/users/:id", delete_user)
```

### Response

```x
Response.html("<h1>HTML</h1>")
Response.json("{\"key\": \"value\"}")
Response.text("Plain text")
Response.not_found("Not found")
Response.bad_request("Invalid input")
```

### Query Parameters

```x
use xweb.{get_query_param, get_query_param_or}

let q = get_query_param_or(request, "q", "")
```

### JSON

```x
use xweb.{parse_json, to_json, JsonValue}

// Parse JSON
given parse_json(body) {
    is Ok(value) => {
        let name = json_as_string(json_get(value, "name"))
    }
    is Err(e) => Response.bad_request("Invalid JSON")
}

// Create JSON
let json_value = json_object([
    ("name", json_string("X-Web")),
    ("version", json_string("0.1.0"))
])
let json_str = to_json(json_value)
```

## Directory Structure

```
library/xweb/
├── README.md
└── src/
    ├── lib.x              # Main entry
    ├── core/
    │   ├── server.x       # HTTP server
    │   ├── config.x       # Server config
    │   └── runtime.x      # Zig stdlib wrappers
    ├── http/
    │   ├── request.x      # Request type
    │   ├── response.x     # Response type
    │   ├── method.x       # HTTP methods
    │   └── status.x       # Status codes
    ├── routing/
    │   ├── router.x       # Router
    │   └── route.x        # Route definition
    ├── extract/
    │   ├── query.x        # Query params
    │   └── path.x         # Path params
    └── json/
        └── value.x        # JSON utilities
```

## License

MIT / Apache 2.0 / BSD 3-Clause
