//! Abstract Syntax Tree for the Homun language.
//! Every construct maps directly to a Rust construct.

pub type Name = String;
pub type Program = Vec<Stmt>;

// ─────────────────────────────────────────
// Statements
// ─────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Stmt {
    Bind(Name, Expr),
    BindPat(Pat, Expr),
    /// Lvalue assignment: `expr[idx] := rhs` or `expr.field := rhs`
    Assign(Expr, Expr),
    Use(Vec<Name>),
    StructDef(Name, Vec<FieldDef>),
    EnumDef(Name, Vec<VariantDef>),
    Expression(Expr),
}

#[derive(Debug, Clone)]
pub struct FieldDef {
    pub name: Name,
    pub ty: Option<TypeExpr>,
}

#[derive(Debug, Clone)]
pub struct VariantDef {
    pub name: Name,
    pub payload: Option<TypeExpr>,
}

// ─────────────────────────────────────────
// Expressions
// ─────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Expr {
    // Literals
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    None,

    // Variables / field access
    Var(Name),
    Field(Box<Expr>, Name),
    Index(Box<Expr>, Box<Expr>),
    Slice(
        Box<Expr>,
        Option<Box<Expr>>,
        Option<Box<Expr>>,
        Option<Box<Expr>>,
    ),

    // Collections
    List(Vec<Expr>),
    Dict(Vec<(Expr, Expr)>),
    Set(Vec<Expr>),
    Tuple(Vec<Expr>),

    // Struct literal
    Struct(Option<Name>, Vec<(Name, Expr)>),

    // Operators
    BinOp(BinOp, Box<Expr>, Box<Expr>),
    UnOp(UnOp, Box<Expr>),

    // Pipe: lhs | f(args)
    Pipe(Box<Expr>, Box<Expr>),

    // Lambda
    Lambda {
        params: Vec<Param>,
        ret_ty: Option<TypeExpr>,
        void_mark: Option<TypeExpr>,
        stmts: Vec<Stmt>,
        final_expr: Box<Expr>,
    },

    // Application
    Call(Box<Expr>, Vec<Expr>),

    // Control flow as expressions
    If(
        Box<Expr>,
        Vec<Stmt>,
        Box<Expr>,
        Option<(Vec<Stmt>, Box<Expr>)>,
    ),
    Match(Box<Expr>, Vec<MatchArm>),

    // Loops
    For(Name, Box<Expr>, Vec<Stmt>, Option<Box<Expr>>),
    While(Box<Expr>, Vec<Stmt>, Option<Box<Expr>>),

    // Bare block
    Block(Vec<Stmt>, Box<Expr>),

    // Break / continue
    Break(Option<Box<Expr>>),
    Continue,

    // RON load/save
    LoadRon(Box<Expr>, TypeExpr),
    SaveRon(Box<Expr>, Box<Expr>),

    // Range
    Range(Option<Box<Expr>>, Option<Box<Expr>>, Option<Box<Expr>>),

    // Try-unwrap postfix `?`
    TryUnwrap(Box<Expr>),
}

// ─────────────────────────────────────────
// Match arm
// ─────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pat: Pat,
    pub guard: Option<Expr>,
    pub body: Expr,
}

#[derive(Debug, Clone)]
pub enum Pat {
    Wild,
    Var(Name),
    Lit(Expr),
    Tuple(Vec<Pat>),
    Enum(Name, Option<Box<Pat>>),
    None,
}

// ─────────────────────────────────────────
// Types
// ─────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum TypeExpr {
    Name(Name),
    List(Box<TypeExpr>),
    Dict(Box<TypeExpr>, Box<TypeExpr>),
    Set(Box<TypeExpr>),
    Option(Box<TypeExpr>),
    Tuple(Vec<TypeExpr>),
    Void,
    Infer,
}

// ─────────────────────────────────────────
// Operators
// ─────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Neq,
    Lt,
    Gt,
    Le,
    Ge,
    And,
    Or,
    In,
    NotIn,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnOp {
    Not,
    Neg,
}

// ─────────────────────────────────────────
// Lambda parameter
// ─────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Param {
    pub name: Name,
    pub ty: Option<TypeExpr>,
}
