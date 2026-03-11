//! Utility functions for LSP server

use lsp_types::{Position, Range};
use x_lexer::span::Span;

/// Convert a Span to LSP Range
pub fn span_to_range(span: &Span, content: &str) -> Range {
    let mut line = 0;
    let mut character = 0;
    let mut current_pos = 0;

    let content_chars: Vec<char> = content.chars().collect();

    // Find start position
    while current_pos < span.start as usize && current_pos < content_chars.len() {
        if content_chars[current_pos] == '\n' {
            line += 1;
            character = 0;
        } else {
            character += 1;
        }
        current_pos += 1;
    }

    let start = Position { line, character };

    // Find end position
    while current_pos < span.end as usize && current_pos < content_chars.len() {
        if content_chars[current_pos] == '\n' {
            break;
        }
        character += 1;
        current_pos += 1;
    }

    let end = Position { line, character };

    Range { start, end }
}

/// Convert LSP Position to byte offset in content
pub fn position_to_offset(pos: &Position, content: &str) -> usize {
    let mut offset = 0;
    let mut line = 0;

    for c in content.chars() {
        if line == pos.line as usize {
            break;
        }
        if c == '\n' {
            line += 1;
        }
        offset += c.len_utf8();
    }

    offset + pos.character as usize
}
