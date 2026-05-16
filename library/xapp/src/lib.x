// X-App - Next.js-style Full-Stack Framework for X Language
//
// Features:
// - File-based routing (like Next.js pages/)
// - Built-in database (SQLite-like via std.sqlite)
// - API routes for backend endpoints
// - SSR-style page rendering
// - Single binary deployment

module xapp

import std.prelude
import xweb
import std.sqlite

// ============================================================================
// App Configuration
// ============================================================================

export record Config {
    name: string
    version: string
    database_path: string
    port: Int
    host: string
}

export function default_config() -> Config {
    Config {
        name: "xapp",
        version: "0.1.0",
        database_path: "app.db",
        port: 3000,
        host: "127.0.0.1"
    }
}

// ============================================================================
// Application State
// ============================================================================

export record App {
    config: Config
    db: Database
    router: xweb.Router
}

export function create_app(config: Config) -> App {
    let db = std.sqlite.new_db()
    App {
        config: config,
        db: db,
        router: xweb.Router.new()
    }
}

// ============================================================================
// Route Definitions (Next.js-style)
// ============================================================================

// Page route
export record PageRoute {
    path: string
    handler: function(xweb.Request) -> xweb.Response
}

// API route
export record ApiRoute {
    path: string
    method: string  // GET, POST, PUT, DELETE
    handler: function(xweb.Request) -> xweb.Response
}

// ============================================================================
// Database Integration
// ============================================================================

export function init_database(app: App) -> App {
    // Create users table if not exists
    let user_cols = [
        std.sqlite.Column { name: "id", col_type: std.sqlite.SqlType.Integer },
        std.sqlite.Column { name: "email", col_type: std.sqlite.SqlType.Text },
        std.sqlite.Column { name: "name", col_type: std.sqlite.SqlType.Text },
        std.sqlite.Column { name: "password_hash", col_type: std.sqlite.SqlType.Text }
    ]
    let new_db = std.sqlite.create_table(app.db, "users", user_cols)
    App {
        config: app.config,
        db: new_db,
        router: app.router
    }
}

export function db_query(app: App, sql: string, params: [SqlValue]) -> Result {
    std.sqlite.insert(app.db, "users", params)
}

// ============================================================================
// HTTP Server
// ============================================================================

export function start_server(app: App) -> unit {
    let server = xweb.Server.new()
        .port(app.config.port)
        .host(app.config.host)
        .router(app.router)

    println("X-App server starting on http://" + app.config.host + ":" + int_to_string(app.config.port))
    // Note: server.start() would be async
}

// ============================================================================
// Page Rendering (SSR-style)
// ============================================================================

export function render_html(title: string, content: string) -> xweb.Response {
    let html = "<!DOCTYPE html><html><head><title>" + title + "</title></head><body>" + content + "</body></html>"
    xweb.html(html)
}

export function render_json(data: string) -> xweb.Response {
    xweb.json(data)
}

// ============================================================================
// Session/Cookie Helpers
// ============================================================================

export record Session {
    user_id: Option<Int>
    data: Map<string, string>
}

export function create_session() -> Session {
    Session { user_id: None, data: std.map.empty<string, string>() }
}

export function set_session_user(session: Session, user_id: Int) -> Session {
    Session { user_id: Some(user_id), data: session.data }
}

// ============================================================================
// Static File Serving
// ============================================================================

export record StaticFile {
    path: string
    content_type: string
    content: string
}

export function serve_static(file_path: string) -> StaticFile {
    let content = std.fs.read_file(file_path)
    let mime_type = guess_mime_type(file_path)
    StaticFile { path: file_path, content_type: mime_type, content: content }
}

function guess_mime_type(path: string) -> string {
    if string.ends_with(path, ".html") {
        "text/html"
    } else if string.ends_with(path, ".css") {
        "text/css"
    } else if string.ends_with(path, ".js") {
        "application/javascript"
    } else if string.ends_with(path, ".png") {
        "image/png"
    } else if string.ends_with(path, ".jpg") or string.ends_with(path, ".jpeg") {
        "image/jpeg"
    } else {
        "application/octet-stream"
    }
}

// ============================================================================
// API Response Helpers
// ============================================================================

export function api_success(data: string) -> xweb.Response {
    let response = "{\"success\": true, \"data\": " + data + "}"
    xweb.json(response)
}

export function api_error(message: string, code: Int) -> xweb.Response {
    let response = "{\"success\": false, \"error\": \"" + message + "\", \"code\": " + int_to_string(code) + "}"
    xweb.json(response)
}