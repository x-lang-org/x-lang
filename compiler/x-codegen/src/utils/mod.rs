//! 后端共享工具模块
//!
//! 提供所有后端共享的实用工具，减少代码重复

pub mod buffer;
pub mod escape;
pub mod operators;

pub use buffer::CodeBuffer;
pub use escape::escape_string;
pub use operators::OperatorConfig;
