//! 后端共享工具模块
//!
//! 提供所有后端共享的实用工具，减少代码重复

pub mod buffer;
pub mod escape;
pub mod header;
pub mod operators;
pub mod symbols;

pub use buffer::CodeBuffer;
pub use escape::{escape_assembly_string, escape_string};
pub use header::{generate_header_with_version, headers, GENERATOR_NAME};
pub use operators::OperatorConfig;
pub use symbols::{SymbolInfo, SymbolScope, SymbolTable, SymbolType};
