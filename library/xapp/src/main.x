// X-App Framework - Main Entry Point
// Next.js-style full-stack framework for X Language

export use xapp

// Re-export key types and functions
export from xapp {
    Config, App, PageRoute, ApiRoute,
    default_config, create_app, start_server,
    init_database, db_query,
    render_html, render_json,
    api_success, api_error,
    Session, create_session, set_session_user,
    StaticFile, serve_static
}

// ============================================================================
// Quick Start Example
// ============================================================================
//
// import xapp
//
// function home_page(req: xweb.Request) -> xweb.Response {
//     let html = "<h1>Welcome to X-App!</h1><p>Built with X Language</p>"
//     xapp.render_html("Home", html)
// }
//
// function api_users(req: xweb.Request) -> xweb.Response {
//     let users = "[{\"id\": 1, \"name\": \"Alice\"}]"
//     xapp.api_success(users)
// }
//
// async function main() {
//     let config = xapp.default_config()
//     let app = xapp.create_app(config)
//     app = xapp.init_database(app)
//
//     app.router = app.router
//         .get("/", home_page)
//         .get("/api/users", api_users)
//
//     xapp.start_server(app)
// }