// dep/ast_access.rs — Free-function accessors bridging .hom modules to Rust AST enums.
//
// Pattern: every accessor takes an OWNED value (not a reference), because
// generated .hom code passes all Var-args through clone_arg which emits
// `x.clone()` — producing T by value, not &T.  Taking by value avoids the
// type mismatch that would occur with `&T` signatures.
//
// Phase 1: sema accessors (P1.1)
// Phase 2: codegen accessors (P2.1)

// ─── Kind discriminators ──────────────────────────────────────────────────────

pub fn stmt_kind(s: Stmt) -> String {
    match s {
        Stmt::Bind(_, _) => "Bind".to_string(),
        Stmt::BindPat(_, _) => "BindPat".to_string(),
        Stmt::Assign(_, _) => "Assign".to_string(),
        Stmt::Use(_) => "Use".to_string(),
        Stmt::StructDef(_, _) => "StructDef".to_string(),
        Stmt::EnumDef(_, _) => "EnumDef".to_string(),
        Stmt::Expression(_) => "Expression".to_string(),
    }
}

pub fn expr_kind(e: Expr) -> String {
    match e {
        Expr::Int(_) => "Int".to_string(),
        Expr::Float(_) => "Float".to_string(),
        Expr::Bool(_) => "Bool".to_string(),
        Expr::Str(_) => "Str".to_string(),
        Expr::None => "None".to_string(),
        Expr::Var(_) => "Var".to_string(),
        Expr::Field(_, _) => "Field".to_string(),
        Expr::Index(_, _) => "Index".to_string(),
        Expr::Slice(_, _, _, _) => "Slice".to_string(),
        Expr::List(_) => "List".to_string(),
        Expr::Dict(_) => "Dict".to_string(),
        Expr::Set(_) => "Set".to_string(),
        Expr::Tuple(_) => "Tuple".to_string(),
        Expr::Struct(_, _) => "Struct".to_string(),
        Expr::BinOp(_, _, _) => "BinOp".to_string(),
        Expr::UnOp(_, _) => "UnOp".to_string(),
        Expr::Pipe(_, _) => "Pipe".to_string(),
        Expr::Lambda { .. } => "Lambda".to_string(),
        Expr::Call(_, _) => "Call".to_string(),
        Expr::If(_, _, _, _) => "If".to_string(),
        Expr::Match(_, _) => "Match".to_string(),
        Expr::For(_, _, _, _) => "For".to_string(),
        Expr::While(_, _, _) => "While".to_string(),
        Expr::Block(_, _) => "Block".to_string(),
        Expr::Break(_) => "Break".to_string(),
        Expr::Continue => "Continue".to_string(),
        Expr::LoadRon(_, _) => "LoadRon".to_string(),
        Expr::SaveRon(_, _) => "SaveRon".to_string(),
        Expr::Range(_, _, _) => "Range".to_string(),
        Expr::TryUnwrap(_) => "TryUnwrap".to_string(),
    }
}

pub fn pat_kind(p: Pat) -> String {
    match p {
        Pat::Wild => "Wild".to_string(),
        Pat::Var(_) => "Var".to_string(),
        Pat::Lit(_) => "Lit".to_string(),
        Pat::Tuple(_) => "Tuple".to_string(),
        Pat::Enum(_, _) => "Enum".to_string(),
        Pat::None => "None".to_string(),
    }
}

// ─── Stmt accessors ───────────────────────────────────────────────────────────

/// Returns the name from Stmt::Bind, Stmt::StructDef, or Stmt::EnumDef.
pub fn stmt_bind_name(s: Stmt) -> String {
    match s {
        Stmt::Bind(n, _) => n,
        Stmt::StructDef(n, _) => n,
        Stmt::EnumDef(n, _) => n,
        _ => panic!("stmt_bind_name: not Bind/StructDef/EnumDef"),
    }
}

/// Returns the expression from Stmt::Bind.
pub fn stmt_bind_expr(s: Stmt) -> Expr {
    match s {
        Stmt::Bind(_, e) => e,
        _ => panic!("stmt_bind_expr: not Bind"),
    }
}

