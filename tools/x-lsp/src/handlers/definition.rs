//! Go to definition handler

use lsp_types::{request::GotoDefinition, Location, Range};

use crate::server::LspServer;
use crate::utils;

/// Register definition handler with the server
pub fn register(server: &mut LspServer) {
    let workspace = server.workspace();
    server.register_request_handler::<GotoDefinition>(move |req| {
        let params: lsp_types::GotoDefinitionParams = serde_json::from_value(req.params)?;
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        if let Some(doc) = workspace.get_document(&uri) {
            let content = doc.content();
            let offset = utils::position_to_offset(&position, content);

            // Find definition at position
            if let Some(location) = find_definition(&doc, offset) {
                let resp = lsp_server::Response::new_ok(req.id, Some(location));
                return Ok(resp);
            }
        }

        // No definition found
        let resp = lsp_server::Response::new_ok(req.id, Option::<Location>::None);
        Ok(resp)
    });
}

/// Find definition at the given offset
fn find_definition(doc: &crate::state::Document, offset: usize) -> Option<Location> {
    let ast = doc.ast()?;
    let program = ast.as_ref();

    // Look for declarations and return their location
    for decl in &program.declarations {
        match decl {
            x_parser::ast::Declaration::Function(func) => {
                // Check if offset is within the function name
                let name_start = func.span.start;
                let name_end = name_start + func.name.len();

                if offset >= name_start && offset <= name_end {
                    let content = doc.content();
                    return Some(Location {
                        uri: doc.uri().clone(),
                        range: Range {
                            start: utils::offset_to_position(name_start, content),
                            end: utils::offset_to_position(name_end, content),
                        },
                    });
                }
            }
            x_parser::ast::Declaration::Variable(var) => {
                if let Some(name) = var.simple_name() {
                    let name_start = var.span.start;
                    let name_end = name_start + name.len();

                    if offset >= name_start && offset <= name_end {
                        let content = doc.content();
                        return Some(Location {
                            uri: doc.uri().clone(),
                            range: Range {
                                start: utils::offset_to_position(name_start, content),
                                end: utils::offset_to_position(name_end, content),
                            },
                        });
                    }
                }
            }
            _ => {}
        }
    }

    None
}
