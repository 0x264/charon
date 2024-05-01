pub const MAGIC: &str = "charon-bytecode\0";

pub const CONSTANT_LONG: u8   = 0x1;
pub const CONSTANT_DOUBLE: u8 = 0x2;
pub const CONSTANT_STRING: u8 = 0x3;

pub const CURRENT_VERSION_MINOR: u8 = 0;
pub const CURRENT_VERSION_MAJOR: u8 = 1;

pub const ENTRY_NAME: &str = "$";

#[derive(Debug)]
pub enum ConstantItem {
    Long(i64),
    Double(f64),
    String(String)
}