/// Returns the pattern from Stmt::BindPat.
pub fn stmt_bindpat_pat(s: Stmt) -> Pat {
    match s {
        Stmt::BindPat(p, _) => p,
        _ => panic!("stmt_bindpat_pat: not BindPat"),
    }
}

/// Returns the expression from Stmt::BindPat.
pub fn stmt_bindpat_expr(s: Stmt) -> Expr {
    match s {
        Stmt::BindPat(_, e) => e,
        _ => panic!("stmt_bindpat_expr: not BindPat"),
    }
}

/// Returns the lhs expression from Stmt::Assign.
pub fn stmt_assign_lhs(s: Stmt) -> Expr {
    match s {
        Stmt::Assign(lhs, _) => lhs,
        _ => panic!("stmt_assign_lhs: not Assign"),
    }
}

/// Returns the rhs expression from Stmt::Assign.
pub fn stmt_assign_rhs(s: Stmt) -> Expr {
    match s {
        Stmt::Assign(_, rhs) => rhs,
        _ => panic!("stmt_assign_rhs: not Assign"),
    }
}

/// Returns the expression from Stmt::Expression.
pub fn stmt_expression_expr(s: Stmt) -> Expr {
    match s {
        Stmt::Expression(e) => e,
        _ => panic!("stmt_expression_expr: not Expression"),
    }
}

// ─── Expr accessors ───────────────────────────────────────────────────────────

/// Returns the variable name from Expr::Var.
pub fn expr_var_name(e: Expr) -> String {
    match e {
        Expr::Var(n) => n,
        _ => panic!("expr_var_name: not Var"),
    }
}

/// Returns the base expression from Expr::Field.
pub fn expr_field_expr(e: Expr) -> Expr {
    match e {
        Expr::Field(base, _) => *base,
        _ => panic!("expr_field_expr: not Field"),
    }
}

/// Returns the field name from Expr::Field.
pub fn expr_field_name(e: Expr) -> String {
    match e {
        Expr::Field(_, name) => name,
        _ => panic!("expr_field_name: not Field"),
    }
}

/// Returns the base expression from Expr::Index.
pub fn expr_index_expr(e: Expr) -> Expr {
    match e {
        Expr::Index(base, _) => *base,
        _ => panic!("expr_index_expr: not Index"),
    }
}

/// Returns the index expression from Expr::Index.
pub fn expr_index_idx(e: Expr) -> Expr {
    match e {
        Expr::Index(_, idx) => *idx,
        _ => panic!("expr_index_idx: not Index"),
    }
}

/// Returns the base expression from Expr::Slice.
pub fn expr_slice_expr(e: Expr) -> Expr {
    match e {
        Expr::Slice(base, _, _, _) => *base,
        _ => panic!("expr_slice_expr: not Slice"),
    }
}

/// Returns the optional `from` bound from Expr::Slice.
pub fn expr_slice_from(e: Expr) -> Option<Expr> {
    match e {
        Expr::Slice(_, from, _, _) => from.map(|x| *x),
        _ => panic!("expr_slice_from: not Slice"),
    }
}

/// Returns the optional `to` bound from Expr::Slice.
pub fn expr_slice_to(e: Expr) -> Option<Expr> {
    match e {
        Expr::Slice(_, _, to, _) => to.map(|x| *x),
        _ => panic!("expr_slice_to: not Slice"),
    }
}

/// Returns the optional step from Expr::Slice.
pub fn expr_slice_step(e: Expr) -> Option<Expr> {
    match e {
        Expr::Slice(_, _, _, step) => step.map(|x| *x),
        _ => panic!("expr_slice_step: not Slice"),
    }
}

/// Returns items from Expr::List.
pub fn expr_list_items(e: Expr) -> Vec<Expr> {
    match e {
        Expr::List(xs) => xs,
        _ => panic!("expr_list_items: not List"),
    }
}

