#[derive(Clone, Debug, PartialEq)]
pub enum TokenKind {
    // operators
    LParen,// (
    RParen,// )
    LBrace,// {
    RBrace,// }
    LBracket,// [
    RBracket,// ]
    Semi,// ;
    Comma,// ,
    Dot,// .
    Eq,// =
    Gt,// >
    Lt,// <
    Bang,// !
    EqEq,// ==
    GtEq,// >=
    LtEq,// <=
    BangEq,// !=
    AmpAmp,// &&
    BarBar,// ||
    Plus,// +
    Sub,// -
    Star,// *
    Slash,// /
    PlusEq,// +=
    SubEq,// -=
    StarEq,// *=
    SlashEq,// /=

    // keywords
    Var,
    True,
    False,
    If,
    Else,
    While,
    Break,
    Continue,
    Return,
    Func,
    Class,
    This,
    Null,

    Identifier(String),

    // literal
    Long(i64),
    Double(f64),
    String(String)
}

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub offset: usize
}

impl Token {
    pub fn new(kind: TokenKind, offset: usize) -> Self {
        Self {
            kind,
            offset
        }
    }
}