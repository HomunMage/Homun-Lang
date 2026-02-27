/// Parser: recursive-descent Pratt parser → AST
use crate::ast::*;
use crate::lexer::{Token, TokenKind};

pub fn parse(tokens: Vec<Token>) -> Result<Program, String> {
    let mut p = Parser { tokens, pos: 0 };
    let prog = p.parse_program()?;
    Ok(prog)
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> &Token {
        &self.tokens[self.pos.min(self.tokens.len() - 1)]
    }

    fn advance(&mut self) -> &Token {
        let t = &self.tokens[self.pos.min(self.tokens.len() - 1)];
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        t
    }

    fn expect(&mut self, kind: &TokenKind) -> Result<(), String> {
        let t = self.advance();
        if std::mem::discriminant(&t.kind) == std::mem::discriminant(kind) || t.kind == *kind {
            Ok(())
        } else {
            Err(format!("Expected {:?} but got {:?}", kind, t.kind))
        }
    }

    fn check(&self, kind: &TokenKind) -> bool {
        self.peek().kind == *kind
    }

    fn consume(&mut self, kind: &TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn save(&self) -> usize {
        self.pos
    }

    fn restore(&mut self, pos: usize) {
        self.pos = pos;
    }

    // ─── Entry point ─────────────────────────────────────────────

    fn parse_program(&mut self) -> Result<Program, String> {
        let mut stmts = Vec::new();
        loop {
            match self.peek().kind {
                TokenKind::Eof => break,
                TokenKind::RBrace => break,
                TokenKind::Use => stmts.push(self.parse_use()?),
                TokenKind::Ident(_) => stmts.push(self.parse_top_bind()?),
                _ => break,
            }
        }
        self.expect(&TokenKind::Eof)?;
        Ok(stmts)
    }

    // ─── Top-level statements ────────────────────────────────────

    fn parse_use(&mut self) -> Result<Stmt, String> {
        self.expect(&TokenKind::Use)?;
        let path = self.parse_mod_path()?;
        Ok(Stmt::Use(path))
    }

    fn parse_mod_path(&mut self) -> Result<Vec<Name>, String> {
        let t = self.advance().clone();
        match &t.kind {
            TokenKind::Ident(n) => {
                let mut path = vec![n.clone()];
                if self.check(&TokenKind::Colon) {
                    self.advance();
                    if self.check(&TokenKind::Colon) {
                        self.advance();
                        let rest = self.parse_mod_path()?;
                        path.extend(rest);
                    }
                }
                Ok(path)
            }
            _ => Err("Expected module name".to_string()),
        }
    }

    fn parse_top_bind(&mut self) -> Result<Stmt, String> {
        let t = self.advance().clone();
        let name = match &t.kind {
            TokenKind::Ident(n) => n.clone(),
            _ => "_".to_string(),
        };
        self.expect(&TokenKind::Assign)?;
        match self.peek().kind {
            TokenKind::Struct => {
                self.advance();
                let fields = self.parse_brace_fields()?;
                Ok(Stmt::StructDef(name, fields))
            }
            TokenKind::Enum => {
                self.advance();
                let variants = self.parse_brace_variants()?;
                Ok(Stmt::EnumDef(name, variants))
            }
            _ => {
                let rhs = self.parse_expr()?;
                Ok(Stmt::Bind(name, rhs))
            }
        }
    }

    fn parse_brace_fields(&mut self) -> Result<Vec<FieldDef>, String> {
        self.expect(&TokenKind::LBrace)?;
        let mut fields = Vec::new();
        loop {
            match &self.peek().kind {
                TokenKind::RBrace => break,
                TokenKind::Ident(n) => {
                    let name = n.clone();
                    self.advance();
                    self.expect(&TokenKind::Colon)?;
                    let ty = self.parse_type_expr()?;
                    self.consume(&TokenKind::Comma);
                    fields.push(FieldDef { name, ty: Some(ty) });
                }
                _ => break,
            }
        }
        self.expect(&TokenKind::RBrace)?;
        Ok(fields)
    }

    fn parse_brace_variants(&mut self) -> Result<Vec<VariantDef>, String> {
        self.expect(&TokenKind::LBrace)?;
        let mut variants = Vec::new();
        loop {
            match &self.peek().kind {
                TokenKind::RBrace => break,
                TokenKind::Ident(n) => {
                    let name = n.clone();
                    self.advance();
                    let payload = if self.check(&TokenKind::LParen) {
                        self.advance();
                        let ty = self.parse_type_expr()?;
                        self.expect(&TokenKind::RParen)?;
                        Some(ty)
                    } else {
                        None
                    };
                    self.consume(&TokenKind::Comma);
                    variants.push(VariantDef { name, payload });
                }
                _ => break,
            }
        }
        self.expect(&TokenKind::RBrace)?;
        Ok(variants)
    }

    // ─── Block statements ────────────────────────────────────────

    fn parse_block_stmts(&mut self) -> Result<Vec<Stmt>, String> {
        let mut stmts = Vec::new();
        loop {
            match self.peek().kind {
                TokenKind::RBrace | TokenKind::Eof => break,
                TokenKind::Ident(_) => stmts.push(self.parse_block_bind()?),
                _ => {
                    let e = self.parse_expr()?;
                    stmts.push(Stmt::Expression(e));
                }
            }
        }
        Ok(stmts)
    }

    fn parse_block_bind(&mut self) -> Result<Stmt, String> {
        let saved = self.save();
        // Peek at first token — must be an ident to be a bind.
        let name = match self.peek().kind.clone() {
            TokenKind::Ident(n) => n,
            _ => {
                let e = self.parse_expr()?;
                return Ok(Stmt::Expression(e));
            }
        };
        self.advance(); // consume the ident

        // Check for tuple bind: a, b := expr
        if self.check(&TokenKind::Comma) {
            let mut names = vec![name];
            while self.consume(&TokenKind::Comma) {
                match self.peek().kind.clone() {
                    TokenKind::Ident(n) => {
                        self.advance();
                        names.push(n);
                    }
                    _ => {
                        self.restore(saved);
                        let e = self.parse_expr()?;
                        return Ok(Stmt::Expression(e));
                    }
                }
            }
            if self.check(&TokenKind::Assign) {
                self.advance();
                let rhs = self.parse_expr()?;
                let pats = names.into_iter().map(Pat::Var).collect();
                return Ok(Stmt::BindPat(Pat::Tuple(pats), rhs));
            }
            self.restore(saved);
            let e = self.parse_expr()?;
            return Ok(Stmt::Expression(e));
        }

        // Single name bind: a := expr
        if self.check(&TokenKind::Assign) {
            self.advance();
            let rhs = self.parse_expr()?;
            Ok(Stmt::Bind(name, rhs))
        } else {
            self.restore(saved);
            let e = self.parse_expr()?;
            Ok(Stmt::Expression(e))
        }
    }

    fn parse_block_stmts_braced(&mut self) -> Result<Vec<Stmt>, String> {
        self.expect(&TokenKind::LBrace)?;
        let stmts = self.parse_block_stmts()?;
        self.expect(&TokenKind::RBrace)?;
        Ok(stmts)
    }

    fn split_block(stmts: Vec<Stmt>) -> (Vec<Stmt>, Expr) {
        if stmts.is_empty() {
            return (vec![], Expr::Tuple(vec![]));
        }
        let mut stmts = stmts;
        match stmts.last() {
            Some(Stmt::Expression(_)) => {
                if let Stmt::Expression(e) = stmts.pop().unwrap() {
                    (stmts, e)
                } else {
                    unreachable!()
                }
            }
            _ => (stmts, Expr::Tuple(vec![])),
        }
    }

    // ─── Expressions ─────────────────────────────────────────────

    fn parse_expr(&mut self) -> Result<Expr, String> {
        self.parse_pipe()
    }

    fn parse_pipe(&mut self) -> Result<Expr, String> {
        let mut lhs = self.parse_or()?;
        while self.check(&TokenKind::Pipe) {
            self.advance();
            let rhs = self.parse_postfix()?;
            lhs = Expr::Pipe(Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    fn parse_or(&mut self) -> Result<Expr, String> {
        let mut lhs = self.parse_and()?;
        while self.check(&TokenKind::Or) {
            self.advance();
            let rhs = self.parse_and()?;
            lhs = Expr::BinOp(BinOp::Or, Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    fn parse_and(&mut self) -> Result<Expr, String> {
        let mut lhs = self.parse_not()?;
        while self.check(&TokenKind::And) {
            self.advance();
            let rhs = self.parse_not()?;
            lhs = Expr::BinOp(BinOp::And, Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    fn parse_not(&mut self) -> Result<Expr, String> {
        if self.check(&TokenKind::Not) {
            self.advance();
            let e = self.parse_not()?;
            Ok(Expr::UnOp(UnOp::Not, Box::new(e)))
        } else {
            self.parse_cmp()
        }
    }

    fn parse_cmp(&mut self) -> Result<Expr, String> {
        let lhs = self.parse_add_sub()?;
        let op = match &self.peek().kind {
            TokenKind::Eq => Some(BinOp::Eq),
            TokenKind::Neq => Some(BinOp::Neq),
            TokenKind::Lt => Some(BinOp::Lt),
            TokenKind::Gt => Some(BinOp::Gt),
            TokenKind::Le => Some(BinOp::Le),
            TokenKind::Ge => Some(BinOp::Ge),
            TokenKind::In => Some(BinOp::In),
            TokenKind::Not => {
                self.advance();
                self.expect(&TokenKind::In)?;
                let rhs = self.parse_add_sub()?;
                return Ok(Expr::BinOp(BinOp::NotIn, Box::new(lhs), Box::new(rhs)));
            }
            _ => None,
        };
        if let Some(op) = op {
            self.advance();
            let rhs = self.parse_add_sub()?;
            Ok(Expr::BinOp(op, Box::new(lhs), Box::new(rhs)))
        } else {
            Ok(lhs)
        }
    }

    fn parse_add_sub(&mut self) -> Result<Expr, String> {
        let mut lhs = self.parse_mul_div()?;
        loop {
            match self.peek().kind {
                TokenKind::Plus => {
                    self.advance();
                    let rhs = self.parse_mul_div()?;
                    lhs = Expr::BinOp(BinOp::Add, Box::new(lhs), Box::new(rhs));
                }
                TokenKind::Minus => {
                    self.advance();
                    let rhs = self.parse_mul_div()?;
                    lhs = Expr::BinOp(BinOp::Sub, Box::new(lhs), Box::new(rhs));
                }
                _ => break,
            }
        }
        Ok(lhs)
    }

    fn parse_mul_div(&mut self) -> Result<Expr, String> {
        let mut lhs = self.parse_unary()?;
        loop {
            match self.peek().kind {
                TokenKind::Star => {
                    self.advance();
                    let rhs = self.parse_unary()?;
                    lhs = Expr::BinOp(BinOp::Mul, Box::new(lhs), Box::new(rhs));
                }
                TokenKind::Slash => {
                    self.advance();
                    let rhs = self.parse_unary()?;
                    lhs = Expr::BinOp(BinOp::Div, Box::new(lhs), Box::new(rhs));
                }
                TokenKind::Percent => {
                    self.advance();
                    let rhs = self.parse_unary()?;
                    lhs = Expr::BinOp(BinOp::Mod, Box::new(lhs), Box::new(rhs));
                }
                _ => break,
            }
        }
        Ok(lhs)
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        if self.check(&TokenKind::Minus) {
            self.advance();
            let e = self.parse_unary()?;
            Ok(Expr::UnOp(UnOp::Neg, Box::new(e)))
        } else {
            self.parse_postfix()
        }
    }

    fn parse_postfix(&mut self) -> Result<Expr, String> {
        let mut e = self.parse_atom()?;
        loop {
            match self.peek().kind {
                TokenKind::Dot => {
                    self.advance();
                    let t = self.advance().clone();
                    match &t.kind {
                        TokenKind::Ident(n) => {
                            let name = n.clone();
                            if self.check(&TokenKind::LParen) {
                                let args = self.parse_arg_list()?;
                                e = Expr::Call(Box::new(Expr::Field(Box::new(e), name)), args);
                            } else {
                                e = Expr::Field(Box::new(e), name);
                            }
                        }
                        _ => return Err("Expected field name after '.'".to_string()),
                    }
                }
                TokenKind::LBracket => {
                    self.advance();
                    let result = self.parse_slice_or_index()?;
                    self.expect(&TokenKind::RBracket)?;
                    match result {
                        SliceOrIndex::Index(idx) => {
                            e = Expr::Index(Box::new(e), Box::new(idx));
                        }
                        SliceOrIndex::Slice(a, b, c) => {
                            e = Expr::Slice(
                                Box::new(e),
                                a.map(Box::new),
                                b.map(Box::new),
                                (*c).map(Box::new),
                            );
                        }
                    }
                }
                TokenKind::LParen => {
                    // Only treat as a function call if `(` is on the same line
                    // as the preceding token. Prevents match arm bodies from
                    // greedily consuming the next arm's tuple pattern as args.
                    let cur = self.pos.min(self.tokens.len() - 1);
                    if cur > 0 && self.tokens[cur].pos.line != self.tokens[cur - 1].pos.line {
                        break;
                    }
                    let args = self.parse_arg_list()?;
                    e = Expr::Call(Box::new(e), args);
                }
                TokenKind::Question => {
                    self.advance();
                    e = Expr::TryUnwrap(Box::new(e));
                }
                _ => break,
            }
        }
        Ok(e)
    }

    fn parse_slice_or_index(&mut self) -> Result<SliceOrIndex, String> {
        if self.check(&TokenKind::Colon) {
            self.advance();
            self.parse_slice_rest(None)
        } else {
            let e = self.parse_expr()?;
            if self.check(&TokenKind::Colon) {
                self.advance();
                self.parse_slice_rest(Some(e))
            } else {
                Ok(SliceOrIndex::Index(e))
            }
        }
    }

    fn parse_slice_rest(&mut self, start: Option<Expr>) -> Result<SliceOrIndex, String> {
        let end = self.parse_opt_slice()?;
        let step = if self.consume(&TokenKind::Colon) {
            self.parse_opt_slice()?
        } else {
            None
        };
        Ok(SliceOrIndex::Slice(start, end, Box::new(step)))
    }

    fn parse_opt_slice(&mut self) -> Result<Option<Expr>, String> {
        match self.peek().kind {
            TokenKind::RBracket | TokenKind::Colon => Ok(None),
            _ => Ok(Some(self.parse_expr()?)),
        }
    }

    // ─── Atoms ───────────────────────────────────────────────────

    fn parse_atom(&mut self) -> Result<Expr, String> {
        let kind = self.peek().kind.clone();
        match kind {
            TokenKind::Int(n) => {
                self.advance();
                Ok(Expr::Int(n))
            }
            TokenKind::Float(n) => {
                self.advance();
                Ok(Expr::Float(n))
            }
            TokenKind::Bool(b) => {
                self.advance();
                Ok(Expr::Bool(b))
            }
            TokenKind::Str(ref s) => {
                let s = s.clone();
                self.advance();
                Ok(Expr::Str(s))
            }
            TokenKind::None => {
                self.advance();
                Ok(Expr::None)
            }

            TokenKind::LParen => self.parse_lambda_or_paren(),

            TokenKind::At => self.parse_collection(),
            TokenKind::If => self.parse_if_expr(),
            TokenKind::Match => self.parse_match_expr(),
            TokenKind::For => self.parse_for_expr(),
            TokenKind::While => self.parse_while_expr(),
            TokenKind::Break => self.parse_break_expr(),
            TokenKind::Continue => {
                self.advance();
                Ok(Expr::Continue)
            }
            TokenKind::LBrace => self.parse_inline_block(),
            TokenKind::Underscore => {
                self.advance();
                Ok(Expr::Var("_".to_string()))
            }

            TokenKind::Ident(ref n) => {
                let name = n.clone();
                self.advance();
                if self.check(&TokenKind::LBrace) && is_upper_first(&name) {
                    let fields = self.parse_struct_lit_fields()?;
                    Ok(Expr::Struct(Some(name), fields))
                } else {
                    Ok(Expr::Var(name))
                }
            }

            _ => Err(format!("Unexpected token: {:?}", kind)),
        }
    }

    // ─── Lambda vs parenthesised expr ────────────────────────────

    fn parse_lambda_or_paren(&mut self) -> Result<Expr, String> {
        let saved = self.save();
        // Try lambda
        if let Ok(lambda) = self.try_parse_lambda() {
            return Ok(lambda);
        }
        self.restore(saved);

        // Parenthesised expr or tuple
        self.expect(&TokenKind::LParen)?;
        let e = self.parse_expr()?;
        if self.check(&TokenKind::Comma) {
            self.advance();
            let rest = self.parse_expr_sep(&TokenKind::RParen)?;
            self.expect(&TokenKind::RParen)?;
            let mut items = vec![e];
            items.extend(rest);
            Ok(Expr::Tuple(items))
        } else {
            self.expect(&TokenKind::RParen)?;
            Ok(e)
        }
    }

    fn try_parse_lambda(&mut self) -> Result<Expr, String> {
        self.expect(&TokenKind::LParen)?;
        let params = self.parse_param_list()?;
        self.expect(&TokenKind::RParen)?;
        // Must see ->
        if !self.check(&TokenKind::Arrow) {
            return Err("not a lambda".to_string());
        }
        self.advance();
        // Return type (optional)
        let (ret_ty, void_mark) = match self.peek().kind {
            TokenKind::LBrace => (None, None),
            TokenKind::Underscore => {
                self.advance();
                (None, Some(TypeExpr::Void))
            }
            _ => {
                let ty = self.parse_type_expr()?;
                (Some(ty), None)
            }
        };
        let stmts = self.parse_block_stmts_braced()?;
        let (ss, fe) = Self::split_block(stmts);
        Ok(Expr::Lambda {
            params,
            ret_ty,
            void_mark,
            stmts: ss,
            final_expr: Box::new(fe),
        })
    }

    fn parse_param_list(&mut self) -> Result<Vec<Param>, String> {
        if self.check(&TokenKind::RParen) {
            return Ok(vec![]);
        }
        let mut params = vec![self.parse_one_param()?];
        while self.consume(&TokenKind::Comma) {
            if self.check(&TokenKind::RParen) {
                break;
            }
            params.push(self.parse_one_param()?);
        }
        Ok(params)
    }

    fn parse_one_param(&mut self) -> Result<Param, String> {
        let t = self.advance().clone();
        match &t.kind {
            TokenKind::Ident(n) => {
                let name = n.clone();
                if self.consume(&TokenKind::Colon) {
                    let ty = self.parse_type_expr()?;
                    Ok(Param { name, ty: Some(ty) })
                } else {
                    Ok(Param { name, ty: None })
                }
            }
            TokenKind::Underscore => Ok(Param {
                name: "_".to_string(),
                ty: None,
            }),
            _ => Err(format!("Expected param name, got {:?}", t.kind)),
        }
    }

    fn parse_arg_list(&mut self) -> Result<Vec<Expr>, String> {
        self.expect(&TokenKind::LParen)?;
        let args = self.parse_expr_sep(&TokenKind::RParen)?;
        self.expect(&TokenKind::RParen)?;
        Ok(args)
    }

    fn parse_expr_sep(&mut self, stop: &TokenKind) -> Result<Vec<Expr>, String> {
        if self.check(stop) {
            return Ok(vec![]);
        }
        let mut items = vec![self.parse_expr()?];
        while self.consume(&TokenKind::Comma) {
            if self.check(stop) {
                break;
            }
            items.push(self.parse_expr()?);
        }
        Ok(items)
    }

    // ─── Collections ─────────────────────────────────────────────

    fn parse_collection(&mut self) -> Result<Expr, String> {
        self.expect(&TokenKind::At)?;
        match self.peek().kind {
            TokenKind::LBracket => self.parse_list(),
            TokenKind::LBrace => self.parse_dict(),
            TokenKind::LParen => self.parse_set(),
            _ => Err("Expected [, { or ( after @".to_string()),
        }
    }

    fn parse_list(&mut self) -> Result<Expr, String> {
        self.expect(&TokenKind::LBracket)?;
        if self.check(&TokenKind::RBracket) {
            self.advance();
            return Ok(Expr::List(vec![]));
        }
        let items = self.parse_expr_sep(&TokenKind::RBracket)?;
        self.expect(&TokenKind::RBracket)?;
        Ok(Expr::List(items))
    }

    fn parse_dict(&mut self) -> Result<Expr, String> {
        self.expect(&TokenKind::LBrace)?;
        if self.check(&TokenKind::RBrace) {
            self.advance();
            return Ok(Expr::Dict(vec![]));
        }
        let mut pairs = Vec::new();
        loop {
            let k = self.parse_expr()?;
            self.expect(&TokenKind::Colon)?;
            let v = self.parse_expr()?;
            pairs.push((k, v));
            if !self.consume(&TokenKind::Comma) {
                break;
            }
        }
        self.expect(&TokenKind::RBrace)?;
        Ok(Expr::Dict(pairs))
    }

    fn parse_set(&mut self) -> Result<Expr, String> {
        self.expect(&TokenKind::LParen)?;
        let items = self.parse_expr_sep(&TokenKind::RParen)?;
        self.expect(&TokenKind::RParen)?;
        Ok(Expr::Set(items))
    }

    fn parse_struct_lit_fields(&mut self) -> Result<Vec<(Name, Expr)>, String> {
        self.expect(&TokenKind::LBrace)?;
        let mut fields = Vec::new();
        loop {
            match &self.peek().kind {
                TokenKind::RBrace => break,
                TokenKind::Ident(n) => {
                    let name = n.clone();
                    self.advance();
                    self.expect(&TokenKind::Colon)?;
                    let v = self.parse_expr()?;
                    self.consume(&TokenKind::Comma);
                    fields.push((name, v));
                }
                _ => break,
            }
        }
        self.expect(&TokenKind::RBrace)?;
        Ok(fields)
    }

    // ─── Control flow ────────────────────────────────────────────

    fn parse_if_expr(&mut self) -> Result<Expr, String> {
        self.expect(&TokenKind::If)?;
        self.expect(&TokenKind::LParen)?;
        let cond = self.parse_expr()?;
        self.expect(&TokenKind::RParen)?;
        self.expect(&TokenKind::Do)?;
        let then_stmts = self.parse_block_stmts_braced()?;
        let (ts, te) = Self::split_block(then_stmts);
        let else_clause = if self.check(&TokenKind::Else) {
            self.advance();
            let else_stmts = self.parse_block_stmts_braced()?;
            let (es, ee) = Self::split_block(else_stmts);
            Some((es, Box::new(ee)))
        } else {
            None
        };
        Ok(Expr::If(Box::new(cond), ts, Box::new(te), else_clause))
    }

    fn parse_match_expr(&mut self) -> Result<Expr, String> {
        self.expect(&TokenKind::Match)?;
        let scrut = self.parse_expr()?;
        self.expect(&TokenKind::LBrace)?;
        let mut arms = Vec::new();
        while !self.check(&TokenKind::RBrace) {
            arms.push(self.parse_match_arm()?);
        }
        self.expect(&TokenKind::RBrace)?;
        Ok(Expr::Match(Box::new(scrut), arms))
    }

    fn parse_match_arm(&mut self) -> Result<MatchArm, String> {
        let p0 = self.parse_pat()?;
        let pat = if self.check(&TokenKind::Comma) {
            self.advance();
            let rest = self.parse_more_pats()?;
            let mut all = vec![p0];
            all.extend(rest);
            Pat::Tuple(all)
        } else {
            p0
        };
        let guard = if self.check(&TokenKind::If) {
            self.advance();
            Some(self.parse_expr()?)
        } else {
            None
        };
        self.expect(&TokenKind::FatArrow)?;
        let body = self.parse_expr()?;
        Ok(MatchArm { pat, guard, body })
    }

    fn parse_more_pats(&mut self) -> Result<Vec<Pat>, String> {
        let mut pats = vec![self.parse_pat()?];
        while self.check(&TokenKind::Comma) {
            self.advance();
            pats.push(self.parse_pat()?);
        }
        Ok(pats)
    }

    fn parse_pat(&mut self) -> Result<Pat, String> {
        let kind = self.peek().kind.clone();
        match kind {
            TokenKind::Underscore => {
                self.advance();
                Ok(Pat::Wild)
            }
            TokenKind::None => {
                self.advance();
                Ok(Pat::None)
            }
            TokenKind::Ident(ref n) => {
                let name = n.clone();
                self.advance();
                if self.check(&TokenKind::Dot) {
                    self.advance();
                    let t = self.advance().clone();
                    match &t.kind {
                        TokenKind::Ident(v) => {
                            let full = format!("{}.{}", name, v);
                            if self.check(&TokenKind::LParen) {
                                self.advance();
                                let p = self.parse_pat()?;
                                self.expect(&TokenKind::RParen)?;
                                Ok(Pat::Enum(full, Some(Box::new(p))))
                            } else {
                                Ok(Pat::Enum(full, None))
                            }
                        }
                        _ => Err("Expected variant name after '.'".to_string()),
                    }
                } else if self.check(&TokenKind::LParen) {
                    // Constructor pattern: Ok(x), Err(msg), Some(x) — same-line only
                    let cur = self.pos.min(self.tokens.len() - 1);
                    if cur > 0 && self.tokens[cur].pos.line == self.tokens[cur - 1].pos.line {
                        self.advance(); // consume (
                        let p = self.parse_pat()?;
                        self.expect(&TokenKind::RParen)?;
                        Ok(Pat::Enum(name, Some(Box::new(p))))
                    } else {
                        Ok(Pat::Var(name))
                    }
                } else {
                    Ok(Pat::Var(name))
                }
            }
            TokenKind::Int(n) => {
                self.advance();
                Ok(Pat::Lit(Expr::Int(n)))
            }
            TokenKind::Float(n) => {
                self.advance();
                Ok(Pat::Lit(Expr::Float(n)))
            }
            TokenKind::Bool(b) => {
                self.advance();
                Ok(Pat::Lit(Expr::Bool(b)))
            }
            TokenKind::Str(ref s) => {
                let s = s.clone();
                self.advance();
                Ok(Pat::Lit(Expr::Str(s)))
            }
            TokenKind::Minus => {
                self.advance();
                match self.peek().kind.clone() {
                    TokenKind::Int(n) => {
                        self.advance();
                        Ok(Pat::Lit(Expr::Int(-n)))
                    }
                    TokenKind::Float(n) => {
                        self.advance();
                        Ok(Pat::Lit(Expr::Float(-n)))
                    }
                    _ => Err("Expected number after '-' in pattern".to_string()),
                }
            }
            TokenKind::LParen => {
                self.advance(); // consume (
                if self.check(&TokenKind::RParen) {
                    self.advance();
                    return Ok(Pat::Tuple(vec![]));
                }
                let mut pats = vec![self.parse_pat()?];
                while self.consume(&TokenKind::Comma) {
                    if self.check(&TokenKind::RParen) {
                        break;
                    }
                    pats.push(self.parse_pat()?);
                }
                self.expect(&TokenKind::RParen)?;
                Ok(Pat::Tuple(pats))
            }
            _ => Err(format!("Expected pattern, got {:?}", kind)),
        }
    }

    fn parse_for_expr(&mut self) -> Result<Expr, String> {
        self.expect(&TokenKind::For)?;
        let t = self.advance().clone();
        let var_name = match &t.kind {
            TokenKind::Ident(n) => n.clone(),
            TokenKind::Underscore => "_".to_string(),
            _ => return Err("Expected loop variable".to_string()),
        };
        self.expect(&TokenKind::In)?;
        let iter = self.parse_expr()?;
        self.expect(&TokenKind::Do)?;
        let stmts = self.parse_block_stmts_braced()?;
        let (ss, fe) = Self::split_block(stmts);
        Ok(Expr::For(var_name, Box::new(iter), ss, Some(Box::new(fe))))
    }

    fn parse_while_expr(&mut self) -> Result<Expr, String> {
        self.expect(&TokenKind::While)?;
        self.expect(&TokenKind::LParen)?;
        let cond = self.parse_expr()?;
        self.expect(&TokenKind::RParen)?;
        self.expect(&TokenKind::Do)?;
        let stmts = self.parse_block_stmts_braced()?;
        let (ss, fe) = Self::split_block(stmts);
        Ok(Expr::While(Box::new(cond), ss, Some(Box::new(fe))))
    }

    fn parse_break_expr(&mut self) -> Result<Expr, String> {
        self.expect(&TokenKind::Break)?;
        if self.check(&TokenKind::FatArrow) {
            self.advance();
            let e = self.parse_expr()?;
            Ok(Expr::Break(Some(Box::new(e))))
        } else {
            Ok(Expr::Break(None))
        }
    }

    fn parse_inline_block(&mut self) -> Result<Expr, String> {
        let stmts = self.parse_block_stmts_braced()?;
        let (ss, fe) = Self::split_block(stmts);
        Ok(Expr::Block(ss, Box::new(fe)))
    }

    // ─── Types ───────────────────────────────────────────────────

    fn parse_type_expr(&mut self) -> Result<TypeExpr, String> {
        match self.peek().kind {
            TokenKind::At => {
                self.advance();
                match self.peek().kind {
                    TokenKind::LBracket => {
                        self.advance();
                        let ty = self.parse_type_expr()?;
                        self.expect(&TokenKind::RBracket)?;
                        Ok(TypeExpr::List(Box::new(ty)))
                    }
                    TokenKind::LBrace => {
                        self.advance();
                        let k = self.parse_type_expr()?;
                        self.expect(&TokenKind::Colon)?;
                        let v = self.parse_type_expr()?;
                        self.expect(&TokenKind::RBrace)?;
                        Ok(TypeExpr::Dict(Box::new(k), Box::new(v)))
                    }
                    TokenKind::LParen => {
                        self.advance();
                        let ty = self.parse_type_expr()?;
                        self.expect(&TokenKind::RParen)?;
                        Ok(TypeExpr::Set(Box::new(ty)))
                    }
                    _ => Ok(TypeExpr::Infer),
                }
            }
            TokenKind::Underscore => {
                self.advance();
                Ok(TypeExpr::Void)
            }
            TokenKind::Ident(ref n) => {
                let name = n.clone();
                self.advance();
                Ok(TypeExpr::Name(name))
            }
            _ => Ok(TypeExpr::Infer),
        }
    }
}

// ─── Utilities ───────────────────────────────────────────────

enum SliceOrIndex {
    Index(Expr),
    Slice(Option<Expr>, Option<Expr>, Box<Option<Expr>>),
}

fn is_upper_first(s: &str) -> bool {
    s.chars().next().is_some_and(|c| c.is_uppercase())
}