/// Returns items from Expr::Set.
pub fn expr_set_items(e: Expr) -> Vec<Expr> {
    match e {
        Expr::Set(xs) => xs,
        _ => panic!("expr_set_items: not Set"),
    }
}

/// Returns items from Expr::Tuple.
pub fn expr_tuple_items(e: Expr) -> Vec<Expr> {
    match e {
        Expr::Tuple(xs) => xs,
        _ => panic!("expr_tuple_items: not Tuple"),
    }
}

/// Returns pairs from Expr::Dict.
pub fn expr_dict_pairs(e: Expr) -> Vec<(Expr, Expr)> {
    match e {
        Expr::Dict(pairs) => pairs,
        _ => panic!("expr_dict_pairs: not Dict"),
    }
}

/// Returns the optional name from Expr::Struct.
pub fn expr_struct_name(e: Expr) -> Option<String> {
    match e {
        Expr::Struct(name, _) => name,
        _ => panic!("expr_struct_name: not Struct"),
    }
}

/// Returns the field list from Expr::Struct.
pub fn expr_struct_fields(e: Expr) -> Vec<(String, Expr)> {
    match e {
        Expr::Struct(_, fields) => fields,
        _ => panic!("expr_struct_fields: not Struct"),
    }
}

/// Returns the operator string from Expr::BinOp.
pub fn expr_binop_op(e: Expr) -> String {
    match e {
        Expr::BinOp(op, _, _) => format!("{:?}", op),
        _ => panic!("expr_binop_op: not BinOp"),
    }
}

/// Returns the lhs from Expr::BinOp.
pub fn expr_binop_lhs(e: Expr) -> Expr {
    match e {
        Expr::BinOp(_, lhs, _) => *lhs,
        _ => panic!("expr_binop_lhs: not BinOp"),
    }
}

/// Returns the rhs from Expr::BinOp.
pub fn expr_binop_rhs(e: Expr) -> Expr {
    match e {
        Expr::BinOp(_, _, rhs) => *rhs,
        _ => panic!("expr_binop_rhs: not BinOp"),
    }
}

/// Returns the operator string from Expr::UnOp.
pub fn expr_unop_op(e: Expr) -> String {
    match e {
        Expr::UnOp(op, _) => format!("{:?}", op),
        _ => panic!("expr_unop_op: not UnOp"),
    }
}

/// Returns the operand from Expr::UnOp.
pub fn expr_unop_expr(e: Expr) -> Expr {
    match e {
        Expr::UnOp(_, a) => *a,
        _ => panic!("expr_unop_expr: not UnOp"),
    }
}

/// Returns the lhs from Expr::Pipe.
pub fn expr_pipe_lhs(e: Expr) -> Expr {
    match e {
        Expr::Pipe(lhs, _) => *lhs,
        _ => panic!("expr_pipe_lhs: not Pipe"),
    }
}

/// Returns the rhs from Expr::Pipe.
pub fn expr_pipe_rhs(e: Expr) -> Expr {
    match e {
        Expr::Pipe(_, rhs) => *rhs,
        _ => panic!("expr_pipe_rhs: not Pipe"),
    }
}

/// Returns the function expression from Expr::Call.
pub fn expr_call_func(e: Expr) -> Expr {
    match e {
        Expr::Call(f, _) => *f,
        _ => panic!("expr_call_func: not Call"),
    }
}

/// Returns the argument list from Expr::Call.
pub fn expr_call_args(e: Expr) -> Vec<Expr> {
    match e {
        Expr::Call(_, args) => args,
        _ => panic!("expr_call_args: not Call"),
    }
}

/// Returns the condition from Expr::If.
pub fn expr_if_cond(e: Expr) -> Expr {
    match e {
        Expr::If(cond, _, _, _) => *cond,
        _ => panic!("expr_if_cond: not If"),
    }
}

/// Returns the then-branch statements from Expr::If.
pub fn expr_if_then_stmts(e: Expr) -> Vec<Stmt> {
    match e {
        Expr::If(_, stmts, _, _) => stmts,
        _ => panic!("expr_if_then_stmts: not If"),
    }
}

