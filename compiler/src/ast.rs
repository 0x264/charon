#[derive(Debug)]
pub enum Expr {
    True,
    Flase,
    Null,
    This,
    Long(i64),
    Double(f64),
    String(String),
    Binary(BinaryExpr),
    Unary(UnaryExpr),
    Call(CallExpr),
    GetVar(String),
    Getter(GetterExpr)
}

#[derive(Debug)]
pub enum BinaryOp {
    Add, Sub, Multiply, Divide,
    Gt, Lt, EqEq, GtEq, LtEq, BangEq,
    And, Or
}

#[derive(Debug)]
pub struct BinaryExpr {
    pub left: Box<Expr>,
    pub op: BinaryOp,
    pub right: Box<Expr>
}

impl BinaryExpr {
    pub fn new(left: Box<Expr>, op: BinaryOp, right: Box<Expr>) -> Self {
        Self { left, op, right }
    }
}

#[derive(Debug)]
pub enum UnaryOp {
    Bang, Neg
}

#[derive(Debug)]
pub struct UnaryExpr {
    pub op: UnaryOp,
    pub expr: Box<Expr>
}

impl UnaryExpr {
    pub fn new(op: UnaryOp, expr: Box<Expr>) -> Self {
        Self { op, expr }
    }
}

#[derive(Debug)]
pub struct CallExpr {
    pub owner: Box<Expr>,
    pub args: Vec<Expr>
}

impl CallExpr {
    pub fn new(owner: Box<Expr>, args: Vec<Expr>) -> Self {
        Self { owner, args }
    }
}

#[derive(Debug)]
pub struct GetterExpr {
    pub owner: Box<Expr>,
    pub field: String
}

impl GetterExpr {
    pub fn new(owner: Box<Expr>, field: String) -> Self {
        Self { owner, field }
    }
}

#[derive(Debug)]
pub enum Stmt {
    VarDef(VarDefStmt),
    Expr(Box<Expr>),
    SetVar(SetVarStmt),
    Setter(SetterStmt),
    If(IfStmt),
    While(WhileStmt),
    Return(Option<Box<Expr>>),
    Block(Vec<Stmt>)
}

#[derive(Debug)]
pub struct VarDefStmt {
    pub name: String,
    pub init: Option<Box<Expr>>
}

impl VarDefStmt {
    pub fn new(name: String, init: Option<Box<Expr>>) -> Self {
        Self { name, init }
    }
}

#[derive(Debug)]
pub enum AssignOp {
    Assign, AddAssign, SubAssign, MultiplyAssign, DivideAssign
}

#[derive(Debug)]
pub struct SetVarStmt {
    pub to: String,
    pub op: AssignOp,
    pub value: Box<Expr>
}

impl SetVarStmt {
    pub fn new(to: String, op: AssignOp, value: Box<Expr>) -> Self {
        Self { to, op, value }
    }
}

#[derive(Debug)]
pub struct SetterStmt {
    pub owner: Box<Expr>,
    pub field: String,
    pub op: AssignOp,
    pub value: Box<Expr>
}

impl SetterStmt {
    pub fn new(owner: Box<Expr>, field: String, op: AssignOp, value: Box<Expr>) -> Self {
        Self { owner, field, op, value }
    }
}

#[derive(Debug)]
pub struct IfStmt {
    pub cond: Box<Expr>,
    pub then: Vec<Stmt>,
    pub els: Vec<Stmt>
}

impl IfStmt {
    pub fn new(cond: Box<Expr>, then: Vec<Stmt>, els: Vec<Stmt>) -> Self {
        Self { cond, then, els }
    }
}

#[derive(Debug)]
pub struct WhileStmt {
    pub cond: Box<Expr>,
    pub body: Vec<Stmt>
}

impl WhileStmt {
    pub fn new(cond: Box<Expr>, body: Vec<Stmt>) -> Self {
        Self { cond, body }
    }
}

#[derive(Debug)]
pub struct FuncDecl {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<Stmt>
}

impl FuncDecl {
    pub fn new(name: String, params: Vec<String>, body: Vec<Stmt>) -> Self {
        Self { name, params, body }
    }
}

#[derive(Debug)]
pub struct ClassDecl {
    pub name: String,
    pub methods: Vec<FuncDecl>
}

impl ClassDecl {
    pub fn new(name: String, methods: Vec<FuncDecl>) -> Self {
        Self { name, methods }
    }
}

#[derive(Debug)]
pub struct Program {
    pub funcs: Vec<FuncDecl>,
    pub classes: Vec<ClassDecl>
}

impl Program {
    pub fn new(funcs: Vec<FuncDecl>, classes: Vec<ClassDecl>) -> Self {
        Self { funcs, classes }
    }
}