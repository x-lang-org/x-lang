//! Symbol table and symbol information

use lsp_types::Url;
use x_lexer::span::Span;

/// Type of symbol
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    /// Function definition
    Function,
    /// Variable definition
    Variable,
    /// Type definition
    Type,
    /// Struct/record field
    Field,
    /// Module/namespace
    Module,
    /// Parameter
    Parameter,
    /// Constant
    Constant,
}

/// Symbol information
#[derive(Debug, Clone)]
pub struct Symbol {
    /// Name of the symbol
    pub name: String,
    /// Kind of symbol
    pub kind: SymbolKind,
    /// URI of the file where the symbol is defined
    pub uri: Url,
    /// Span of the symbol definition
    pub span: Span,
    /// Type of the symbol (as string for display)
    pub type_string: Option<String>,
    /// Documentation for the symbol
    pub documentation: Option<String>,
}

/// Symbol table for a document or workspace
#[derive(Debug, Default)]
pub struct SymbolTable {
    /// Symbols keyed by name
    symbols: Vec<Symbol>,
}

impl SymbolTable {
    /// Create a new empty symbol table
    pub fn new() -> Self {
        Self {
            symbols: Vec::new(),
        }
    }

    /// Add a symbol to the table
    pub fn add(&mut self, symbol: Symbol) {
        self.symbols.push(symbol);
    }

    /// Find a symbol by name
    pub fn find_by_name(&self, name: &str) -> Vec<&Symbol> {
        self.symbols.iter().filter(|s| s.name == name).collect()
    }

    /// Find a symbol at the given position
    pub fn find_at_position(&self, line: u32, character: u32, content: &str) -> Option<&Symbol> {
        // TODO: Implement position lookup
        None
    }

    /// Get all symbols in the table
    pub fn all_symbols(&self) -> &[Symbol] {
        &self.symbols
    }

    /// Clear all symbols
    pub fn clear(&mut self) {
        self.symbols.clear();
    }
}