/// Returns the then-branch final expression from Expr::If.
pub fn expr_if_then_expr(e: Expr) -> Expr {
    match e {
        Expr::If(_, _, te, _) => *te,
        _ => panic!("expr_if_then_expr: not If"),
    }
}

/// Returns `true` if Expr::If has an else-branch.
pub fn expr_if_has_else(e: Expr) -> bool {
    match e {
        Expr::If(_, _, _, ec) => ec.is_some(),
        _ => panic!("expr_if_has_else: not If"),
    }
}

/// Returns the else-branch statements from Expr::If (panics if absent).
pub fn expr_if_else_stmts(e: Expr) -> Vec<Stmt> {
    match e {
        Expr::If(_, _, _, Some((stmts, _))) => stmts,
        Expr::If(_, _, _, None) => panic!("expr_if_else_stmts: no else branch"),
        _ => panic!("expr_if_else_stmts: not If"),
    }
}

/// Returns the else-branch final expression from Expr::If (panics if absent).
pub fn expr_if_else_expr(e: Expr) -> Expr {
    match e {
        Expr::If(_, _, _, Some((_, ee))) => *ee,
        Expr::If(_, _, _, None) => panic!("expr_if_else_expr: no else branch"),
        _ => panic!("expr_if_else_expr: not If"),
    }
}

/// Returns the scrutinee from Expr::Match.
pub fn expr_match_scrutinee(e: Expr) -> Expr {
    match e {
        Expr::Match(sc, _) => *sc,
        _ => panic!("expr_match_scrutinee: not Match"),
    }
}

/// Returns the arms from Expr::Match.
pub fn expr_match_arms(e: Expr) -> Vec<MatchArm> {
    match e {
        Expr::Match(_, arms) => arms,
        _ => panic!("expr_match_arms: not Match"),
    }
}

/// Returns the loop variable name from Expr::For.
pub fn expr_for_var(e: Expr) -> String {
    match e {
        Expr::For(v, _, _, _) => v,
        _ => panic!("expr_for_var: not For"),
    }
}

/// Returns the iterator expression from Expr::For.
pub fn expr_for_iter(e: Expr) -> Expr {
    match e {
        Expr::For(_, iter, _, _) => *iter,
        _ => panic!("expr_for_iter: not For"),
    }
}

/// Returns the body statements from Expr::For.
pub fn expr_for_stmts(e: Expr) -> Vec<Stmt> {
    match e {
        Expr::For(_, _, stmts, _) => stmts,
        _ => panic!("expr_for_stmts: not For"),
    }
}

/// Returns the optional final expression from Expr::For.
pub fn expr_for_final(e: Expr) -> Option<Expr> {
    match e {
        Expr::For(_, _, _, fe) => fe.map(|x| *x),
        _ => panic!("expr_for_final: not For"),
    }
}

/// Returns the condition from Expr::While.
pub fn expr_while_cond(e: Expr) -> Expr {
    match e {
        Expr::While(cond, _, _) => *cond,
        _ => panic!("expr_while_cond: not While"),
    }
}

/// Returns the body statements from Expr::While.
pub fn expr_while_stmts(e: Expr) -> Vec<Stmt> {
    match e {
        Expr::While(_, stmts, _) => stmts,
        _ => panic!("expr_while_stmts: not While"),
    }
}

/// Returns the optional final expression from Expr::While.
pub fn expr_while_final(e: Expr) -> Option<Expr> {
    match e {
        Expr::While(_, _, fe) => fe.map(|x| *x),
        _ => panic!("expr_while_final: not While"),
    }
}

/// Returns the body statements from Expr::Block.
pub fn expr_block_stmts(e: Expr) -> Vec<Stmt> {
    match e {
        Expr::Block(stmts, _) => stmts,
        _ => panic!("expr_block_stmts: not Block"),
    }
}

/// Returns the final expression from Expr::Block.
pub fn expr_block_final(e: Expr) -> Expr {
    match e {
        Expr::Block(_, fe) => *fe,
        _ => panic!("expr_block_final: not Block"),
    }
}

