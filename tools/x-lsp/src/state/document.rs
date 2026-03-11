//! Document state management

use std::sync::Arc;

use anyhow::Result;
use lsp_types::Url;
use x_parser::ast::Program;
use x_lexer::{token::Token, span::Span};
use x_parser::parse_program;

/// Represents an open text document in the workspace
#[derive(Debug, Clone)]
pub struct Document {
    /// Document URI
    uri: Url,
    /// Current content of the document
    content: String,
    /// Version number from LSP
    version: i32,
    /// Parsed tokens (cached)
    tokens: Option<Vec<(Token, Span)>>,
    /// Parsed AST (cached)
    ast: Option<Arc<Program>>,
    /// Last parse error message
    parse_error: Option<String>,
}

impl Document {
    /// Create a new document
    pub fn new(uri: Url, content: String, version: i32) -> Self {
        let mut doc = Self {
            uri,
            content,
            version,
            tokens: None,
            ast: None,
            parse_error: None,
        };

        // Parse immediately
        let _ = doc.parse();
        doc
    }

    /// Update document content
    pub fn update(&mut self, content: String, version: i32) {
        self.content = content;
        self.version = version;
        self.tokens = None;
        self.ast = None;
        self.parse_error = None;

        // Re-parse
        let _ = self.parse();
    }

    /// Parse the document content into tokens and AST
    pub fn parse(&mut self) -> Result<()> {
        // Lexical analysis
        let mut lexer = x_lexer::new_lexer(&self.content);
        let mut tokens = Vec::new();
        let mut lex_error = None;

        while let Some(result) = lexer.next() {
            match result {
                Ok((token, span)) => {
                    tokens.push((token, span));
                }
                Err(e) => {
                    lex_error = Some(e);
                    break;
                }
            }
        }

        if let Some(e) = lex_error {
            self.parse_error = Some(e.to_string());
            return Err(anyhow::anyhow!(e));
        }

        self.tokens = Some(tokens);

        // Parsing
        match parse_program(&self.content) {
            Ok(ast) => {
                self.ast = Some(Arc::new(ast));
                self.parse_error = None;
                Ok(())
            }
            Err(e) => {
                self.parse_error = Some(e.to_string());
                Err(anyhow::anyhow!(e))
            }
        }
    }

    /// Get document URI
    pub fn uri(&self) -> &Url {
        &self.uri
    }

    /// Get document content
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Get document version
    pub fn version(&self) -> i32 {
        self.version
    }

    /// Get parsed tokens (if available)
    pub fn tokens(&self) -> Option<&[(Token, Span)]> {
        self.tokens.as_deref()
    }

    /// Get parsed AST (if available)
    pub fn ast(&self) -> Option<&Arc<Program>> {
        self.ast.as_ref()
    }

    /// Get parse error message (if any)
    pub fn parse_error(&self) -> Option<&String> {
        self.parse_error.as_ref()
    }
}
