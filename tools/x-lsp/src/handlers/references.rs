//! Find references handler

use lsp_types::{
    request::References, Location, Range,
};

use crate::server::LspServer;
use crate::utils;

/// Register references handler with the server
pub fn register(server: &mut LspServer) {
    let workspace = server.workspace();
    server.register_request_handler::<References>(move |req| {
        let params: lsp_types::ReferenceParams = serde_json::from_value(req.params)?;
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        if let Some(doc) = workspace.get_document(&uri) {
            let content = doc.content();
            let offset = utils::position_to_offset(&position, content);

            // Find references at position
            let references = find_references(&doc, offset);
            let resp = lsp_server::Response::new_ok(req.id, references);
            return Ok(resp);
        }

        // No references found
        let resp = lsp_server::Response::new_ok(req.id, Vec::<Location>::new());
        Ok(resp)
    });
}

/// Find all references to the symbol at the given offset
fn find_references(
    doc: &crate::state::Document,
    offset: usize,
) -> Vec<Location> {
    let ast_binding = doc.ast();
    let program = match ast_binding.as_deref() {
        Some(p) => p,
        None => return Vec::new(),
    };

    let mut references = Vec::new();
    let content = doc.content();

    // First, find the symbol name at the offset
    let symbol_name = find_symbol_name_at_offset(program, offset);
    if symbol_name.is_none() {
        return Vec::new();
    }
    let symbol_name = symbol_name.unwrap();

    // Now find all references to this symbol
    // Check declarations
    for decl in &program.declarations {
        match decl {
            x_parser::ast::Declaration::Function(func) => {
                if func.name == symbol_name {
                    let start = func.span.start as usize;
                    let end = start + func.name.len();
                    references.push(Location {
                        uri: doc.uri().clone(),
                        range: Range {
                            start: utils::offset_to_position(start, content),
                            end: utils::offset_to_position(end, content),
                        },
                    });
                }
            }
            x_parser::ast::Declaration::Variable(var) => {
                if var.name == symbol_name {
                    let start = var.span.start as usize;
                    let end = start + var.name.len();
                    references.push(Location {
                        uri: doc.uri().clone(),
                        range: Range {
                            start: utils::offset_to_position(start, content),
                            end: utils::offset_to_position(end, content),
                        },
                    });
                }
            }
            _ => {}
        }
    }

    references
}

/// Find the symbol name at the given offset
fn find_symbol_name_at_offset(program: &x_parser::ast::Program, offset: usize) -> Option<String> {
    for decl in &program.declarations {
        match decl {
            x_parser::ast::Declaration::Function(func) => {
                let start = func.span.start as usize;
                let end = func.span.end as usize;
                if offset >= start && offset <= end {
                    return Some(func.name.clone());
                }
            }
            x_parser::ast::Declaration::Variable(var) => {
                let start = var.span.start as usize;
                let end = var.span.end as usize;
                if offset >= start && offset <= end {
                    return Some(var.name.clone());
                }
            }
            _ => {}
        }
    }

    // Check statements
    for stmt in &program.statements {
        if let x_parser::ast::StatementKind::Variable(var) = &stmt.node {
            let start = stmt.span.start as usize;
            let end = stmt.span.end as usize;
            if offset >= start && offset <= end {
                return Some(var.name.clone());
            }
        }
    }

    None
}