/// Returns the optional value from Expr::Break.
pub fn expr_break_value(e: Expr) -> Option<Expr> {
    match e {
        Expr::Break(v) => v.map(|x| *x),
        _ => panic!("expr_break_value: not Break"),
    }
}

/// Returns the params from Expr::Lambda.
pub fn expr_lambda_params(e: Expr) -> Vec<Param> {
    match e {
        Expr::Lambda { params, .. } => params,
        _ => panic!("expr_lambda_params: not Lambda"),
    }
}

/// Returns the body statements from Expr::Lambda.
pub fn expr_lambda_stmts(e: Expr) -> Vec<Stmt> {
    match e {
        Expr::Lambda { stmts, .. } => stmts,
        _ => panic!("expr_lambda_stmts: not Lambda"),
    }
}

/// Returns the final expression from Expr::Lambda.
pub fn expr_lambda_final(e: Expr) -> Expr {
    match e {
        Expr::Lambda { final_expr, .. } => *final_expr,
        _ => panic!("expr_lambda_final: not Lambda"),
    }
}

/// Returns the path expression from Expr::LoadRon.
pub fn expr_loadron_path(e: Expr) -> Expr {
    match e {
        Expr::LoadRon(path, _) => *path,
        _ => panic!("expr_loadron_path: not LoadRon"),
    }
}

/// Returns the data expression from Expr::SaveRon.
pub fn expr_saveron_data(e: Expr) -> Expr {
    match e {
        Expr::SaveRon(data, _) => *data,
        _ => panic!("expr_saveron_data: not SaveRon"),
    }
}

/// Returns the path expression from Expr::SaveRon.
pub fn expr_saveron_path(e: Expr) -> Expr {
    match e {
        Expr::SaveRon(_, path) => *path,
        _ => panic!("expr_saveron_path: not SaveRon"),
    }
}

/// Returns the inner expression from Expr::TryUnwrap.
pub fn expr_tryunwrap_expr(e: Expr) -> Expr {
    match e {
        Expr::TryUnwrap(inner) => *inner,
        _ => panic!("expr_tryunwrap_expr: not TryUnwrap"),
    }
}

// ─── Pat accessors ────────────────────────────────────────────────────────────

/// Returns the variable name from Pat::Var.
pub fn pat_var_name(p: Pat) -> String {
    match p {
        Pat::Var(n) => n,
        _ => panic!("pat_var_name: not Var"),
    }
}

/// Returns the sub-patterns from Pat::Tuple.
pub fn pat_tuple_pats(p: Pat) -> Vec<Pat> {
    match p {
        Pat::Tuple(pats) => pats,
        _ => panic!("pat_tuple_pats: not Tuple"),
    }
}

/// Returns the variant name from Pat::Enum.
pub fn pat_enum_name(p: Pat) -> String {
    match p {
        Pat::Enum(name, _) => name,
        _ => panic!("pat_enum_name: not Enum"),
    }
}

/// Returns the optional payload pattern from Pat::Enum.
pub fn pat_enum_payload(p: Pat) -> Option<Pat> {
    match p {
        Pat::Enum(_, payload) => payload.map(|x| *x),
        _ => panic!("pat_enum_payload: not Enum"),
    }
}

/// Returns the literal expression from Pat::Lit.
pub fn pat_lit_expr(p: Pat) -> Expr {
    match p {
        Pat::Lit(e) => e,
        _ => panic!("pat_lit_expr: not Lit"),
    }
}

// ─── MatchArm accessors ───────────────────────────────────────────────────────

/// Returns the pattern from a MatchArm.
pub fn arm_pat(arm: MatchArm) -> Pat {
    arm.pat
}

/// Returns the optional guard expression from a MatchArm.
pub fn arm_guard(arm: MatchArm) -> Option<Expr> {
    arm.guard
}

/// Returns the body expression from a MatchArm.
pub fn arm_body(arm: MatchArm) -> Expr {
    arm.body
}

