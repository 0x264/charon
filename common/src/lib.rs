pub mod opcode;
pub mod constant;
pub mod reader;
pub mod program;
pub mod loader;
pub mod line_column_info;

pub type Result<T> = std::result::Result<T, String>;