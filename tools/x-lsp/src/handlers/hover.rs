//! Hover provider handler

use lsp_types::{
    request::HoverRequest,
    Hover, HoverContents, MarkedString,
};

use crate::server::LspServer;
use crate::utils;

/// Register hover handler with the server
pub fn register(server: &mut LspServer) {
    let workspace = server.workspace();
    server.register_request_handler::<HoverRequest>(move |req| {
        let params: lsp_types::HoverParams = serde_json::from_value(req.params)?;
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        if let Some(doc) = workspace.get_document(&uri) {
            let content = doc.content();
            let offset = utils::position_to_offset(&position, content);

            // Find symbol at position and provide hover info
            if let Some(hover_info) = get_hover_info(&doc, offset) {
                let resp = lsp_server::Response::new_ok(req.id, hover_info);
                return Ok(resp);
            }
        }

        // Return null if no hover info available
        let resp = lsp_server::Response::new_ok(req.id, Option::<Hover>::None);
        Ok(resp)
    });
}

/// Get hover information for a position in the document
fn get_hover_info(doc: &crate::state::Document, offset: usize) -> Option<Hover> {
    // Get AST and try to find symbol at position
    let ast = doc.ast()?;
    let program = ast.as_ref();

    // Look for function declarations
    for decl in &program.declarations {
        match decl {
            x_parser::ast::Declaration::Function(func) => {
                let func_name_start = func.name.len() + 4; // "fn " or "function "
                if offset >= func.span.start as usize - func_name_start
                    && offset <= func.span.end as usize
                {
                    // Hovering over function name
                    let mut type_str = String::from("fn ");
                    type_str.push_str(&func.name);
                    type_str.push('(');

                    let params: Vec<String> = func
                        .parameters
                        .iter()
                        .map(|p| {
                            let mut s = p.name.clone();
                            if let Some(ty) = &p.type_annot {
                                s.push_str(": ");
                                s.push_str(&ty.to_string());
                            }
                            s
                        })
                        .collect();
                    type_str.push_str(&params.join(", "));
                    type_str.push(')');

                    if let Some(ret) = &func.return_type {
                        type_str.push_str(" -> ");
                        type_str.push_str(&ret.to_string());
                    }

                    return Some(Hover {
                        contents: HoverContents::Scalar(MarkedString::String(type_str)),
                        range: None,
                    });
                }
            }
            x_parser::ast::Declaration::Variable(var) => {
                if offset >= var.span.start as usize && offset <= var.span.end as usize {
                    let mut type_str = String::new();
                    if var.is_mutable {
                        type_str.push_str("mut ");
                    }
                    type_str.push_str("let ");
                    type_str.push_str(&var.name);

                    if let Some(ty) = &var.type_annot {
                        type_str.push_str(": ");
                        type_str.push_str(&ty.to_string());
                    }

                    return Some(Hover {
                        contents: HoverContents::Scalar(MarkedString::String(type_str)),
                        range: None,
                    });
                }
            }
            _ => {}
        }
    }

    // Look for type annotations in statements
    for stmt in &program.statements {
        if let x_parser::ast::StatementKind::Variable(var) = &stmt.node {
            if offset >= stmt.span.start as usize && offset <= stmt.span.end as usize {
                let mut type_str = String::new();
                if var.is_mutable {
                    type_str.push_str("mut ");
                }
                type_str.push_str("let ");
                type_str.push_str(&var.name);

                if let Some(ty) = &var.type_annot {
                    type_str.push_str(": ");
                    type_str.push_str(&ty.to_string());
                }

                return Some(Hover {
                    contents: HoverContents::Scalar(MarkedString::String(type_str)),
                    range: None,
                });
            }
        }
    }

    None
}