// ─── Param accessor ───────────────────────────────────────────────────────────

/// Returns the parameter name.
pub fn param_name(p: Param) -> String {
    p.name
}

/// Returns the optional type annotation from a Param.
pub fn param_ty(p: Param) -> Option<TypeExpr> {
    p.ty
}

// ─── Literal value accessors (Phase 2 / codegen) ─────────────────────────────

/// Returns the integer value from Expr::Int.
pub fn expr_int_val(e: Expr) -> i64 {
    match e {
        Expr::Int(n) => n,
        _ => panic!("expr_int_val: not Int"),
    }
}

/// Returns the float value from Expr::Float.
pub fn expr_float_val(e: Expr) -> f64 {
    match e {
        Expr::Float(f) => f,
        _ => panic!("expr_float_val: not Float"),
    }
}

/// Returns the bool value from Expr::Bool.
pub fn expr_bool_val(e: Expr) -> bool {
    match e {
        Expr::Bool(b) => b,
        _ => panic!("expr_bool_val: not Bool"),
    }
}

/// Returns the string value from Expr::Str.
pub fn expr_str_val(e: Expr) -> String {
    match e {
        Expr::Str(s) => s,
        _ => panic!("expr_str_val: not Str"),
    }
}

// ─── Lambda ret_ty / void_mark (Phase 2 / codegen) ───────────────────────────

/// Returns the optional return type annotation from Expr::Lambda.
pub fn expr_lambda_ret_ty(e: Expr) -> Option<TypeExpr> {
    match e {
        Expr::Lambda { ret_ty, .. } => ret_ty,
        _ => panic!("expr_lambda_ret_ty: not Lambda"),
    }
}

/// Returns the optional void marker from Expr::Lambda.
pub fn expr_lambda_void_mark(e: Expr) -> Option<TypeExpr> {
    match e {
        Expr::Lambda { void_mark, .. } => void_mark,
        _ => panic!("expr_lambda_void_mark: not Lambda"),
    }
}

// ─── Range accessors (Phase 2 / codegen) ─────────────────────────────────────

/// Returns the optional start bound from Expr::Range.
pub fn expr_range_start(e: Expr) -> Option<Expr> {
    match e {
        Expr::Range(s, _, _) => s.map(|x| *x),
        _ => panic!("expr_range_start: not Range"),
    }
}

/// Returns the optional end bound from Expr::Range.
pub fn expr_range_end(e: Expr) -> Option<Expr> {
    match e {
        Expr::Range(_, end, _) => end.map(|x| *x),
        _ => panic!("expr_range_end: not Range"),
    }
}

/// Returns the optional step from Expr::Range.
pub fn expr_range_step(e: Expr) -> Option<Expr> {
    match e {
        Expr::Range(_, _, st) => st.map(|x| *x),
        _ => panic!("expr_range_step: not Range"),
    }
}

// ─── LoadRon type accessor (Phase 2 / codegen) ───────────────────────────────

/// Returns the type expression from Expr::LoadRon.
pub fn expr_loadron_type(e: Expr) -> TypeExpr {
    match e {
        Expr::LoadRon(_, ty) => ty,
        _ => panic!("expr_loadron_type: not LoadRon"),
    }
}

// ─── Stmt StructDef / EnumDef / Use accessors (Phase 2 / codegen) ────────────

/// Returns the field list from Stmt::StructDef.
pub fn stmt_structdef_fields(s: Stmt) -> Vec<FieldDef> {
    match s {
        Stmt::StructDef(_, fields) => fields,
        _ => panic!("stmt_structdef_fields: not StructDef"),
    }
}

/// Returns the variant list from Stmt::EnumDef.
pub fn stmt_enumdef_variants(s: Stmt) -> Vec<VariantDef> {
    match s {
        Stmt::EnumDef(_, variants) => variants,
        _ => panic!("stmt_enumdef_variants: not EnumDef"),
    }
}

/// Returns the path from Stmt::Use.
pub fn stmt_use_path(s: Stmt) -> Vec<String> {
    match s {
        Stmt::Use(path) => path,
        _ => panic!("stmt_use_path: not Use"),
    }
}

