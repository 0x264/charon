pub const OP_CONST_NULL: u8 = 1;
pub const OP_CONST_TRUE: u8 = 2;
pub const OP_CONST_FALSE: u8 = 3;

pub const OP_LCONST_M1: u8 = 4;
pub const OP_LCONST_0: u8 = 5;
pub const OP_LCONST_1: u8 = 6;
pub const OP_LCONST_2: u8 = 7;
pub const OP_LCONST_3: u8 = 8;
pub const OP_LCONST_4: u8 = 9;
pub const OP_LCONST_5: u8 = 10;

pub const OP_LDC: u8 = 11;

pub const OP_NEG: u8 = 12;

pub const OP_ADD: u8 = 13;

pub const OP_SUB: u8 = 14;
pub const OP_MUL: u8 = 15;
pub const OP_DIV: u8 = 16;

pub const OP_NOT: u8 = 17;

pub const OP_CMP_EQ: u8 = 18;
pub const OP_CMP_BANGEQ: u8 = 19;
pub const OP_CMP_GT: u8 = 20;
pub const OP_CMP_LT: u8 = 21;
pub const OP_CMP_GTEQ: u8 = 22;
pub const OP_CMP_LTEQ: u8 = 23;

pub const OP_IF: u8 = 24;
pub const OP_IF_NOT: u8 = 25;
pub const OP_GOTO: u8 = 26;

pub const OP_INVOKE: u8 = 27;

pub const OP_RETURN: u8 = 28;

pub const OP_POP: u8 = 29;

pub const OP_SET_GLOBAL: u8 = 30;

pub const OP_GET_GLOBAL: u8 = 31;

pub const OP_SET_LOCAL: u8 = 32;

pub const OP_GET_LOCAL: u8 = 33;

pub const OP_SET_FIELD: u8 = 34;

pub const OP_GET_MEMBER: u8 = 35;

pub const OP_DUP: u8 = 36;

pub const OP_DEF_GLOBAL: u8 = 37;