//! Document state management

use std::sync::Arc;

use anyhow::Result;
use lsp_types::Url;
use x_parser::ast::Program;
use x_lexer::{token::Token, span::Span};
use x_parser::parse_program;
use x_typechecker::TypeError;

/// Type check error with span information
#[derive(Debug, Clone)]
pub struct TypeCheckError {
    /// Error message
    pub message: String,
    /// Span of the error
    pub span: Option<Span>,
}

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
    /// Type check errors
    type_errors: Vec<TypeCheckError>,
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
            type_errors: Vec::new(),
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
        self.type_errors.clear();

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
                self.ast = Some(Arc::new(ast.clone()));
                self.parse_error = None;

                // Type checking
                self.type_errors.clear();
                if let Err(type_err) = x_typechecker::type_check(&ast) {
                    self.type_errors.push(TypeCheckError {
                        message: type_err.to_string(),
                        span: Self::extract_span_from_type_error(&type_err),
                    });
                }

                Ok(())
            }
            Err(e) => {
                self.parse_error = Some(e.to_string());
                Err(anyhow::anyhow!(e))
            }
        }
    }

    /// Extract span from a type error if available
    fn extract_span_from_type_error(err: &TypeError) -> Option<Span> {
        match err {
            TypeError::UndefinedVariable { span, .. } => Some(*span),
            TypeError::UndefinedType { span, .. } => Some(*span),
            TypeError::TypeMismatch { span, .. } => Some(*span),
            TypeError::ArgumentCountMismatch { span, .. } => Some(*span),
            TypeError::ReturnTypeMismatch { span, .. } => Some(*span),
            TypeError::DuplicateDeclaration { span, .. } => Some(*span),
            TypeError::InvalidTypeAnnotation { span } => Some(*span),
            TypeError::TypeParameterMismatch { span } => Some(*span),
            TypeError::TypeConstraintViolation { span } => Some(*span),
            TypeError::RecursiveType { span } => Some(*span),
            TypeError::CannotInferType { span } => Some(*span),
            TypeError::NotImplemented { span, .. } => Some(*span),
            TypeError::InternalError { span, .. } => Some(*span),
            TypeError::ParameterCountMismatch { span, .. } => Some(*span),
            TypeError::ParameterTypeMismatch { span } => Some(*span),
            TypeError::TypeIncompatible { span } => Some(*span),
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

    /// Get type check errors
    pub fn type_errors(&self) -> &[TypeCheckError] {
        &self.type_errors
    }
}
