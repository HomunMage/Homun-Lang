/// Code generator: walks the Homun AST and emits Rust source text.
use crate::ast::*;
use std::collections::{HashMap, HashSet};

type Indent = usize;
type Scope = HashSet<Name>;

fn ind(n: Indent) -> String {
    " ".repeat(n * 4)
}

// ─── Entry point ─────────────────────────────────────────────

pub fn codegen_program_with_resolved(
    prog: &Program,
    resolved_hom_files: &HashSet<String>,
    resolved_rs_content: &HashMap<String, String>,
) -> String {
    prog.iter()
        .filter_map(|s| {
            // Skip use statements for .hom files that were resolved and inlined.
            if let Stmt::Use(path) = s {
                if path.len() == 1 && resolved_hom_files.contains(&path[0]) {
                    return None;
                }
            }
            Some(codegen_top_level(0, s, resolved_rs_content))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn codegen_top_level(i: Indent, stmt: &Stmt, rs_content: &HashMap<String, String>) -> String {
    match stmt {
        Stmt::Use(path) if path.len() == 1 => {
            if let Some(content) = rs_content.get(&path[0]) {
                format!("// ── use {} ──\n{}", path[0], content)
            } else {
                format!("use {};", path.join("::"))
            }
        }
        Stmt::Use(path) => {
            format!("use {};", path.join("::"))
        }
        Stmt::StructDef(name, fields) => {
            let mut lines = vec![
                "#[derive(Debug, Clone)]".to_string(),
                format!("pub struct {} {{", name),
            ];
            for f in fields {
                lines.push(codegen_field(i + 1, f));
            }
            lines.push("}".to_string());
            lines.join("\n")
        }
        Stmt::EnumDef(name, variants) => {
            let mut lines = vec![
                "#[derive(Debug, Clone)]".to_string(),
                format!("pub enum {} {{", name),
            ];
            for v in variants {
                lines.push(codegen_variant(i + 1, v));
            }
            lines.push("}".to_string());
            lines.join("\n")
        }
        Stmt::Bind(
            name,
            Expr::Lambda {
                params,
                ret_ty,
                void_mark,
                stmts,
                final_expr,
            },
        ) => codegen_fn(
            i,
            name,
            params,
            ret_ty.as_ref(),
            void_mark.as_ref(),
            stmts,
            final_expr,
        ),
        Stmt::Bind(name, expr) => {
            format!(
                "pub const {}: _ = {};",
                to_upper(name),
                cg_expr(i, &HashSet::new(), expr)
            )
        }
        Stmt::Expression(e) => {
            format!("{};", cg_expr(i, &HashSet::new(), e))
        }
    }
}

fn codegen_field(i: Indent, f: &FieldDef) -> String {
    let ty = f.ty.as_ref().map_or("_".to_string(), codegen_type);
    format!("{}pub {}: {},", ind(i), f.name, ty)
}

fn codegen_variant(i: Indent, v: &VariantDef) -> String {
    match &v.payload {
        None => format!("{}{},", ind(i), v.name),
        Some(ty) => format!("{}{}({}),", ind(i), v.name, codegen_type(ty)),
    }
}

// ─── Functions ───────────────────────────────────────────────

fn codegen_fn(
    i: Indent,
    name: &str,
    params: &[Param],
    ret_ty: Option<&TypeExpr>,
    void_mark: Option<&TypeExpr>,
    stmts: &[Stmt],
    fe: &Expr,
) -> String {
    let scope0: Scope = params.iter().map(|p| p.name.clone()).collect();
    let param_str = codegen_params_mut(params);
    let ret_str = match void_mark {
        Some(_) => String::new(),
        None => ret_ty.map_or(String::new(), |t| format!(" -> {}", codegen_type(t))),
    };
    let generics = infer_generics(params);
    let gen_str = if generics.is_empty() {
        String::new()
    } else {
        format!("<{}>", generics.join(", "))
    };
    let body_lines = cg_body(i + 1, &scope0, stmts, fe);
    let mut lines = vec![format!(
        "pub fn {}{}({}){} {{",
        name, gen_str, param_str, ret_str
    )];
    lines.extend(body_lines);
    lines.push("}".to_string());
    lines.join("\n")
}

// ─── Body / statement codegen with scope ─────────────────────

fn cg_body(i: Indent, scope: &Scope, stmts: &[Stmt], fe: &Expr) -> Vec<String> {
    let (mut lines, scope2) = cg_stmts(i, scope, stmts);
    lines.push(format!("{}{}", ind(i), cg_expr(i, &scope2, fe)));
    lines
}

fn cg_stmts(i: Indent, scope: &Scope, stmts: &[Stmt]) -> (Vec<String>, Scope) {
    let mut scope = scope.clone();
    let mut lines = Vec::new();
    for s in stmts {
        let (line, new_scope) = cg_stmt(i, &scope, s);
        lines.push(line);
        scope = new_scope;
    }
    (lines, scope)
}

fn cg_stmt(i: Indent, scope: &Scope, stmt: &Stmt) -> (String, Scope) {
    match stmt {
        Stmt::Bind(
            name,
            Expr::Lambda {
                params,
                stmts,
                final_expr,
                ..
            },
        ) => {
            let param_str = params
                .iter()
                .map(codegen_param)
                .collect::<Vec<_>>()
                .join(", ");
            let mut inner_scope = scope.clone();
            for p in params {
                inner_scope.insert(p.name.clone());
            }
            let body_lines = cg_body(i + 1, &inner_scope, stmts, final_expr);
            let line = format!(
                "{}let {} = |{}| {{\n{}\n{}}};",
                ind(i),
                name,
                param_str,
                body_lines.join("\n"),
                ind(i)
            );
            let mut s = scope.clone();
            s.insert(name.clone());
            (line, s)
        }
        Stmt::Bind(name, expr) => {
            let rhs = cg_expr(i, scope, expr);
            if scope.contains(name) {
                (format!("{}{} = {};", ind(i), name, rhs), scope.clone())
            } else {
                let mut s = scope.clone();
                s.insert(name.clone());
                (format!("{}let mut {} = {};", ind(i), name, rhs), s)
            }
        }
        Stmt::Use(path) => (format!("{}use {};", ind(i), path.join("::")), scope.clone()),
        Stmt::StructDef(name, fields) => {
            let field_lines: Vec<String> = fields.iter().map(|f| codegen_field(i + 1, f)).collect();
            let line = format!(
                "{}#[derive(Debug,Clone)]\n{}struct {} {{\n{}\n{}}}",
                ind(i),
                ind(i),
                name,
                field_lines.join("\n"),
                ind(i)
            );
            let mut s = scope.clone();
            s.insert(name.clone());
            (line, s)
        }
        Stmt::EnumDef(name, variants) => {
            let var_lines: Vec<String> =
                variants.iter().map(|v| codegen_variant(i + 1, v)).collect();
            let line = format!(
                "{}#[derive(Debug,Clone)]\n{}enum {} {{\n{}\n{}}}",
                ind(i),
                ind(i),
                name,
                var_lines.join("\n"),
                ind(i)
            );
            let mut s = scope.clone();
            s.insert(name.clone());
            (line, s)
        }
        Stmt::Expression(e) => (
            format!("{}{};", ind(i), cg_expr(i, scope, e)),
            scope.clone(),
        ),
    }
}

// ─── Parameters ──────────────────────────────────────────────

fn codegen_param(p: &Param) -> String {
    match (&p.name[..], &p.ty) {
        ("_", _) => "_: _".to_string(),
        (name, None) => format!("{}: _", name),
        (name, Some(ty)) => format!("{}: {}", name, codegen_type(ty)),
    }
}

fn codegen_params_mut(params: &[Param]) -> String {
    let generics = ["T", "U", "V", "W", "X", "Y", "Z"];
    let mut gen_idx = 0;
    let mut parts = Vec::new();
    for p in params {
        if p.name == "_" {
            parts.push("_: _".to_string());
        } else if let Some(ty) = &p.ty {
            parts.push(format!("mut {}: {}", p.name, codegen_type(ty)));
        } else {
            let g = generics[gen_idx];
            gen_idx += 1;
            parts.push(format!("mut {}: {}", p.name, g));
        }
    }
    parts.join(", ")
}

fn infer_generics(params: &[Param]) -> Vec<String> {
    let generics = ["T", "U", "V", "W", "X", "Y", "Z"];
    let n = params
        .iter()
        .filter(|p| p.ty.is_none() && p.name != "_")
        .count();
    generics[..n]
        .iter()
        .map(|g| format!("{}: Clone", g))
        .collect()
}

// ─── Expressions ─────────────────────────────────────────────

fn cg_expr(i: Indent, sc: &Scope, expr: &Expr) -> String {
    match expr {
        Expr::Int(n) => n.to_string(),
        Expr::Float(n) => format!("{}f32", n),
        Expr::Bool(b) => if *b { "true" } else { "false" }.to_string(),
        Expr::None => "None".to_string(),
        Expr::Str(s) => codegen_string(s),
        Expr::Var(n) if n == "_" => "_".to_string(),
        Expr::Var(n) if n == "str" => "str_of".to_string(),
        Expr::Var(n) => n.clone(),

        Expr::Field(e, field) => format!("{}.{}", cg_expr(i, sc, e), field),
        Expr::Index(e, idx) => format!("{}.homun_idx({})", cg_expr(i, sc, e), cg_expr(i, sc, idx)),

        Expr::Slice(e, start, end, step) => {
            format!(
                "slice!({}, {}, {}, {})",
                cg_expr(i, sc, e),
                start
                    .as_ref()
                    .map_or("0".to_string(), |e| cg_expr(i, sc, e)),
                end.as_ref()
                    .map_or("i64::MAX".to_string(), |e| cg_expr(i, sc, e)),
                step.as_ref().map_or("1".to_string(), |e| cg_expr(i, sc, e)),
            )
        }

        Expr::List(items) => format!("vec![{}]", commas(i, sc, items)),
        Expr::Dict(pairs) => {
            let inner: Vec<String> = pairs
                .iter()
                .map(|(k, v)| format!("{} => {}", cg_expr(i, sc, k), cg_expr(i, sc, v)))
                .collect();
            format!("dict![{}]", inner.join(", "))
        }
        Expr::Set(items) => format!("set![{}]", commas(i, sc, items)),
        Expr::Tuple(items) => format!("({})", commas(i, sc, items)),

        Expr::Struct(Some(name), fields) => {
            let inner: Vec<String> = fields
                .iter()
                .map(|(n, e)| format!("{}: {}", n, struct_val(i, sc, e)))
                .collect();
            format!("{} {{ {} }}", name, inner.join(", "))
        }
        Expr::Struct(None, fields) => {
            let inner: Vec<String> = fields.iter().map(|(_, e)| cg_expr(i, sc, e)).collect();
            format!("({})", inner.join(", "))
        }

        Expr::BinOp(op, lhs, rhs) => cg_bin_op(i, sc, op, lhs, rhs),
        Expr::UnOp(UnOp::Not, e) => format!("!{}", cg_expr(i, sc, e)),
        Expr::UnOp(UnOp::Neg, e) => format!("-{}", cg_expr(i, sc, e)),

        Expr::Pipe(lhs, rhs) => match rhs.as_ref() {
            Expr::Call(f, args) => {
                if let Expr::Var(fn_name) = f.as_ref() {
                    if HOMUN_MACROS.contains(&fn_name.as_str()) {
                        return format!(
                            "{}!({}{})",
                            fn_name,
                            cg_expr(i, sc, lhs),
                            opt_args(i, sc, args)
                        );
                    }
                }
                let mut all_args = vec![cg_expr(i, sc, lhs)];
                all_args.extend(args.iter().map(|a| cg_expr(i, sc, a)));
                format!("{}({})", cg_expr(i, sc, f), all_args.join(", "))
            }
            _ => format!("{}({})", cg_expr(i, sc, rhs), cg_expr(i, sc, lhs)),
        },

        Expr::Lambda {
            params,
            stmts,
            final_expr,
            ..
        } => {
            let param_str = params
                .iter()
                .map(codegen_param)
                .collect::<Vec<_>>()
                .join(", ");
            let mut inner_scope = sc.clone();
            for p in params {
                inner_scope.insert(p.name.clone());
            }
            let body_lines = cg_body(i + 1, &inner_scope, stmts, final_expr);
            format!(
                "|{}| {{\n{}\n{}}}",
                param_str,
                body_lines.join("\n"),
                ind(i)
            )
        }

        Expr::Call(f, args) => {
            if let Expr::Var(n) = f.as_ref() {
                if n == "print" {
                    return cg_print(i, sc, args);
                }
                if HOMUN_MACROS.contains(&n.as_str()) {
                    return format!("{}!({})", n, commas(i, sc, args));
                }
            }
            let arg_strs: Vec<String> = args.iter().map(|a| clone_arg(i, sc, a)).collect();
            format!("{}({})", cg_expr(i, sc, f), arg_strs.join(", "))
        }

        Expr::If(cond, ts, te, ec) => {
            let then_lines = cg_body(i + 1, sc, ts, te);
            let else_str = match ec {
                Some((es, ee)) => {
                    let else_lines = cg_body(i + 1, sc, es, ee);
                    format!(" else {{\n{}\n{}}}", else_lines.join("\n"), ind(i))
                }
                None => String::new(),
            };
            format!(
                "if {} {{\n{}\n{}}}{}",
                cg_expr(i, sc, cond),
                then_lines.join("\n"),
                ind(i),
                else_str
            )
        }

        Expr::Match(scrut, arms) => {
            let arm_strs: Vec<String> = arms.iter().map(|a| cg_arm(i, sc, a)).collect();
            format!(
                "match {} {{\n{}\n{}}}",
                cg_expr(i, sc, scrut),
                arm_strs.join("\n"),
                ind(i)
            )
        }

        Expr::Block(stmts, fe) => {
            let body_lines = cg_body(i + 1, sc, stmts, fe);
            format!("{{\n{}\n{}}}", body_lines.join("\n"), ind(i))
        }

        Expr::For(var, iter, stmts, fe) => {
            let mut scope0 = sc.clone();
            scope0.insert(var.clone());
            let (body_lines, _) = cg_stmts(i + 1, &scope0, stmts);
            let final_line = match fe {
                Some(e) => {
                    if matches!(e.as_ref(), Expr::Tuple(items) if items.is_empty()) {
                        vec![]
                    } else {
                        vec![format!("{}{};", ind(i + 1), cg_expr(i + 1, &scope0, e))]
                    }
                }
                None => vec![],
            };
            let mut all = body_lines;
            all.extend(final_line);
            format!(
                "for {} in {} {{\n{}\n{}}}",
                var,
                cg_expr(i, sc, iter),
                all.join("\n"),
                ind(i)
            )
        }

        Expr::While(cond, stmts, fe) => {
            let (body_lines, _) = cg_stmts(i + 1, sc, stmts);
            let final_line = match fe {
                Some(e) => vec![format!("{}{};", ind(i + 1), cg_expr(i + 1, sc, e))],
                None => vec![],
            };
            let mut all = body_lines;
            all.extend(final_line);
            format!(
                "while {} {{\n{}\n{}}}",
                cg_expr(i, sc, cond),
                all.join("\n"),
                ind(i)
            )
        }

        Expr::Break(None) => "break".to_string(),
        Expr::Break(Some(e)) => format!("return {}", cg_expr(i, sc, e)),
        Expr::Continue => "continue".to_string(),

        Expr::LoadRon(path, ty) => {
            format!(
                "ron::from_str::<{}>(&std::fs::read_to_string({}).unwrap()).unwrap()",
                codegen_type(ty),
                cg_expr(i, sc, path)
            )
        }
        Expr::SaveRon(d, p) => {
            format!(
                "std::fs::write({}, ron::to_string(&{}).unwrap()).unwrap()",
                cg_expr(i, sc, p),
                cg_expr(i, sc, d)
            )
        }

        Expr::TryUnwrap(inner) => format!("{}?", cg_expr(i, sc, inner)),

        Expr::Range(start, end, step) => match (start, end, step) {
            (None, Some(e), None) => format!("(0..{})", cg_expr(i, sc, e)),
            (Some(s), Some(e), None) => format!("({}..{})", cg_expr(i, sc, s), cg_expr(i, sc, e)),
            (Some(s), Some(e), Some(st)) => format!(
                "({}..{}).step_by({} as usize)",
                cg_expr(i, sc, s),
                cg_expr(i, sc, e),
                cg_expr(i, sc, st)
            ),
            _ => "(0..)".to_string(),
        },
    }
}

// ─── Expression helpers ──────────────────────────────────────

fn cg_bin_op(i: Indent, sc: &Scope, op: &BinOp, lhs: &Expr, rhs: &Expr) -> String {
    let l = cg_expr(i, sc, lhs);
    let r = cg_expr(i, sc, rhs);
    match op {
        BinOp::Add if is_list_expr(lhs) || is_list_expr(rhs) => {
            format!("homun_concat({}, {})", l, r)
        }
        BinOp::Add => format!("{} + {}", l, r),
        BinOp::Sub => format!("{} - {}", l, r),
        BinOp::Mul => format!("{} * {}", l, r),
        BinOp::Div => format!("{} / {}", l, r),
        BinOp::Mod => format!("{} % {}", l, r),
        BinOp::Eq => format!("{} == {}", l, r),
        BinOp::Neq => format!("{} != {}", l, r),
        BinOp::Lt => format!("{} < {}", l, r),
        BinOp::Gt => format!("{} > {}", l, r),
        BinOp::Le => format!("{} <= {}", l, r),
        BinOp::Ge => format!("{} >= {}", l, r),
        BinOp::And => format!("{} && {}", l, r),
        BinOp::Or => format!("{} || {}", l, r),
        BinOp::In => format!("homun_in!({}, {})", l, r),
        BinOp::NotIn => format!("!homun_in!({}, {})", l, r),
    }
}

fn cg_print(i: Indent, sc: &Scope, args: &[Expr]) -> String {
    match args {
        [Expr::Str(s)] => {
            let (fmt, fmt_args) = parse_interp(s);
            if fmt_args.is_empty() {
                format!("println!(\"{}\")", fmt)
            } else {
                format!("println!(\"{}\", {})", fmt, fmt_args.join(", "))
            }
        }
        [e] => format!("println!(\"{{}}\", {})", cg_expr(i, sc, e)),
        _ => format!(
            "println!({})",
            args.iter()
                .map(|e| cg_expr(i, sc, e))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn cg_arm(i: Indent, sc: &Scope, arm: &MatchArm) -> String {
    let pat_s = cg_pat(&arm.pat);
    let guard_s = arm
        .guard
        .as_ref()
        .map_or(String::new(), |g| format!(" if {}", cg_expr(i, sc, g)));
    let body_s = match &arm.body {
        Expr::Str(s) => format!("{}.to_string()", codegen_string(s)),
        other => cg_expr(i + 1, sc, other),
    };
    format!("{}{}{} => {},", ind(i + 1), pat_s, guard_s, body_s)
}

fn cg_pat(pat: &Pat) -> String {
    match pat {
        Pat::Wild => "_".to_string(),
        Pat::None => "None".to_string(),
        Pat::Var(n) => n.clone(),
        Pat::Lit(e) => cg_expr(0, &HashSet::new(), e),
        Pat::Tuple(pats) => {
            let inner: Vec<String> = pats.iter().map(cg_pat).collect();
            format!("({})", inner.join(", "))
        }
        Pat::Enum(n, None) => n.clone(),
        Pat::Enum(n, Some(p)) => format!("{}({})", n, cg_pat(p)),
    }
}

// ─── Types ───────────────────────────────────────────────────

fn codegen_type(ty: &TypeExpr) -> String {
    match ty {
        TypeExpr::Name(n) => match n.as_str() {
            "int" => "i32".to_string(),
            "float" => "f32".to_string(),
            "bool" => "bool".to_string(),
            "str" => "String".to_string(),
            "none" => "Option<_>".to_string(),
            _ => n.clone(),
        },
        TypeExpr::List(t) => format!("Vec<{}>", codegen_type(t)),
        TypeExpr::Dict(k, v) => format!(
            "std::collections::HashMap<{}, {}>",
            codegen_type(k),
            codegen_type(v)
        ),
        TypeExpr::Set(t) => format!("std::collections::HashSet<{}>", codegen_type(t)),
        TypeExpr::Option(t) => format!("Option<{}>", codegen_type(t)),
        TypeExpr::Tuple(ts) => {
            let inner: Vec<String> = ts.iter().map(codegen_type).collect();
            format!("({})", inner.join(", "))
        }
        TypeExpr::Void => "()".to_string(),
        TypeExpr::Infer => "_".to_string(),
    }
}

// ─── String interpolation ────────────────────────────────────

fn codegen_string(s: &str) -> String {
    let (fmt, args) = parse_interp(s);
    if args.is_empty() {
        format!("\"{}\"", escape_str(s))
    } else {
        format!("format!(\"{}\", {})", fmt, args.join(", "))
    }
}

fn parse_interp(s: &str) -> (String, Vec<String>) {
    let mut fmt = String::new();
    let mut args = Vec::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '{' && i + 1 < chars.len() && chars[i + 1] == '{' {
            fmt.push_str("{{");
            i += 2;
        } else if chars[i] == '$' && i + 1 < chars.len() && chars[i + 1] == '{' {
            i += 2;
            let start = i;
            while i < chars.len() && chars[i] != '}' {
                i += 1;
            }
            let expr: String = chars[start..i].iter().collect();
            args.push(expr);
            fmt.push_str("{}");
            if i < chars.len() {
                i += 1;
            } // skip }
        } else {
            let c = chars[i];
            if c == '"' {
                fmt.push_str("\\\"");
            } else {
                fmt.push(c);
            }
            i += 1;
        }
    }
    (fmt, args)
}

fn escape_str(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out
}

// ─── Utilities ───────────────────────────────────────────────

fn to_upper(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_lowercase() {
                c.to_ascii_uppercase()
            } else {
                c
            }
        })
        .collect()
}

const HOMUN_MACROS: &[&str] = &[
    "range", "len", "filter", "map", "reduce", "slice", "dict", "set",
];

fn is_list_expr(expr: &Expr) -> bool {
    match expr {
        Expr::List(_) | Expr::Slice(..) => true,
        Expr::BinOp(BinOp::Add, l, r) => is_list_expr(l) || is_list_expr(r),
        _ => false,
    }
}

fn clone_arg(i: Indent, sc: &Scope, expr: &Expr) -> String {
    match expr {
        Expr::Var(n) => format!("{}.clone()", n),
        _ => cg_expr(i, sc, expr),
    }
}

fn commas(i: Indent, sc: &Scope, items: &[Expr]) -> String {
    items
        .iter()
        .map(|e| cg_expr(i, sc, e))
        .collect::<Vec<_>>()
        .join(", ")
}

fn opt_args(i: Indent, sc: &Scope, args: &[Expr]) -> String {
    if args.is_empty() {
        String::new()
    } else {
        format!(", {}", commas(i, sc, args))
    }
}

fn struct_val(i: Indent, sc: &Scope, expr: &Expr) -> String {
    match expr {
        Expr::Str(s) => format!("{}.to_string()", codegen_string(s)),
        _ => cg_expr(i, sc, expr),
    }
}

// ─── Tests ───────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer;
    use crate::parser;

    fn compile_snippet(src: &str) -> String {
        let tokens = lexer::lex(src).expect("lex failed");
        let ast = parser::parse(tokens).expect("parse failed");
        let empty_set = HashSet::new();
        let empty_map = HashMap::new();
        codegen_program_with_resolved(&ast, &empty_set, &empty_map)
    }

    /// A2: Ok()/Err() constructors — verified: codegen emits Ok(42) and Err("fail")
    /// No compiler changes needed; these are plain function calls.
    #[test]
    fn test_ok_err_codegen() {
        let src = r#"
main := () -> _ {
  x := Ok(42)
  y := Err("fail")
}
"#;
        let out = compile_snippet(src);
        assert!(
            out.contains("Ok(42)"),
            "codegen should emit Ok(42), got:\n{}",
            out
        );
        assert!(
            out.contains("Err(\"fail\".to_string())") || out.contains("Err(\"fail\")"),
            "codegen should emit Err(\"fail\"), got:\n{}",
            out
        );
    }
}
