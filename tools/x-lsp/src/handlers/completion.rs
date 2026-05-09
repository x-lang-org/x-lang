//! Code completion handler

use std::sync::LazyLock;

use lsp_types::{
    request::Completion, CompletionItem, CompletionItemKind, CompletionItemLabelDetails,
    CompletionList, InsertTextMode,
};

use crate::server::LspServer;

// Cached static completion data - computed once at startup
static KEYWORDS_COMPLETIONS: LazyLock<Vec<CompletionItem>> = LazyLock::new(|| {
    vec![
        "needs", "given", "wait", "when", "is", "can", "atomic", "fn", "func", "function", "let",
        "var", "const", "if", "else", "while", "for", "loop", "match", "return", "break",
        "continue", "struct", "enum", "type", "impl", "trait", "use", "mod", "pub", "mut", "ref",
        "self", "Self", "true", "false", "null", "None", "Some", "Ok", "Err",
    ]
    .iter()
    .map(|k| CompletionItem {
        label: k.to_string(),
        kind: Some(CompletionItemKind::KEYWORD),
        insert_text: Some(k.to_string()),
        ..Default::default()
    })
    .collect()
});

static TYPES_COMPLETIONS: LazyLock<Vec<CompletionItem>> = LazyLock::new(|| {
    vec![
        "int", "i8", "i16", "i32", "i64", "isize", "uint", "u8", "u16", "u32", "u64", "usize",
        "float", "f32", "f64", "bool", "string", "char", "void", "never", "any",
    ]
    .iter()
    .map(|t| CompletionItem {
        label: t.to_string(),
        kind: Some(CompletionItemKind::TYPE_PARAMETER),
        insert_text: Some(t.to_string()),
        ..Default::default()
    })
    .collect()
});

static BUILTIN_FUNCTIONS_COMPLETIONS: LazyLock<Vec<CompletionItem>> = LazyLock::new(|| {
    vec![
        ("print", "print($0)"),
        ("println", "println($0)"),
        ("dbg", "dbg!($0)"),
        ("panic", "panic!($0)"),
        ("assert", "assert!($0)"),
        ("assert_eq", "assert_eq!($0, $1)"),
        ("len", "len($0)"),
        ("push", "push($0)"),
        ("pop", "pop()"),
        ("insert", "insert($0, $1)"),
        ("remove", "remove($0)"),
        ("get", "get($0)"),
        ("parse", "parse::<$0>($1)"),
        ("to_string", "to_string()"),
        ("to_int", "to_int()"),
        ("to_float", "to_float()"),
    ]
    .iter()
    .map(|(name, insert)| CompletionItem {
        label: name.to_string(),
        kind: Some(CompletionItemKind::FUNCTION),
        insert_text: Some(insert.to_string()),
        insert_text_mode: Some(InsertTextMode::AS_IS),
        ..Default::default()
    })
    .collect()
});

/// Register completion handler with the server
pub fn register(server: &mut LspServer) {
    let workspace = server.workspace();
    server.register_request_handler::<Completion>(move |req| {
        let params: lsp_types::CompletionParams = serde_json::from_value(req.params)?;
        let uri = params.text_document_position.text_document.uri;
        let _position = params.text_document_position.position;

        let mut items = Vec::new();

        if let Some(doc) = workspace.get_document(&uri) {
            // Add cached static completions
            items.extend(KEYWORDS_COMPLETIONS.iter().cloned());
            items.extend(TYPES_COMPLETIONS.iter().cloned());
            items.extend(BUILTIN_FUNCTIONS_COMPLETIONS.iter().cloned());

            // Add symbols from current file
            items.extend(get_symbol_completions(&doc));
        }

        let result = CompletionList {
            is_incomplete: false,
            items,
        };

        let resp = lsp_server::Response::new_ok(req.id, result);
        Ok(resp)
    });
}

/// Get symbol completions from the current document
fn get_symbol_completions(doc: &crate::state::Document) -> Vec<CompletionItem> {
    let ast = match doc.ast() {
        Some(a) => a.as_ref(),
        None => return Vec::new(),
    };

    let mut items = Vec::new();

    for decl in &ast.declarations {
        match decl {
            x_parser::ast::Declaration::Function(func) => {
                let label_details = Some(CompletionItemLabelDetails {
                    detail: func.return_type.as_ref().map(|r| r.to_string()),
                    ..Default::default()
                });

                items.push(CompletionItem {
                    label: func.name.clone(),
                    kind: Some(CompletionItemKind::FUNCTION),
                    label_details,
                    insert_text: Some(create_function_snippet(&func.name, &func.parameters)),
                    insert_text_mode: Some(InsertTextMode::AS_IS),
                    ..Default::default()
                });
            }
            x_parser::ast::Declaration::Variable(var) => {
                if let Some(name) = var.simple_name() {
                    items.push(CompletionItem {
                        label: name.to_string(),
                        kind: Some(CompletionItemKind::VARIABLE),
                        detail: var.type_annot.as_ref().map(|t| t.to_string()),
                        insert_text: Some(name.to_string()),
                        ..Default::default()
                    });
                }
            }
            x_parser::ast::Declaration::Class(class) => {
                items.push(CompletionItem {
                    label: class.name.clone(),
                    kind: Some(CompletionItemKind::CLASS),
                    insert_text: Some(class.name.clone()),
                    ..Default::default()
                });
            }
            x_parser::ast::Declaration::Trait(trait_decl) => {
                items.push(CompletionItem {
                    label: trait_decl.name.clone(),
                    kind: Some(CompletionItemKind::INTERFACE),
                    insert_text: Some(trait_decl.name.clone()),
                    ..Default::default()
                });
            }
            x_parser::ast::Declaration::TypeAlias(alias) => {
                items.push(CompletionItem {
                    label: alias.name.clone(),
                    kind: Some(CompletionItemKind::TYPE_PARAMETER),
                    detail: Some(alias.type_.to_string()),
                    insert_text: Some(alias.name.clone()),
                    ..Default::default()
                });
            }
            _ => {}
        }
    }

    items
}

/// Create a function snippet with parameters
fn create_function_snippet(name: &str, params: &[x_parser::ast::Parameter]) -> String {
    let mut snippet = name.to_string();
    snippet.push('(');

    let param_snippets: Vec<String> = params
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let placeholder = format!("${{{}:{}}}", i + 1, p.name);
            if let Some(ty) = &p.type_annot {
                format!("{}: {}", placeholder, ty)
            } else {
                placeholder
            }
        })
        .collect();

    snippet.push_str(&param_snippets.join(", "));
    snippet.push(')');

    snippet
}