// ─── FieldDef accessors (Phase 2 / codegen) ──────────────────────────────────

/// Returns the field name from a FieldDef.
pub fn fielddef_name(f: FieldDef) -> String {
    f.name
}

/// Returns the optional type from a FieldDef.
pub fn fielddef_ty(f: FieldDef) -> Option<TypeExpr> {
    f.ty
}

// ─── VariantDef accessors (Phase 2 / codegen) ────────────────────────────────

/// Returns the variant name from a VariantDef.
pub fn variantdef_name(v: VariantDef) -> String {
    v.name
}

/// Returns the optional payload type from a VariantDef.
pub fn variantdef_payload(v: VariantDef) -> Option<TypeExpr> {
    v.payload
}

// ─── TypeExpr accessors (Phase 2 / codegen) ──────────────────────────────────

/// Returns a kind string for a TypeExpr: "Name","List","Dict","Set","Option","Tuple","Generic","Void","Infer".
pub fn type_kind(t: TypeExpr) -> String {
    match t {
        TypeExpr::Name(_) => "Name".to_string(),
        TypeExpr::List(_) => "List".to_string(),
        TypeExpr::Dict(_, _) => "Dict".to_string(),
        TypeExpr::Set(_) => "Set".to_string(),
        TypeExpr::Option(_) => "Option".to_string(),
        TypeExpr::Tuple(_) => "Tuple".to_string(),
        TypeExpr::Generic(_, _) => "Generic".to_string(),
        TypeExpr::Void => "Void".to_string(),
        TypeExpr::Infer => "Infer".to_string(),
    }
}

/// Returns the name string from TypeExpr::Name.
pub fn type_name_val(t: TypeExpr) -> String {
    match t {
        TypeExpr::Name(n) => n,
        _ => panic!("type_name_val: not Name"),
    }
}

/// Returns the inner type from TypeExpr::List.
pub fn type_list_inner(t: TypeExpr) -> TypeExpr {
    match t {
        TypeExpr::List(inner) => *inner,
        _ => panic!("type_list_inner: not List"),
    }
}

/// Returns the key type from TypeExpr::Dict.
pub fn type_dict_key(t: TypeExpr) -> TypeExpr {
    match t {
        TypeExpr::Dict(k, _) => *k,
        _ => panic!("type_dict_key: not Dict"),
    }
}

/// Returns the value type from TypeExpr::Dict.
pub fn type_dict_val(t: TypeExpr) -> TypeExpr {
    match t {
        TypeExpr::Dict(_, v) => *v,
        _ => panic!("type_dict_val: not Dict"),
    }
}

/// Returns the inner type from TypeExpr::Set.
pub fn type_set_inner(t: TypeExpr) -> TypeExpr {
    match t {
        TypeExpr::Set(inner) => *inner,
        _ => panic!("type_set_inner: not Set"),
    }
}

/// Returns the inner type from TypeExpr::Option.
pub fn type_option_inner(t: TypeExpr) -> TypeExpr {
    match t {
        TypeExpr::Option(inner) => *inner,
        _ => panic!("type_option_inner: not Option"),
    }
}

/// Returns the element types from TypeExpr::Tuple.
pub fn type_tuple_items(t: TypeExpr) -> Vec<TypeExpr> {
    match t {
        TypeExpr::Tuple(ts) => ts,
        _ => panic!("type_tuple_items: not Tuple"),
    }
}

/// Returns the base name from TypeExpr::Generic.
pub fn type_generic_name(t: TypeExpr) -> String {
    match t {
        TypeExpr::Generic(n, _) => n,
        _ => panic!("type_generic_name: not Generic"),
    }
}

/// Returns the type parameters from TypeExpr::Generic.
pub fn type_generic_params(t: TypeExpr) -> Vec<TypeExpr> {
    match t {
        TypeExpr::Generic(_, ps) => ps,
        _ => panic!("type_generic_params: not Generic"),
    }
}
