//! Document symbols handler (Outline)

#![allow(deprecated)] // SymbolInformation::deprecated is deprecated in favor of tags

use lsp_types::{Location, Range, SymbolInformation, SymbolKind};

use crate::server::LspServer;
use crate::utils;

/// Register document symbol handler with the server
pub fn register(server: &mut LspServer) {
    let workspace = server.workspace();
    server.register_request_handler::<lsp_types::request::DocumentSymbolRequest>(move |req| {
        let params: lsp_types::DocumentSymbolParams = serde_json::from_value(req.params)?;
        let uri = params.text_document.uri;

        if let Some(doc) = workspace.get_document(&uri) {
            let symbols = get_document_symbols(&doc);

            let resp = lsp_server::Response::new_ok(req.id, symbols);
            return Ok(resp);
        }

        let resp = lsp_server::Response::new_ok(req.id, Vec::<SymbolInformation>::new());
        Ok(resp)
    });
}

/// Get all symbols in the document
fn get_symbols_for_document(doc: &crate::state::Document) -> Vec<SymbolInformation> {
    let ast = match doc.ast() {
        Some(a) => a.as_ref(),
        None => return Vec::new(),
    };

    let mut symbols = Vec::new();

    // Get symbols from declarations
    for decl in &ast.declarations {
        match decl {
            x_parser::ast::Declaration::Function(func) => {
                let range = utils::span_to_range(&func.span, doc.content());
                symbols.push(SymbolInformation {
                    name: func.name.clone(),
                    kind: SymbolKind::FUNCTION,
                    location: Location {
                        uri: doc.uri().clone(),
                        range,
                    },
                    container_name: None,
                    tags: None,
                    deprecated: None,
                });
            }
            x_parser::ast::Declaration::Variable(var) => {
                if let Some(name) = var.simple_name() {
                    let range = utils::span_to_range(&var.span, doc.content());
                    symbols.push(SymbolInformation {
                        name: name.to_string(),
                        kind: SymbolKind::VARIABLE,
                        location: Location {
                            uri: doc.uri().clone(),
                            range,
                        },
                        container_name: None,
                        tags: None,
                        deprecated: None,
                    });
                }
            }
            x_parser::ast::Declaration::Class(class) => {
                // Assuming span exists
                let range = Range::default();
                symbols.push(SymbolInformation {
                    name: class.name.clone(),
                    kind: SymbolKind::CLASS,
                    location: Location {
                        uri: doc.uri().clone(),
                        range,
                    },
                    container_name: None,
                    tags: None,
                    deprecated: None,
                });

                // Add methods
                for member in &class.members {
                    match member {
                        x_parser::ast::ClassMember::Method(method) => {
                            symbols.push(SymbolInformation {
                                name: method.name.clone(),
                                kind: SymbolKind::METHOD,
                                location: Location {
                                    uri: doc.uri().clone(),
                                    range: Range::default(),
                                },
                                container_name: Some(class.name.clone()),
                                tags: None,
                                deprecated: None,
                            });
                        }
                        x_parser::ast::ClassMember::Field(field) => {
                            if let Some(name) = field.simple_name() {
                                symbols.push(SymbolInformation {
                                    name: name.to_string(),
                                    kind: SymbolKind::FIELD,
                                    location: Location {
                                        uri: doc.uri().clone(),
                                        range: utils::span_to_range(&field.span, doc.content()),
                                    },
                                    container_name: Some(class.name.clone()),
                                    tags: None,
                                    deprecated: None,
                                });
                            }
                        }
                        _ => {}
                    }
                }
            }
            x_parser::ast::Declaration::Trait(trait_decl) => {
                symbols.push(SymbolInformation {
                    name: trait_decl.name.clone(),
                    kind: SymbolKind::INTERFACE,
                    location: Location {
                        uri: doc.uri().clone(),
                        range: Range::default(),
                    },
                    container_name: None,
                    tags: None,
                    deprecated: None,
                });
            }
            x_parser::ast::Declaration::TypeAlias(alias) => {
                symbols.push(SymbolInformation {
                    name: alias.name.clone(),
                    kind: SymbolKind::TYPE_PARAMETER,
                    location: Location {
                        uri: doc.uri().clone(),
                        range: Range::default(),
                    },
                    container_name: None,
                    tags: None,
                    deprecated: None,
                });
            }
            _ => {}
        }
    }

    // Get symbols from statements
    for stmt in &ast.statements {
        if let x_parser::ast::StatementKind::Variable(var) = &stmt.node {
            if let Some(name) = var.simple_name() {
                let range = utils::span_to_range(&stmt.span, doc.content());
                symbols.push(SymbolInformation {
                    name: name.to_string(),
                    kind: SymbolKind::VARIABLE,
                    location: Location {
                        uri: doc.uri().clone(),
                        range,
                    },
                    container_name: None,
                    tags: None,
                    deprecated: None,
                });
            }
        }
    }

    symbols
}

/// Get document symbols (alternative hierarchical representation)
fn get_document_symbols(doc: &crate::state::Document) -> Vec<SymbolInformation> {
    get_symbols_for_document(doc)
}
