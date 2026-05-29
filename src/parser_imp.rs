// parser_imp.rs — Rust helpers for parser.hom.
//
// Thread-local state (pos, err, gensym_counter) lives in parser.hom as
// @thread_local bindings; the generated _get/_set accessors are called here.
// PARSE_TOKENS stays in Rust because it needs a concrete Vec<Token> type.
//
// Public API groups:
//   Token inspection:  ps_peek_token, to_i64, to_f64
//   Token matching:    ps_same_line (ps_check/consume/expect/advance_ident in parser.hom)
//   AST constructors:  mk_expr_slice/lambda/for/while, mk_variantdef_multi/positional
//   Option helpers:    some_else, no_else

use std::cell::RefCell;

// ─── Thread-local token list (pos/err/gensym live in parser.hom) ────────────

thread_local! {
    static PARSE_TOKENS: RefCell<Vec<Token>> = const { RefCell::new(vec![]) };
}

fn has_err_internal() -> bool {
    !parse_err_get().is_empty()
}

fn peek_token_internal() -> Token {
    PARSE_TOKENS.with(|t| {
        let tokens = t.borrow();
        let pos = parse_pos_get() as usize;
        let idx = pos.min(tokens.len() - 1);
        tokens[idx].clone()
    })
}

fn advance_internal() {
    let pos = parse_pos_get() as usize;
    let len = PARSE_TOKENS.with(|t| t.borrow().len());
    if pos < len {
        parse_pos_set(parse_pos_get() + 1);
    }
}

// TokenKind has @derive(Debug) in lexer.hom — extract variant name from "{:?}"
fn token_kind_str(kind: &TokenKind) -> String {
    format!("{:?}", kind)
        .split('(')
        .next()
        .unwrap_or("Eof")
        .to_string()
}

// Thin Rust wrapper — sets parse_err back to empty string.
// Kept here instead of .hom because assigning "" to a @thread_local String
// requires String::new() (not a &str literal), which the current bootstrap
// codegen doesn't add automatically for the BindMut→set path.
pub fn ps_clear_err() {
    parse_err_set(String::new());
}

pub fn ps_peek_token() -> Token {
    peek_token_internal()
}

// ─── Numeric cast helpers (used by ps_peek_int/ps_peek_float in parser.hom) ──

pub fn to_i64(n: i32) -> i64 {
    n as i64
}

pub fn to_f64(f: f32) -> f64 {
    f as f64
}

// ─── Token matching ─────────────────────────────────────────────────────────

/// True if the current token is on the same line as the previous token.
pub fn ps_same_line() -> bool {
    if has_err_internal() {
        return false;
    }
    PARSE_TOKENS.with(|t| {
        let tokens = t.borrow();
        let pos = parse_pos_get() as usize;
        let cur = pos.min(tokens.len() - 1);
        cur > 0 && tokens[cur].pos.line == tokens[cur - 1].pos.line
    })
}

// ─── AST constructors: Expr ─────────────────────────────────────────────────

pub fn mk_expr_char_from_str(s: String) -> Expr {
    Expr::Char(s.chars().next().unwrap_or('\0'))
}
pub fn mk_expr_slice(
    base: Expr,
    from: Option<Expr>,
    to: Option<Expr>,
    step: Option<Expr>,
) -> Expr {
    Expr::Slice(
        Box::new(base),
        from.map(Box::new),
        to.map(Box::new),
        step.map(Box::new),
    )
}
pub fn mk_expr_lambda(
    params: Vec<Param>,
    ret_ty: Option<TypeExpr>,
    void_mark: Option<TypeExpr>,
    stmts: Vec<Stmt>,
    final_expr: Expr,
) -> Expr {
    Expr::Lambda {
        generics: vec![],
        params,
        ret_ty,
        void_mark,
        stmts,
        final_expr: Box::new(final_expr),
    }
}

pub fn mk_expr_lambda_generics(
    generics: Vec<String>,
    params: Vec<Param>,
    ret_ty: Option<TypeExpr>,
    void_mark: Option<TypeExpr>,
    stmts: Vec<Stmt>,
    final_expr: Expr,
) -> Expr {
    Expr::Lambda {
        generics,
        params,
        ret_ty,
        void_mark,
        stmts,
        final_expr: Box::new(final_expr),
    }
}
pub fn mk_expr_for(var: String, iter: Expr, stmts: Vec<Stmt>, final_expr: Option<Expr>) -> Expr {
    Expr::For(var, Box::new(iter), stmts, final_expr.map(Box::new))
}
pub fn mk_expr_while(cond: Expr, stmts: Vec<Stmt>, final_expr: Option<Expr>) -> Expr {
    Expr::While(Box::new(cond), stmts, final_expr.map(Box::new))
}

// ─── AST constructors: other ────────────────────────────────────────────────

pub fn mk_variantdef_multi(name: String, fnames: Vec<String>, ftys: Vec<TypeExpr>) -> VariantDef {
    let fields = fnames
        .into_iter()
        .zip(ftys)
        .map(|(n, ty)| {
            let opt_name = if n.is_empty() { None } else { Some(n) };
            (opt_name, ty)
        })
        .collect();
    VariantDef { name, fields }
}
pub fn mk_variantdef_positional(name: String, ftys: Vec<TypeExpr>) -> VariantDef {
    let fields = ftys.into_iter().map(|ty| (None, ty)).collect();
    VariantDef { name, fields }
}

// ─── Option helpers ─────────────────────────────────────────────────────────

/// Construct a Some else-clause for Expr::If.
pub fn some_else(stmts: Vec<Stmt>, expr: Expr) -> Option<(Vec<Stmt>, Box<Expr>)> {
    Some((stmts, Box::new(expr)))
}

/// Construct a None else-clause for Expr::If.
pub fn no_else() -> Option<(Vec<Stmt>, Box<Expr>)> {
    std::option::Option::None
}

// ─── @! inner-attribute helpers ─────────────────────────────────────────────

/// Peek at the kind of the token one position ahead (pos+1).
pub fn ps_peek_next_kind() -> String {
    if has_err_internal() {
        return "Eof".to_string();
    }
    PARSE_TOKENS.with(|t| {
        let tokens = t.borrow();
        let pos = parse_pos_get() as usize;
        let next_idx = (pos + 1).min(tokens.len() - 1);
        token_kind_str(&tokens[next_idx].kind)
    })
}

// ─── Public entry point ─────────────────────────────────────────────────────

/// Parse a token list into a Program (Vec<Stmt>).
/// This is the public API — called from main_imp.rs and resolver_imp.rs.
/// Calls parse_program() which is defined in the .hom-compiled code below.
pub fn parse(tokens: Vec<Token>) -> Result<Vec<Stmt>, String> {
    PARSE_TOKENS.with(|t| *t.borrow_mut() = tokens);
    parse_pos_set(0);
    parse_err_set(String::new());
    gensym_counter_set(0);
    let program = parse_program();
    let err = parse_err_get();
    if !err.is_empty() {
        Err(err)
    } else {
        Ok(program)
    }
}

