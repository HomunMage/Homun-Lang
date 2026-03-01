// dep/codegen_helpers.rs — String and type helpers for .hom codegen module.
//
// These functions are extracted from codegen.rs so that the future codegen.hom
// can call them through the dep bridge.  They contain Rust-specific logic that
// cannot be expressed in Homun:
//   - parse_interp() / escape_str() : character-level string scanning
//   - codegen_type()                : recursive TypeExpr → Rust type mapping
//   - codegen_params_mut()          : Param[] → "mut p: T" strings
//   - infer_generics()              : count un-typed params → T/U/V generic list
//   - is_str_expr() / is_list_expr(): expr-kind predicates for operator dispatch
//
// All functions take owned values (not references) so that .hom-generated code,
// which wraps every argument in `.clone()`, can call them without type errors.

// ─── Indentation ─────────────────────────────────────────────────────────────

/// Returns `n * 4` spaces as an indentation string.
pub fn ind(n: i32) -> String {
    " ".repeat((n * 4) as usize)
}

/// Concatenate two `Vec<String>` values (helper for codegen.hom list concat).
pub fn vec_extend_strings(mut a: Vec<String>, b: Vec<String>) -> Vec<String> {
    a.extend(b);
    a
}

// ─── String interpolation ────────────────────────────────────────────────────

/// Parse a Homun string literal for `${}` interpolation.
/// Returns `(format_string, vec_of_args)` where `format_string` uses `{}`
/// for each interpolated slot and `vec_of_args` contains the raw expression text.
pub fn parse_interp(s: String) -> (String, Vec<String>) {
    let mut fmt = String::new();
    let mut args = Vec::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0usize;
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
                i += 1; // skip '}'
            }
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

/// Escape a raw string value for embedding in a Rust string literal.
pub fn escape_str(s: String) -> String {
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

/// Render a Homun string value to a Rust string expression.
/// If the string has no `${}` interpolations, emits a `"..."` literal.
/// Otherwise emits a `format!(...)` call.
pub fn codegen_string(s: String) -> String {
    let (fmt, args) = parse_interp(s.clone());
    if args.is_empty() {
        format!("\"{}\"", escape_str(s))
    } else {
        format!("format!(\"{}\", {})", fmt, args.join(", "))
    }
}

// ─── Type codegen ────────────────────────────────────────────────────────────

/// Convert a Homun TypeExpr to its Rust type spelling.
pub fn codegen_type(ty: TypeExpr) -> String {
    match ty {
        TypeExpr::Name(n) => match n.as_str() {
            "int" => "i32".to_string(),
            "float" => "f32".to_string(),
            "bool" => "bool".to_string(),
            "str" => "String".to_string(),
            "char" => "char".to_string(),
            "none" => "Option<_>".to_string(),
            _ => n,
        },
        TypeExpr::List(inner) => format!("Vec<{}>", codegen_type(*inner)),
        TypeExpr::Dict(k, v) => format!(
            "std::collections::HashMap<{}, {}>",
            codegen_type(*k),
            codegen_type(*v)
        ),
        TypeExpr::Set(inner) => {
            format!("std::collections::HashSet<{}>", codegen_type(*inner))
        }
        TypeExpr::Option(inner) => format!("Option<{}>", codegen_type(*inner)),
        TypeExpr::Tuple(ts) => {
            let inner: Vec<String> = ts.into_iter().map(codegen_type).collect();
            format!("({})", inner.join(", "))
        }
        TypeExpr::Generic(n, params) => {
            let inner: Vec<String> = params.into_iter().map(codegen_type).collect();
            format!("{}<{}>", n, inner.join(", "))
        }
        TypeExpr::Void => "()".to_string(),
        TypeExpr::Infer => "_".to_string(),
    }
}

// ─── Parameter codegen ───────────────────────────────────────────────────────

/// Render a single parameter without `mut` prefix (for inline lambdas).
pub fn codegen_param(p: Param) -> String {
    match p.name.as_str() {
        "_" => "_: _".to_string(),
        name => match p.ty {
            None => format!("{}: _", name),
            Some(ty) => format!("{}: {}", name, codegen_type(ty)),
        },
    }
}

/// Render a parameter list with `mut` prefix for top-level function signatures.
/// Untyped parameters get generic type letters T, U, V, ...
pub fn codegen_params_mut(params: Vec<Param>) -> String {
    let generics = ["T", "U", "V", "W", "X", "Y", "Z"];
    let mut gen_idx = 0usize;
    let mut parts = Vec::new();
    for p in params {
        if p.name == "_" {
            parts.push("_: _".to_string());
        } else if let Some(ty) = p.ty {
            parts.push(format!("mut {}: {}", p.name, codegen_type(ty)));
        } else {
            let g = generics[gen_idx];
            gen_idx += 1;
            parts.push(format!("mut {}: {}", p.name, g));
        }
    }
    parts.join(", ")
}

/// Infer the list of generic type parameters (`T: Clone`, `U: Clone`, ...) needed
/// for a function based on how many parameters lack explicit type annotations.
pub fn infer_generics(params: Vec<Param>) -> Vec<String> {
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

// ─── Expression predicates ───────────────────────────────────────────────────

/// Returns `true` if the expression is known to produce a string value.
/// Used by `cg_bin_op` to decide whether `+` should emit string concatenation.
pub fn is_str_expr(expr: Expr) -> bool {
    match expr {
        Expr::Str(_) => true,
        Expr::BinOp(BinOp::Add, l, r) => is_str_expr(*l) || is_str_expr(*r),
        Expr::Call(f, _) => {
            if let Expr::Var(n) = *f {
                n == "str"
            } else {
                false
            }
        }
        _ => false,
    }
}

/// Returns `true` if the expression is known to produce a list (Vec) value.
/// Used by `cg_bin_op` to decide whether `+` should emit `homun_concat`.
pub fn is_list_expr(expr: Expr) -> bool {
    match expr {
        Expr::List(_) | Expr::Slice(_, _, _, _) => true,
        Expr::BinOp(BinOp::Add, l, r) => is_list_expr(*l) || is_list_expr(*r),
        _ => false,
    }
}

// ─── Utilities ───────────────────────────────────────────────────────────────

/// Returns `true` if the first character of `s` is an ASCII uppercase letter.
/// Used in codegen to distinguish enum variants (PascalCase) from struct fields.
pub fn is_upper_first(s: String) -> bool {
    s.chars().next().is_some_and(|c| c.is_uppercase())
}

// ─── Homun macro names ───────────────────────────────────────────────────────

/// Names of Homun builtins that are emitted as Rust macros (`name!(...)`)
/// rather than regular function calls.
pub const HOMUN_MACROS: &[&str] = &[
    "range", "len", "filter", "map", "reduce", "slice", "dict", "set",
];

/// Returns `true` if `name` is a Homun macro name.
pub fn is_homun_macro(name: String) -> bool {
    HOMUN_MACROS.contains(&name.as_str())
}

// ─── Preamble helpers ────────────────────────────────────────────────────────

/// Standard derive attributes for Homun-generated structs and enums.
pub fn derive_attrs() -> String {
    "#[derive(Debug, Clone, PartialEq)]".to_string()
}

/// Format the generic type-parameter clause for a function.
/// Returns `"<T: Clone, U: Clone>"` when non-empty, or `""` when empty.
pub fn generics_str(generics: Vec<String>) -> String {
    if generics.is_empty() {
        String::new()
    } else {
        format!("<{}>", generics.join(", "))
    }
}
