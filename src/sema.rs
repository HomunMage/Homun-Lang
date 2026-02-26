/// Semantic analysis pass for Homun.
///
/// 1. snake_case enforcement for variable/lambda names
/// 2. Undefined reference check for top-level bindings
/// 3. Mutual recursion detection
use crate::ast::*;
use std::collections::{HashMap, HashSet};
use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum SemaError {
    BadCasing(Name),
    MutualRec(Name, Name),
    Undefined(Name),
}

impl fmt::Display for SemaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SemaError::BadCasing(n) => write!(f, "SEMA ERROR: '{}' must be snake_case", n),
            SemaError::MutualRec(a, b) => write!(
                f,
                "SEMA ERROR: mutual recursion between '{}' and '{}' is forbidden",
                a, b
            ),
            SemaError::Undefined(n) => write!(f, "SEMA ERROR: undefined reference '{}'", n),
        }
    }
}

pub fn analyze_program(prog: &Program) -> Result<(), Vec<SemaError>> {
    analyze_program_with_imports(prog, &HashSet::new())
}

pub fn analyze_program_with_imports(
    prog: &Program,
    imported_names: &HashSet<String>,
) -> Result<(), Vec<SemaError>> {
    let builtins: HashSet<String> = [
        "print", "len", "range", "str", "int", "float", "bool", "filter", "map", "reduce",
        "load_ron", "save_ron", "clamp", "update", "idle", "attack", "die", "warn", "recover",
        "std",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();

    let top_names: HashSet<String> = prog
        .iter()
        .filter_map(|s| {
            if let Stmt::Bind(n, _) = s {
                Some(n.clone())
            } else {
                None
            }
        })
        .collect();

    let env0: HashSet<String> = builtins
        .union(&top_names)
        .cloned()
        .collect::<HashSet<_>>()
        .union(imported_names)
        .cloned()
        .collect();

    let mut errs = Vec::new();
    errs.extend(check_stmts(&env0, prog));
    errs.extend(check_casing_all(prog));
    errs.extend(check_mutual_rec(prog));

    if errs.is_empty() {
        Ok(())
    } else {
        Err(errs)
    }
}

// ─── 1. snake_case enforcement ───────────────────────────────

fn check_casing_all(prog: &Program) -> Vec<SemaError> {
    prog.iter().flat_map(check_casing_stmt).collect()
}

fn check_casing_stmt(stmt: &Stmt) -> Vec<SemaError> {
    match stmt {
        Stmt::Bind(n, _) => {
            if is_type_name(n) || is_snake(n) {
                vec![]
            } else {
                vec![SemaError::BadCasing(n.clone())]
            }
        }
        _ => vec![],
    }
}

fn is_snake(n: &str) -> bool {
    n.is_empty()
        || n.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

fn is_type_name(n: &str) -> bool {
    n.chars().next().is_some_and(|c| c.is_uppercase())
}

// ─── 2. Undefined reference check ───────────────────────────

fn check_stmts(env: &HashSet<String>, stmts: &[Stmt]) -> Vec<SemaError> {
    let mut errs = Vec::new();
    let mut env = env.clone();
    for s in stmts {
        errs.extend(check_stmt(&env, s));
        match s {
            Stmt::Bind(n, _) | Stmt::StructDef(n, _) | Stmt::EnumDef(n, _) => {
                env.insert(n.clone());
            }
            _ => {}
        }
    }
    errs
}

fn check_stmt(env: &HashSet<String>, stmt: &Stmt) -> Vec<SemaError> {
    match stmt {
        Stmt::Bind(_, e) | Stmt::Expression(e) => check_expr(env, e),
        _ => vec![],
    }
}

fn check_expr(env: &HashSet<String>, expr: &Expr) -> Vec<SemaError> {
    match expr {
        Expr::Var(n) => {
            if n == "_" || env.contains(n) {
                vec![]
            } else {
                vec![SemaError::Undefined(n.clone())]
            }
        }
        Expr::Field(e, _) => check_expr(env, e),
        Expr::Index(e, idx) => {
            let mut errs = check_expr(env, e);
            errs.extend(check_expr(env, idx));
            errs
        }
        Expr::Slice(e, a, b, c) => {
            let mut errs = check_expr(env, e);
            for x in [a, b, c].into_iter().flatten() {
                errs.extend(check_expr(env, x));
            }
            errs
        }
        Expr::List(xs) | Expr::Set(xs) | Expr::Tuple(xs) => {
            xs.iter().flat_map(|x| check_expr(env, x)).collect()
        }
        Expr::Dict(pairs) => pairs
            .iter()
            .flat_map(|(k, v)| {
                let mut e = check_expr(env, k);
                e.extend(check_expr(env, v));
                e
            })
            .collect(),
        Expr::Struct(_, fields) => fields
            .iter()
            .flat_map(|(_, e)| check_expr(env, e))
            .collect(),
        Expr::BinOp(_, a, b) | Expr::Pipe(a, b) => {
            let mut errs = check_expr(env, a);
            errs.extend(check_expr(env, b));
            errs
        }
        Expr::UnOp(_, a) => check_expr(env, a),
        Expr::Call(f, args) => {
            let mut errs = check_expr(env, f);
            for a in args {
                errs.extend(check_expr(env, a));
            }
            errs
        }
        Expr::If(c, ts, te, ec) => {
            let mut errs = check_expr(env, c);
            errs.extend(check_stmts(env, ts));
            errs.extend(check_expr(env, te));
            if let Some((es, ee)) = ec {
                errs.extend(check_stmts(env, es));
                errs.extend(check_expr(env, ee));
            }
            errs
        }
        Expr::Match(sc, arms) => {
            let mut errs = check_expr(env, sc);
            for arm in arms {
                errs.extend(check_arm(env, arm));
            }
            errs
        }
        Expr::For(v, iter, stmts, fe) => {
            let mut env2 = env.clone();
            env2.insert(v.clone());
            let mut errs = check_expr(env, iter);
            errs.extend(check_stmts(&env2, stmts));
            let env_final = stmts_bound(&env2, stmts);
            if let Some(e) = fe {
                errs.extend(check_expr(&env_final, e));
            }
            errs
        }
        Expr::While(c, stmts, fe) => {
            let mut errs = check_expr(env, c);
            errs.extend(check_stmts(env, stmts));
            let env_final = stmts_bound(env, stmts);
            if let Some(e) = fe {
                errs.extend(check_expr(&env_final, e));
            }
            errs
        }
        Expr::Block(stmts, fe) => {
            let mut errs = check_stmts(env, stmts);
            let env_final = stmts_bound(env, stmts);
            errs.extend(check_expr(&env_final, fe));
            errs
        }
        Expr::Break(me) => me.as_ref().map_or(vec![], |e| check_expr(env, e)),
        Expr::Lambda {
            params,
            stmts,
            final_expr,
            ..
        } => {
            let mut env2 = env.clone();
            for p in params {
                env2.insert(p.name.clone());
            }
            let mut errs = check_stmts(&env2, stmts);
            let env_final = stmts_bound(&env2, stmts);
            errs.extend(check_expr(&env_final, final_expr));
            errs
        }
        Expr::LoadRon(p, _) => check_expr(env, p),
        Expr::SaveRon(d, p) => {
            let mut errs = check_expr(env, d);
            errs.extend(check_expr(env, p));
            errs
        }
        _ => vec![],
    }
}

fn check_arm(env: &HashSet<String>, arm: &MatchArm) -> Vec<SemaError> {
    let env2 = extend_with_pat(env, &arm.pat);
    let mut errs = Vec::new();
    if let Some(g) = &arm.guard {
        errs.extend(check_expr(&env2, g));
    }
    errs.extend(check_expr(&env2, &arm.body));
    errs
}

fn extend_with_pat(env: &HashSet<String>, pat: &Pat) -> HashSet<String> {
    let mut env = env.clone();
    match pat {
        Pat::Var(n) => {
            env.insert(n.clone());
        }
        Pat::Tuple(ps) => {
            for p in ps {
                env = extend_with_pat(&env, p);
            }
        }
        Pat::Enum(_, Some(p)) => {
            env = extend_with_pat(&env, p);
        }
        _ => {}
    }
    env
}

fn stmts_bound(env: &HashSet<String>, stmts: &[Stmt]) -> HashSet<String> {
    let mut env = env.clone();
    for s in stmts {
        match s {
            Stmt::Bind(n, _) | Stmt::StructDef(n, _) | Stmt::EnumDef(n, _) => {
                env.insert(n.clone());
            }
            _ => {}
        }
    }
    env
}

// ─── 3. Mutual recursion detection ──────────────────────────

fn check_mutual_rec(prog: &Program) -> Vec<SemaError> {
    let mut call_graph: HashMap<String, Vec<String>> = HashMap::new();
    for s in prog {
        if let Stmt::Bind(n, Expr::Lambda { final_expr, .. }) = s {
            let calls = free_calls_in(n, final_expr);
            call_graph.insert(n.clone(), calls);
        }
    }
    let mut errs = Vec::new();
    for n in call_graph.keys() {
        let callees = &call_graph[n];
        for c in callees {
            if let Some(their_calls) = call_graph.get(c) {
                if their_calls.contains(n) {
                    errs.push(SemaError::MutualRec(n.clone(), c.clone()));
                }
            }
        }
    }
    errs
}

fn free_calls_in(self_name: &str, expr: &Expr) -> Vec<String> {
    collect_calls(expr)
        .into_iter()
        .filter(|n| n != self_name)
        .collect()
}

fn collect_calls(expr: &Expr) -> Vec<String> {
    match expr {
        Expr::Call(f, args) => {
            let mut calls = Vec::new();
            if let Expr::Var(n) = f.as_ref() {
                calls.push(n.clone());
            }
            for a in args {
                calls.extend(collect_calls(a));
            }
            calls
        }
        Expr::BinOp(_, a, b) | Expr::Pipe(a, b) => {
            let mut c = collect_calls(a);
            c.extend(collect_calls(b));
            c
        }
        Expr::UnOp(_, a) => collect_calls(a),
        Expr::If(c, ts, te, ec) => {
            let mut calls = collect_calls(c);
            for s in ts {
                calls.extend(collect_stmt_calls(s));
            }
            calls.extend(collect_calls(te));
            if let Some((es, ee)) = ec {
                for s in es {
                    calls.extend(collect_stmt_calls(s));
                }
                calls.extend(collect_calls(ee));
            }
            calls
        }
        Expr::Match(sc, arms) => {
            let mut calls = collect_calls(sc);
            for arm in arms {
                calls.extend(collect_calls(&arm.body));
            }
            calls
        }
        Expr::For(_, it, ss, fe) => {
            let mut calls = collect_calls(it);
            for s in ss {
                calls.extend(collect_stmt_calls(s));
            }
            if let Some(e) = fe {
                calls.extend(collect_calls(e));
            }
            calls
        }
        Expr::While(c, ss, fe) => {
            let mut calls = collect_calls(c);
            for s in ss {
                calls.extend(collect_stmt_calls(s));
            }
            if let Some(e) = fe {
                calls.extend(collect_calls(e));
            }
            calls
        }
        Expr::Lambda {
            stmts, final_expr, ..
        } => {
            let mut calls = Vec::new();
            for s in stmts {
                calls.extend(collect_stmt_calls(s));
            }
            calls.extend(collect_calls(final_expr));
            calls
        }
        Expr::Block(ss, fe) => {
            let mut calls = Vec::new();
            for s in ss {
                calls.extend(collect_stmt_calls(s));
            }
            calls.extend(collect_calls(fe));
            calls
        }
        _ => vec![],
    }
}

fn collect_stmt_calls(stmt: &Stmt) -> Vec<String> {
    match stmt {
        Stmt::Bind(_, e) | Stmt::Expression(e) => collect_calls(e),
        _ => vec![],
    }
}
