use std::mem;
use crate::ast::*;
use crate::token::{Token, TokenKind};
use crate::err::{Result, Error};

pub struct Parser {
    tokens: Vec<Token>,
    offset: usize
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            offset: 0
        }
    }

    pub fn parse(mut self) -> Result<Program> {
        let mut funcs = Vec::new();
        let mut classes = Vec::new();
        let mut stmts = Vec::new();
        
        while let Some(tok) = self.peek() {
            match tok.kind {
                TokenKind::Func => {
                    self.advance();
                    funcs.push(self.parse_function()?);
                }
                TokenKind::Class => {
                    self.advance();
                    classes.push(self.parse_class()?);
                }
                _ => stmts.push(self.parse_stmt()?)
            }
        }
        
        let entry = FuncDecl::new("$".to_owned(), Vec::new(), stmts);
        funcs.push(entry);
        Ok(Program::new(funcs, classes))
    }
    
    fn parse_function(&mut self) -> Result<FuncDecl> {
        let Some(Token {kind: TokenKind::Identifier(name), offset: _ }) = self.next() else {
            return Err(Error::new("function name not found after keyword `func`".to_owned(), self.offset));
        };
        
        let name = name.to_owned();
        
        self.consume_or_err(&TokenKind::LParen)?;
        
        let mut params = Vec::new();
        
        loop {
            let Some(Token {kind: TokenKind::Identifier(name), offset: _}) = self.peek() else {
                break;
            };
            params.push(name.to_owned());
            self.advance();
            
            if !self.consume(&TokenKind::Comma) {
                break;
            }
        }
        
        self.consume_or_err(&TokenKind::RParen)?;
        
        self.consume_or_err(&TokenKind::LBrace)?;
        Ok(FuncDecl::new(name, params, self.parse_block()?))
    }

    fn parse_class(&mut self) -> Result<ClassDecl> {
        let Some(Token {kind: TokenKind::Identifier(name), offset: _ }) = self.next() else {
            return Err(Error::new("class name not found after keyword `class`".to_owned(), self.offset));
        };
        let name = name.to_owned();
        self.consume_or_err(&TokenKind::LBrace)?;
        
        let mut methods = Vec::new();
        while let Some(Token {kind: TokenKind::Func, offset: _}) = self.peek() {
            self.advance();
            methods.push(self.parse_function()?);
        }
        self.consume_or_err(&TokenKind::RBrace)?;
        Ok(ClassDecl::new(name, methods))
    }
    
    fn parse_stmt(&mut self) -> Result<Stmt> {
        let Some(tok) = self.next() else {
            return Err(Error::new("unexpected end of file".to_owned(), self.offset));
        };
        
        let stmt = match &tok.kind {
            TokenKind::Var => Stmt::VarDef(self.parse_var_def()?),
            TokenKind::If => Stmt::If(self.parse_if()?),
            TokenKind::While => Stmt::While(self.parse_while()?),
            TokenKind::Return => Stmt::Return(self.parse_return()?),
            TokenKind::LBrace => Stmt::Block(self.parse_block()?),
            _ => {
                self.offset -= 1;
                self.parse_assign_or_expr_stmt()?
            }
        };
        Ok(stmt)
    }
    
    fn parse_var_def(&mut self) -> Result<VarDefStmt> {
        let Some(Token {kind: TokenKind::Identifier(name), offset: _}) = self.next() else {
            return Err(Error::new("expected variable name after keyword `var`".to_owned(), self.offset));
        };
        let name = name.to_owned();
        
        let stmt = if self.consume(&TokenKind::Eq) {
            VarDefStmt::new(name, Some(Box::new(self.parse_expr()?)))
        } else {
            VarDefStmt::new(name, None)
        };
        self.consume_or_err(&TokenKind::Semi)?;
        Ok(stmt)
    }
    
    fn parse_if(&mut self) -> Result<IfStmt> {
        self.consume_or_err(&TokenKind::LParen)?;
        let cond = Box::new(self.parse_expr()?);
        self.consume_or_err(&TokenKind::RParen)?;
        let then = self.parse_block_with_lbrace()?;
        let stmt = if self.consume(&TokenKind::Else) {
            let els = if self.consume(&TokenKind::If) {
                vec![Stmt::If(self.parse_if()?)]
            } else {
                self.parse_block_with_lbrace()?
            };
            IfStmt::new(cond, then, els)
        } else {
            IfStmt::new(cond, then, Vec::new())
        };
        Ok(stmt)
    }
    
    fn parse_while(&mut self) -> Result<WhileStmt> {
        self.consume_or_err(&TokenKind::LParen)?;
        let cond = Box::new(self.parse_expr()?);
        self.consume_or_err(&TokenKind::RParen)?;
        let body = self.parse_block_with_lbrace()?;
        Ok(WhileStmt::new(cond, body))
    }
    
    fn parse_return(&mut self) -> Result<Option<Box<Expr>>> {
        if self.consume(&TokenKind::Semi) {
            return Ok(None);
        }
        
        let res = self.parse_expr()?;
        self.consume_or_err(&TokenKind::Semi)?;
        Ok(Some(Box::new(res)))
    }
    
    fn parse_assign_or_expr_stmt(&mut self) -> Result<Stmt> {
        let left = self.parse_expr()?;
        let Some(tok) = self.next() else {
            return Err(Error::new("unexpected end after expr in stmt".to_owned(), self.offset));
        };
        
        let op = match &tok.kind {
            TokenKind::Semi => return Ok(Stmt::Expr(Box::new(left))),
            TokenKind::Eq => AssignOp::Assign,
            TokenKind::PlusEq => AssignOp::AddAssign,
            TokenKind::SubEq => AssignOp::SubAssign,
            TokenKind::StarEq => AssignOp::MultiplyAssign,
            TokenKind::SlashEq => AssignOp::DivideAssign,
            _ => return Err(Error::new(format!("unexpected token: {tok:?}"), self.offset))
        };
        
        if !matches!(left, Expr::Getter(_) | Expr::GetVar(_)) {
            return Err(Error::new("invalid assign target".to_owned(), self.offset));
        }
        
        let value = Box::new(self.parse_expr()?);
        let stmt = match left {
            Expr::GetVar(var) => Stmt::SetVar(SetVarStmt::new(var, op, value)),
            Expr::Getter(getter) => Stmt::Setter(SetterStmt::new(getter.owner, getter.field, op, value)),
            _ => unreachable!()
        };
        self.consume_or_err(&TokenKind::Semi)?;
        Ok(stmt)
    }
    
    fn parse_expr(&mut self) -> Result<Expr> {
        self.logic_or()
    }
    
    fn logic_or(&mut self) -> Result<Expr> {
        let mut left = self.logic_and()?;
        while self.consume(&TokenKind::BarBar) {
            let right = self.logic_and()?;
            left = Expr::Logic(LogicExpr::new(Box::new(left), LogicOp::Or, Box::new(right)));
        }
        Ok(left)
    }
    
    fn logic_and(&mut self) -> Result<Expr> {
        let mut left = self.equal()?;
        while self.consume(&TokenKind::AmpAmp) {
            let right = self.equal()?;
            left = Expr::Logic(LogicExpr::new(Box::new(left), LogicOp::And, Box::new(right)));
        }
        Ok(left)
    }
    
    fn equal(&mut self) -> Result<Expr> {
        let left = self.compare()?;
        if self.consume(&TokenKind::EqEq) {
            Ok(Expr::Binary(BinaryExpr::new(Box::new(left), BinaryOp::EqEq, Box::new(self.compare()?))))
        } else if self.consume(&TokenKind::BangEq) {
            Ok(Expr::Binary(BinaryExpr::new(Box::new(left), BinaryOp::BangEq, Box::new(self.compare()?))))
        } else {
            Ok(left)
        }
    }
    
    fn compare(&mut self) -> Result<Expr> {
        let expr = self.add_sub()?;
        let Some(tok) = self.next() else {
            return Ok(expr);
        };
        let op = match &tok.kind {
            TokenKind::Gt => BinaryOp::Gt,
            TokenKind::Lt => BinaryOp::Lt,
            TokenKind::GtEq => BinaryOp::GtEq,
            TokenKind::LtEq => BinaryOp::LtEq,
            _ => {
                self.offset -= 1;
                return Ok(expr);
            }
        };
        Ok(Expr::Binary(BinaryExpr::new(Box::new(expr), op, Box::new(self.add_sub()?))))
    }
    
    fn add_sub(&mut self) -> Result<Expr> {
        let mut left = self.multiply_divide()?;
        loop {
            if self.consume(&TokenKind::Plus) {
                left = Expr::Binary(BinaryExpr::new(Box::new(left), BinaryOp::Add, Box::new(self.multiply_divide()?)));
            } else if self.consume(&TokenKind::Sub) {
                left = Expr::Binary(BinaryExpr::new(Box::new(left), BinaryOp::Sub, Box::new(self.multiply_divide()?)));
            } else {
                break;
            }
        }
        Ok(left)
    }
    
    fn multiply_divide(&mut self) -> Result<Expr> {
        let mut left = self.unary()?;
        loop {
            if self.consume(&TokenKind::Star) {
                left = Expr::Binary(BinaryExpr::new(Box::new(left), BinaryOp::Multiply, Box::new(self.unary()?)));
            } else if self.consume(&TokenKind::Slash) {
                left = Expr::Binary(BinaryExpr::new(Box::new(left), BinaryOp::Divide, Box::new(self.unary()?)));
            } else {
                break;
            }
        }
        Ok(left)
    }
    
    fn unary(&mut self) -> Result<Expr> {
        if self.consume(&TokenKind::Bang) {
            Ok(Expr::Unary(UnaryExpr::new(UnaryOp::Bang, Box::new(self.unary()?))))
        } else if self.consume(&TokenKind::Sub) {
            Ok(Expr::Unary(UnaryExpr::new(UnaryOp::Neg, Box::new(self.unary()?))))
        } else {
            self.call()
        }
    }
    
    fn call(&mut self) -> Result<Expr> {
        let mut p = self.primary()?;
        loop {
            if self.consume(&TokenKind::LParen) {
                let mut args = Vec::new();
                loop {
                    if self.consume(&TokenKind::RParen) {
                        break;
                    }
                    args.push(self.parse_expr()?);
                    if !self.consume(&TokenKind::Comma) {
                        self.consume_or_err(&TokenKind::RParen)?;
                        break;
                    }
                }
                p = Expr::Call(CallExpr::new(Box::new(p), args));
            } else if self.consume(&TokenKind::Dot) {
                let Some(Token {kind: TokenKind::Identifier(name), offset: _}) = self.next() else {
                    return Err(Error::new("expected identifier".to_owned(), self.offset));
                };
                p = Expr::Getter(GetterExpr::new(Box::new(p), name.to_owned()));
            } else {
                break;
            }
        }
        Ok(p)
    }
    
    fn primary(&mut self) -> Result<Expr> {
        let Some(tok) = self.next() else {
            return Err(Error::new("unexpected end of file".to_owned(), self.offset));
        };
        
        let expr = match &tok.kind {
            TokenKind::Long(v) => Expr::Long(*v),
            TokenKind::Double(v) => Expr::Double(*v),
            TokenKind::String(v) => Expr::String(v.to_owned()),
            TokenKind::True => Expr::True,
            TokenKind::False => Expr::Flase,
            TokenKind::This => Expr::This,
            TokenKind::Null => Expr::Null,
            TokenKind::Identifier(var) => Expr::GetVar(var.to_owned()),
            TokenKind::LParen => {
                let e = self.parse_expr()?;
                self.consume_or_err(&TokenKind::RParen)?;
                e
            }
            _ => return Err(Error::new(format!("unexpected token: {tok:?} in primary stmt"), tok.offset))
        };
        Ok(expr)
    }
    
    fn parse_block_with_lbrace(&mut self) -> Result<Vec<Stmt>> {
        self.consume_or_err(&TokenKind::LBrace)?;
        self.parse_block()
    }
    
    fn parse_block(&mut self) -> Result<Vec<Stmt>> {
        let mut stmts = Vec::new();
        loop {
            if self.consume(&TokenKind::RBrace) {
                break;
            }
            stmts.push(self.parse_stmt()?);
        }
        Ok(stmts)
    }

    fn consume(&mut self, tok: &TokenKind) -> bool {
        if let Some(t) = self.peek()
            && mem::discriminant(&t.kind) == mem::discriminant(tok) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn consume_or_err(&mut self, tok: &TokenKind) -> Result<()> {
        if let Some(t) = self.peek() {
            if mem::discriminant(&t.kind) == mem::discriminant(tok) {
                self.advance();
                Ok(())
            } else {
                Err(Error::new(format!("expected token: {tok:?}, got: {:?}", t.kind), t.offset))
            }
        } else {
            Err(Error::new(format!("failed to consume token: {tok:?}, end of file"), self.offset))
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.offset)
    }

    fn next(&mut self) -> Option<&Token> {
        self.tokens.get(self.offset).map(|t| {
            self.offset += 1;
            t
        })
    }

    fn advance(&mut self) {
        self.offset += 1;
    }
}