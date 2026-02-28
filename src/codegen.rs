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
                "#[derive(Debug, Clone, PartialEq)]".to_string(),
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
                "#[derive(Debug, Clone, PartialEq)]".to_string(),
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
        Stmt::BindPat(_, _) => String::new(), // not valid at top-level
        Stmt::Assign(_, _) => String::new(),  // not valid at top-level
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
        Stmt::BindPat(pat, expr) => {
            let rhs = cg_expr(i, scope, expr);
            let pat_s = cg_bind_pat(pat);
            let mut s = scope.clone();
            bind_vars_from_pat(pat, &mut s);
            (format!("{}let ({}) = {};", ind(i), pat_s, rhs), s)
        }
        Stmt::Assign(lhs, rhs) => {
            let rhs_s = cg_expr(i, scope, rhs);
            (
                format!("{}{} = {};", ind(i), cg_lvalue(i, scope, lhs), rhs_s),
                scope.clone(),
            )
        }
        Stmt::Use(path) => (format!("{}use {};", ind(i), path.join("::")), scope.clone()),
        Stmt::StructDef(name, fields) => {
            let field_lines: Vec<String> = fields.iter().map(|f| codegen_field(i + 1, f)).collect();
            let line = format!(
                "{}#[derive(Debug, Clone, PartialEq)]\n{}struct {} {{\n{}\n{}}}",
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
                "{}#[derive(Debug, Clone, PartialEq)]\n{}enum {} {{\n{}\n{}}}",
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

        Expr::Field(e, field) => {
            // If the base is a PascalCase name, emit :: (enum variant) instead of . (struct field).
            let base = cg_expr(i, sc, e);
            if base.chars().next().is_some_and(|c| c.is_uppercase()) {
                format!("{}::{}", base, field)
            } else {
                format!("{}.{}", base, field)
            }
        }
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
        BinOp::Add if is_str_expr(lhs) || is_str_expr(rhs) => {
            // String concat: ensure LHS is String and RHS is &str for Rust's + operator
            let ls = if matches!(lhs, Expr::Str(_)) {
                format!("{}.to_string()", l)
            } else {
                l
            };
            let rs = format!("&{}", r);
            format!("{} + {}", ls, rs)
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
        Pat::Enum(n, None) => n.replace('.', "::"),
        Pat::Enum(n, Some(p)) => format!("{}({})", n.replace('.', "::"), cg_pat(p)),
    }
}

/// Emit the interior of `let (…) = rhs;` for a BindPat.
/// Each variable gets `mut` prefix; wildcards pass through.
fn cg_bind_pat(pat: &Pat) -> String {
    match pat {
        Pat::Tuple(pats) => pats.iter().map(cg_bind_var).collect::<Vec<_>>().join(", "),
        _ => cg_bind_var(pat),
    }
}

fn cg_bind_var(pat: &Pat) -> String {
    match pat {
        Pat::Var(n) => format!("mut {}", n),
        Pat::Wild => "_".to_string(),
        _ => cg_pat(pat),
    }
}

/// Emit the lvalue side of an assignment.
/// Uses `[idx as usize]` instead of `.homun_idx(idx)` so the result is mutable.
fn cg_lvalue(i: Indent, sc: &Scope, expr: &Expr) -> String {
    match expr {
        Expr::Var(n) => n.clone(),
        Expr::Field(base, field) => format!("{}.{}", cg_lvalue(i, sc, base), field),
        Expr::Index(base, idx) => {
            format!(
                "{}[{} as usize]",
                cg_lvalue(i, sc, base),
                cg_expr(i, sc, idx)
            )
        }
        _ => cg_expr(i, sc, expr),
    }
}

fn bind_vars_from_pat(pat: &Pat, scope: &mut Scope) {
    match pat {
        Pat::Var(n) => {
            scope.insert(n.clone());
        }
        Pat::Tuple(pats) => {
            for p in pats {
                bind_vars_from_pat(p, scope);
            }
        }
        _ => {}
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
        TypeExpr::Generic(n, params) => {
            let inner: Vec<String> = params.iter().map(codegen_type).collect();
            format!("{}<{}>", n, inner.join(", "))
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

fn is_str_expr(expr: &Expr) -> bool {
    match expr {
        Expr::Str(_) => true,
        Expr::BinOp(BinOp::Add, l, r) => is_str_expr(l) || is_str_expr(r),
        Expr::Call(f, _) => matches!(f.as_ref(), Expr::Var(n) if n == "str"),
        _ => false,
    }
}

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
        // String literals passed as arguments need .to_string() so they become String, not &str.
        Expr::Str(s) => format!("{}.to_string()", codegen_string(s)),
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

    /// A3: Tuple destructuring bind — `a, b := expr` emits `let (mut a, mut b) = expr;`
    #[test]
    fn test_tuple_bind_codegen() {
        let src = r#"
foo := () -> _ {
  x, y := get_pair()
}
"#;
        let out = compile_snippet(src);
        assert!(
            out.contains("let (mut x, mut y) = get_pair()"),
            "tuple bind should emit let destructure, got:\n{}",
            out
        );
    }

    /// A3: Three-element tuple destructure
    #[test]
    fn test_tuple_bind_three() {
        let src = r#"
foo := () -> _ {
  a, b, c := triple()
}
"#;
        let out = compile_snippet(src);
        assert!(
            out.contains("let (mut a, mut b, mut c) = triple()"),
            "three-element tuple bind, got:\n{}",
            out
        );
    }

    /// Tuple bind with underscore wildcard — `a, _, c := triple()` emits `let (mut a, _, mut c) = triple()`
    #[test]
    fn test_tuple_bind_wildcard() {
        let src = r#"
foo := () -> _ {
  a, _, c := triple()
  a
}
"#;
        let out = compile_snippet(src);
        assert!(
            out.contains("let (mut a, _, mut c) = triple()"),
            "tuple bind with wildcard, got:\n{}",
            out
        );
    }

    /// `not` on next line after tuple bind must NOT be parsed as `not in`
    #[test]
    fn test_not_after_tuple_bind() {
        let src = r#"
triple := () -> _ { (true, "a", 0) }

test_fn := () -> bool {
  m, a, b := triple()
  not m
}
"#;
        let out = compile_snippet(src);
        assert!(
            out.contains("!m"),
            "not m should compile to !m, got:\n{}",
            out
        );
    }

    /// A4-L2: Tuple patterns in match arms — `(0, 1)` emits `(0, 1) =>`
    #[test]
    fn test_tuple_pat_match() {
        let src = r#"
foo := (dx, dy) -> str {
  match (dx, dy) {
    (0, 1) => "down"
    (0, -1) => "up"
    (1, 0) => "right"
    _ => "other"
  }
}
"#;
        let out = compile_snippet(src);
        assert!(
            out.contains("(0, 1) =>"),
            "tuple pattern should emit (0, 1) =>, got:\n{}",
            out
        );
        assert!(
            out.contains("(0, -1) =>"),
            "negative tuple pattern should emit (0, -1) =>, got:\n{}",
            out
        );
    }

    /// A4-L3: Constructor patterns Ok(x), Err(msg) in match arms
    #[test]
    fn test_constructor_pat_ok_err() {
        let src = r#"
foo := (result) -> str {
  match result {
    Ok(value) => value
    Err(msg) => msg
  }
}
"#;
        let out = compile_snippet(src);
        assert!(
            out.contains("Ok(value) =>"),
            "constructor pattern Ok(value) should emit Ok(value) =>, got:\n{}",
            out
        );
        assert!(
            out.contains("Err(msg) =>"),
            "constructor pattern Err(msg) should emit Err(msg) =>, got:\n{}",
            out
        );
    }

    /// A4-L3: Some(x) constructor pattern and nested Ok(Some(x))
    #[test]
    fn test_constructor_pat_some_nested() {
        let src = r#"
foo := (opt) -> int {
  match opt {
    Some(x) => x
    none => 0
  }
}
"#;
        let out = compile_snippet(src);
        assert!(
            out.contains("Some(x) =>"),
            "constructor pattern Some(x) should emit Some(x) =>, got:\n{}",
            out
        );
    }

    /// A5: Simple indexed assignment — `grid[y][x] := "X"` emits `grid[y as usize][x as usize] = ...`
    #[test]
    fn test_index_assign_simple() {
        let src = r#"
foo := (grid, y, x) -> _ {
  grid[y][x] := "X"
}
"#;
        let out = compile_snippet(src);
        assert!(
            out.contains("grid[y as usize][x as usize]"),
            "nested index assign should emit grid[y as usize][x as usize], got:\n{}",
            out
        );
        // must be an assignment (= not let)
        assert!(
            !out.contains("let mut grid["),
            "index assign must not emit let, got:\n{}",
            out
        );
    }

    /// A5: Field + nested index assignment — `canvas.cells[y][x] := ch`
    #[test]
    fn test_index_assign_field() {
        let src = r#"
foo := (canvas, y, x, ch) -> _ {
  canvas.cells[y][x] := ch
}
"#;
        let out = compile_snippet(src);
        assert!(
            out.contains("canvas.cells[y as usize][x as usize]"),
            "field+index assign should emit canvas.cells[y as usize][x as usize], got:\n{}",
            out
        );
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

    /// TypeExpr::Generic — `Result<int, str>` return type emits `Result<i32, String>`
    #[test]
    fn test_generic_type_result_return() {
        let src = r#"
safe_div := (a: int, b: int) -> Result<int, str> {
  Ok(a / b)
}
"#;
        let out = compile_snippet(src);
        assert!(
            out.contains("-> Result<i32, String>"),
            "generic return type should emit -> Result<i32, String>, got:\n{}",
            out
        );
        assert!(
            out.contains("pub fn safe_div"),
            "should have pub fn safe_div, got:\n{}",
            out
        );
    }

    /// TypeExpr::Generic — `Result<int, str>` param type emits `mut r: Result<i32, String>`
    #[test]
    fn test_generic_type_in_param() {
        let src = r#"
describe := (r: Result<int, str>) -> str {
  match r {
    Ok(val) => "ok"
    Err(msg) => "err"
  }
}
"#;
        let out = compile_snippet(src);
        assert!(
            out.contains("mut r: Result<i32, String>"),
            "generic param should emit mut r: Result<i32, String>, got:\n{}",
            out
        );
        assert!(
            out.contains("Ok(val) =>"),
            "constructor pattern Ok(val) should be emitted, got:\n{}",
            out
        );
        assert!(
            out.contains("Err(msg) =>"),
            "constructor pattern Err(msg) should be emitted, got:\n{}",
            out
        );
    }

    /// Enum variant expressions use :: not . — Direction.TD → Direction::TD
    #[test]
    fn test_enum_variant_double_colon() {
        let src = r#"
Direction := enum { LR, TD }
direction_default := () -> Direction { Direction.TD }
"#;
        let out = compile_snippet(src);
        assert!(
            out.contains("Direction::TD"),
            "enum variant should use :: not ., got:\n{}",
            out
        );
        assert!(
            !out.contains("Direction.TD"),
            "should not contain Direction.TD, got:\n{}",
            out
        );
    }

    /// Enum variant in match pattern uses :: — Direction.LR → Direction::LR
    #[test]
    fn test_enum_variant_match_pattern() {
        let src = r#"
Direction := enum { LR, TD }
is_lr := (d: Direction) -> bool {
  match d {
    Direction.LR => true
    _ => false
  }
}
"#;
        let out = compile_snippet(src);
        assert!(
            out.contains("Direction::LR"),
            "match pattern should use :: for enum variant, got:\n{}",
            out
        );
    }

    /// Integration test: compile a multi-file .hom project using all Part A features.
    ///
    /// Features exercised:
    ///   A1: ? operator in a Result-returning function
    ///   A2: Ok()/Err() constructors
    ///   A3: Tuple destructuring bind (a, b := expr)
    ///   A4-L2: Tuple patterns in match arms ((0,1) => ...)
    ///   A4-L3: Constructor patterns in match arms (Ok(val) => ...)
    ///   A5: Mutable nested indexing (grid[y][x] := val)
    ///
    /// Verifies: homunc compiles to valid .rs, rustc compiles .rs, binary runs correctly.
    #[test]
    fn test_integration_all_part_a_features() {
        use std::fs;
        use std::process::Command;

        let tmp = std::env::temp_dir().join("homun_integ_parta");
        fs::create_dir_all(&tmp).unwrap();

        // helpers.hom — safe_div returns Result<int, str>
        let helpers_hom = r#"
safe_div := (a: int, b: int) -> Result<int, str> {
  if (b == 0) do {
    Err("division by zero")
  } else {
    Ok(a / b)
  }
}
"#;
        fs::write(tmp.join("helpers.hom"), helpers_hom).unwrap();

        // main.hom — uses all Part A features
        let main_hom = r#"
use helpers
use std

describe_result := (r: Result<int, str>) -> str {
  match r {
    Ok(val) => "ok"
    Err(msg) => "err"
  }
}

direction_name := (dx: int, dy: int) -> str {
  match (dx, dy) {
    (0, 1) => "south"
    (1, 0) => "east"
    _ => "other"
  }
}

process_div := (a: int, b: int) -> Result<int, str> {
  q := safe_div(a, b)?
  Ok(q * 2)
}

main := () -> _ {
  r := safe_div(10, 2)
  desc := describe_result(r)
  print(desc)
  x, y := (3, 7)
  print("${x} ${y}")
  dir := direction_name(0, 1)
  print(dir)
  p := process_div(20, 4)
  desc2 := describe_result(p)
  print(desc2)
  print("? operator ok")
  grid := @[@[0, 0, 0], @[0, 0, 0], @[0, 0, 0]]
  grid[1][1] := 99
  print("nested ok")
}
"#;
        fs::write(tmp.join("main.hom"), main_hom).unwrap();

        // Compile via resolver (multi-file)
        let resolved = crate::resolver::resolve(&tmp.join("main.hom"))
            .unwrap_or_else(|e| panic!("Resolver failed: {}", e));

        // Build full Rust source with preamble
        let preamble = concat!(
            "// Generated by homunc — integration test\n",
            "#![allow(unused_variables, unused_mut, dead_code, unused_imports,\
             unused_macros, unused_assignments)]\n",
            "#![allow(non_snake_case)]\n",
            "// ── builtin ────────────────────────────────────────────────\n",
            include_str!("../hom/builtin.rs"),
            "\n"
        );

        let mut rust_src = preamble.to_string();
        for (i, file) in resolved.files.iter().enumerate() {
            rust_src.push_str(&file.rust_code);
            if i + 1 < resolved.files.len() {
                rust_src.push('\n');
            }
        }

        // Write Rust source to temp file
        let rs_path = tmp.join("integ_test.rs");
        fs::write(&rs_path, &rust_src).unwrap();

        // Compile with rustc
        let bin_path = tmp.join("integ_test_bin");
        let compile_out = Command::new("rustc")
            .arg(rs_path.to_str().unwrap())
            .arg("-o")
            .arg(bin_path.to_str().unwrap())
            .output()
            .unwrap_or_else(|e| panic!("rustc not found: {}", e));

        if !compile_out.status.success() {
            let stderr = String::from_utf8_lossy(&compile_out.stderr);
            panic!(
                "rustc compilation failed:\n{}\n\nGenerated source:\n{}",
                stderr, rust_src
            );
        }

        // Run the compiled binary
        let run_out = Command::new(&bin_path)
            .output()
            .unwrap_or_else(|e| panic!("Failed to run binary: {}", e));

        assert!(
            run_out.status.success(),
            "Binary exited with error: {}",
            String::from_utf8_lossy(&run_out.stderr)
        );

        let stdout = String::from_utf8(run_out.stdout).unwrap();
        let lines: Vec<&str> = stdout.trim().lines().collect();

        assert_eq!(lines.len(), 6, "expected 6 output lines, got: {:?}", lines);
        // A2 + A4-L3: Ok/Err + constructor pattern
        assert_eq!(
            lines[0], "ok",
            "A2/A4-L3: describe_result(safe_div(10,2)) == ok"
        );
        // A3: tuple destructuring
        assert_eq!(lines[1], "3 7", "A3: x, y := (3,7) prints '3 7'");
        // A4-L2: tuple pattern
        assert_eq!(lines[2], "south", "A4-L2: direction_name(0,1) == south");
        // A1 + A4-L3: ? operator
        assert_eq!(
            lines[3], "ok",
            "A1: process_div(20,4) succeeds, describe_result == ok"
        );
        assert_eq!(lines[4], "? operator ok", "A1: ? operator printed");
        // A5: nested indexing
        assert_eq!(lines[5], "nested ok", "A5: nested indexing works");
    }
}
