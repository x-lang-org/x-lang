//! LSP request and notification handlers

use crate::server::LspServer;

// Feature handlers
pub mod completion;
pub mod definition;
pub mod references;
pub mod hover;
pub mod diagnostic;
pub mod document_symbol;

/// Register all feature handlers with the server
pub fn register_handlers(server: &mut LspServer) {
    // Register handlers here as they are implemented
    // completion::register(server);
    // definition::register(server);
    // references::register(server);
    // hover::register(server);
    diagnostic::register(server);
    // document_symbol::register(server);
}
