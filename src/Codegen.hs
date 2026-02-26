-- | Code generator: walks the Homun AST and emits Rust source text.
--
-- Philosophy: Homun is a *template-instantiation* language.
-- We emit textual Rust and let rustc do the monomorphization.
-- Every Homun construct maps 1-to-1 to a Rust construct.
module Codegen
  ( codegenProgram
  ) where

import AST
import Data.List (intercalate, isPrefixOf)

-- ─────────────────────────────────────────
-- Indentation helpers
-- ─────────────────────────────────────────

type Indent = Int

ind :: Indent -> String
ind n = replicate (n * 4) ' '

-- ─────────────────────────────────────────
-- Entry point
-- ─────────────────────────────────────────

codegenProgram :: Program -> String
codegenProgram stmts =
  unlines (map (codegenTopLevel 0) stmts)

codegenTopLevel :: Indent -> Stmt -> String
codegenTopLevel i s = case s of
  SUse path ->
    "use " ++ intercalate "::" path ++ ";"

  SStructDef name fields ->
    let fieldLines = map (codegenField (i+1)) fields
    in unlines $
         [ "#[derive(Debug, Clone)]"
         , "pub struct " ++ name ++ " {"
         ] ++ fieldLines ++
         [ "}" ]

  SEnumDef name variants ->
    let varLines = map (codegenVariant (i+1)) variants
    in unlines $
         [ "#[derive(Debug, Clone)]"
         , "pub enum " ++ name ++ " {"
         ] ++ varLines ++
         [ "}" ]

  SBind name expr ->
    -- Top-level bind → function or const
    case expr of
      ELambda params retTy voidMark stmts finalExpr ->
        codegenFn i name params retTy voidMark stmts finalExpr False
      _ ->
        -- constant
        "pub const " ++ toUpper name ++ ": _ = " ++ codegenExpr i expr ++ ";"

  SExprStmt e ->
    codegenExpr i e ++ ";"

codegenField :: Indent -> FieldDef -> String
codegenField i (FieldDef name ty) =
  ind i ++ "pub " ++ name ++ ": " ++ maybe "_" codegenType ty ++ ","

codegenVariant :: Indent -> VariantDef -> String
codegenVariant i (VariantDef name Nothing) =
  ind i ++ name ++ ","
codegenVariant i (VariantDef name (Just ty)) =
  ind i ++ name ++ "(" ++ codegenType ty ++ "),"

-- ─────────────────────────────────────────
-- Functions
-- ─────────────────────────────────────────

codegenFn :: Indent -> Name -> [Param] -> Maybe TypeExpr -> Maybe TypeExpr
          -> [Stmt] -> Expr -> Bool -> String
codegenFn i name params retTy voidMark stmts finalExpr isRec =
  let -- Detect which params are reassigned in the body
      bodyText   = concatMap (codegenStmt (i+1)) stmts ++ codegenExpr (i+1) finalExpr
      paramStr   = codegenParamsMut params stmts finalExpr
      retStr   = case voidMark of
                   Just _  -> ""
                   Nothing -> case retTy of
                     Nothing -> ""
                     Just t  -> " -> " ++ codegenType t
      generics = inferGenerics params retTy
      genStr   = if null generics then "" else "<" ++ intercalate ", " generics ++ ">"
      bodyLines = map (codegenStmt (i+1)) stmts
                  ++ [ind (i+1) ++ codegenExpr (i+1) finalExpr]
  in unlines $
       [ "pub fn " ++ name ++ genStr ++ "(" ++ paramStr ++ ")" ++ retStr ++ " {"
       ] ++ bodyLines ++
       [ "}" ]

codegenParam :: Param -> String
codegenParam (Param name Nothing) =
  -- Will be replaced by T, U, V... from inferGenerics; placeholder here
  name ++ ": _"
codegenParam (Param "_" _) = "_: _"
codegenParam (Param name (Just ty)) =
  name ++ ": " ++ codegenType ty

-- Assign T, U, V... to untyped params; return generic constraints list
inferGenerics :: [Param] -> Maybe TypeExpr -> [String]
inferGenerics params _retTy =
  let untyped = filter (\(Param _ t) -> case t of Nothing -> True; _ -> False) params
      letters = take (length untyped) ["T","U","V","W","X","Y","Z"]
  in map (++ ": Clone") letters

-- Build param string substituting T, U, V for untyped params
codegenParams :: [Param] -> String
codegenParams params =
  let letters = ["T","U","V","W","X","Y","Z"]
      (ps, _) = foldl assign ([], letters) params
  in intercalate ", " (reverse ps)
  where
    assign (acc, ls) (Param name Nothing) =
      let l = head ls
      in (( name ++ ": " ++ l) : acc, tail ls)
    assign (acc, ls) (Param "_" _) = ("_: _" : acc, ls)
    assign (acc, ls) (Param name (Just ty)) =
      ((name ++ ": " ++ codegenType ty) : acc, ls)

-- | Like codegenParams but adds 'mut' to all named params
--   (Homun allows rebinding any variable; #![allow(unused_mut)] suppresses warnings)
codegenParamsMut :: [Param] -> [Stmt] -> Expr -> String
codegenParamsMut params _stmts _finalExpr =
  let letters = ["T","U","V","W","X","Y","Z"]
      (ps, _) = foldl assign ([], letters) params
  in intercalate ", " (reverse ps)
  where
    assign (acc, ls) (Param name Nothing) =
      let l = head ls
      in (("mut " ++ name ++ ": " ++ l) : acc, tail ls)
    assign (acc, ls) (Param "_" _) = ("_: _" : acc, ls)
    assign (acc, ls) (Param name (Just ty)) =
      (("mut " ++ name ++ ": " ++ codegenType ty) : acc, ls)

-- ─────────────────────────────────────────
-- Statements inside functions
-- ─────────────────────────────────────────

-- | Detect re-bind: name appears in the generated RHS (simple substring check)
nameInRhs :: String -> String -> Bool
nameInRhs name rhs = name `isPrefixOf` rhs
                  || any (\pre -> (pre ++ name) `isInfixOf'` rhs)
                         [" ", "(", ", "]
  where isInfixOf' needle haystack = any (isPrefixOf needle) (tails haystack)
        tails [] = [[]]
        tails s@(_:xs) = s : tails xs

codegenStmt :: Indent -> Stmt -> String
codegenStmt i s = case s of
  SBind name expr ->
    case expr of
      ELambda params retTy voidMark stmts finalExpr ->
        -- inner function / closure (use _ for untyped params, not T/U/V)
        let paramStr = intercalate ", " (map codegenParam params)
            retStr   = maybe "" ((" -> " ++) . codegenType) retTy
            bodyLines = map (codegenStmt (i+1)) stmts
                        ++ [ind (i+1) ++ codegenExpr (i+1) finalExpr]
        in ind i ++ "let " ++ name ++ " = |" ++ paramStr ++ "| {"
           ++ "\n" ++ unlines bodyLines ++ ind i ++ "};"
      _ ->
        let rhs = codegenExpr i expr
        in if nameInRhs name rhs
             then ind i ++ name ++ " = " ++ rhs ++ ";"
             else ind i ++ "let mut " ++ name ++ " = " ++ rhs ++ ";"

  SUse path ->
    ind i ++ "use " ++ intercalate "::" path ++ ";"

  SStructDef name fields ->
    -- Inner struct definition
    let fieldLines = map (codegenField (i+1)) fields
    in unlines $
         [ ind i ++ "#[derive(Debug,Clone)]"
         , ind i ++ "struct " ++ name ++ " {"
         ] ++ fieldLines ++ [ind i ++ "}"]

  SEnumDef name variants ->
    let varLines = map (codegenVariant (i+1)) variants
    in unlines $
         [ ind i ++ "#[derive(Debug,Clone)]"
         , ind i ++ "enum " ++ name ++ " {"
         ] ++ varLines ++ [ind i ++ "}"]

  SExprStmt e ->
    ind i ++ codegenExpr i e ++ ";"

-- ─────────────────────────────────────────
-- Expressions
-- ─────────────────────────────────────────

codegenExpr :: Indent -> Expr -> String
codegenExpr i expr = case expr of
  EInt n    -> show n
  EFloat n  -> show n ++ "f32"
  EBool b   -> if b then "true" else "false"
  ENone     -> "None"
  EString s -> codegenString s
  EVar "_"  -> "_"
  EVar "str"    -> "str_of"
  EVar n    -> n

  EField e field ->
    codegenExpr i e ++ "." ++ field

  EIndex e idx ->
    codegenExpr i e ++ ".homun_idx(" ++ codegenExpr i idx ++ ")"

  ESlice e start end step ->
    -- Rust doesn't have Python slices natively; emit a helper call
    let startS = maybe "0" (codegenExpr i) start
        endS   = maybe "i64::MAX" (codegenExpr i) end
        stepS  = maybe "1" (codegenExpr i) step
    in "homun_slice(&" ++ codegenExpr i e ++ ", " ++ startS ++ ", " ++ endS ++ ", " ++ stepS ++ ")"

  EList items ->
    "vec![" ++ intercalate ", " (map (codegenExpr i) items) ++ "]"

  EDict pairs ->
    let kvs = map (\(k,v) -> "(" ++ codegenExpr i k ++ ", " ++ codegenExpr i v ++ ")") pairs
    in "std::collections::HashMap::from([" ++ intercalate ", " kvs ++ "])"

  ESet items ->
    "std::collections::HashSet::from([" ++ intercalate ", " (map (codegenExpr i) items) ++ "])"

  ETuple items ->
    "(" ++ intercalate ", " (map (codegenExpr i) items) ++ ")"

  EStruct (Just name) fields ->
    let fieldStr = intercalate ", " (map (\(n,e) -> n ++ ": " ++ codegenStructField i e) fields)
    in name ++ " { " ++ fieldStr ++ " }"
  EStruct Nothing fields ->
    -- anonymous struct → tuple struct or use a macro; emit as struct literal with unnamed type
    let fieldStr = intercalate ", " (map (\(n,e) -> codegenExpr i e) fields)
    in "(" ++ fieldStr ++ ")"

  EBinOp op lhs rhs ->
    let l = codegenExpr i lhs
        r = codegenExpr i rhs
    in case op of
         OpAdd | isListExpr lhs || isListExpr rhs
                -> "homun_concat(" ++ l ++ ", " ++ r ++ ")"
         OpAdd   -> l ++ " + " ++ r
         OpSub   -> l ++ " - " ++ r
         OpMul   -> l ++ " * " ++ r
         OpDiv   -> l ++ " / " ++ r
         OpMod   -> l ++ " % " ++ r
         OpEq    -> l ++ " == " ++ r
         OpNeq   -> l ++ " != " ++ r
         OpLt    -> l ++ " < " ++ r
         OpGt    -> l ++ " > " ++ r
         OpLe    -> l ++ " <= " ++ r
         OpGe    -> l ++ " >= " ++ r
         OpAnd   -> l ++ " && " ++ r
         OpOr    -> l ++ " || " ++ r
         OpIn    -> r ++ ".contains(&" ++ l ++ ")"
         OpNotIn -> "!" ++ r ++ ".contains(&" ++ l ++ ")"

  EUnOp op e ->
    let s = codegenExpr i e
    in case op of
         OpNot -> "!" ++ s
         OpNeg -> "-" ++ s

  -- Pipe: lhs | f(args)  →  f!(lhs, args) for builtins, f(lhs, args) otherwise
  EPipe lhs (ECall (EVar fn) args) | fn `elem` homunMacros ->
    fn ++ "!(" ++ codegenExpr i lhs ++
    (if null args then "" else ", " ++ intercalate ", " (map (codegenExpr i) args)) ++ ")"
  EPipe lhs (ECall fn args) ->
    codegenExpr i fn ++ "(" ++ codegenExpr i lhs ++
    (if null args then "" else ", " ++ intercalate ", " (map (codegenExpr i) args)) ++ ")"
  EPipe lhs rhs ->
    -- pipe into a bare identifier (0-arg call)
    codegenExpr i rhs ++ "(" ++ codegenExpr i lhs ++ ")"

  ELambda params retTy voidMark stmts finalExpr ->
    let paramStr = intercalate ", " (map codegenParam params)
        bodyLines = map (codegenStmt (i+1)) stmts
                    ++ [ind (i+1) ++ codegenExpr (i+1) finalExpr]
    in "|" ++ paramStr ++ "| {\n" ++ unlines bodyLines ++ ind i ++ "}"

  -- Built-in print: handle interpolated strings specially
  ECall (EVar "print") args ->
    case args of
      [EString s] ->
        let (fmt, fmtArgs) = parseInterp s
        in if null fmtArgs
             then "println!(\"" ++ fmt ++ "\")"
             else "println!(\"" ++ fmt ++ "\", " ++ intercalate ", " fmtArgs ++ ")"
      [e] ->
        "println!(\"{}\", " ++ codegenExpr i e ++ ")"
      _ ->
        "println!(" ++ intercalate ", " (map (codegenExpr i) args) ++ ")"
  -- Built-in macros: range!, len!, filter!, map!, reduce!
  ECall (EVar fn) args | fn `elem` homunMacros ->
    fn ++ "!(" ++ intercalate ", " (map (codegenExpr i) args) ++ ")"
  -- General function call: auto-clone variable arguments
  ECall fn args ->
    codegenExpr i fn ++ "(" ++ intercalate ", " (map (codegenArgClone i) args) ++ ")"

  EIf cond thenStmts thenExpr elseClause ->
    let condS = codegenExpr i cond
        thenLines = map (codegenStmt (i+1)) thenStmts
                    ++ [ind (i+1) ++ codegenExpr (i+1) thenExpr]
        elseStr = case elseClause of
          Nothing -> ""
          Just (es, ee) ->
            let elseLines = map (codegenStmt (i+1)) es
                            ++ [ind (i+1) ++ codegenExpr (i+1) ee]
            in " else {\n" ++ unlines elseLines ++ ind i ++ "}"
    in "if " ++ condS ++ " {\n" ++ unlines thenLines ++ ind i ++ "}" ++ elseStr

  EMatch scrutinee arms ->
    let armStrs = map (codegenArm i) arms
    in "match " ++ codegenExpr i scrutinee ++ " {\n"
       ++ unlines armStrs
       ++ ind i ++ "}"

  EFor "__block__" _ stmts finalExpr ->
    let bodyLines = map (codegenStmt (i+1)) stmts
                    ++ case finalExpr of
                         Just e  -> [ind (i+1) ++ codegenExpr (i+1) e]
                         Nothing -> []
    in "{\n" ++ unlines bodyLines ++ ind i ++ "}"

  EFor var iter stmts finalExpr ->
    let varS  = var
        iterS = codegenExpr i iter
        -- Suppress trivial `true;` from splitBlock when last stmt was a SBind
        finalLine = case finalExpr of
          Just (EBool True) -> []
          Just e            -> [ind (i+1) ++ codegenExpr (i+1) e ++ ";"]
          Nothing           -> []
        bodyLines = map (codegenStmt (i+1)) stmts ++ finalLine
    in "for " ++ varS ++ " in " ++ iterS ++ " {\n"
       ++ unlines bodyLines ++ ind i ++ "}"

  EWhile cond stmts finalExpr ->
    let condS = codegenExpr i cond
        bodyLines = map (codegenStmt (i+1)) stmts
                    ++ case finalExpr of
                         Just e  -> [ind (i+1) ++ codegenExpr (i+1) e ++ ";"]
                         Nothing -> []
    in "while " ++ condS ++ " {\n" ++ unlines bodyLines ++ ind i ++ "}"

  EBreak Nothing  -> "break"
  EBreak (Just e) -> "return " ++ codegenExpr i e

  EContinue -> "continue"

  ELoadRon path ty ->
    "ron::from_str::<" ++ codegenType ty ++ ">(&std::fs::read_to_string("
    ++ codegenExpr i path ++ ").unwrap()).unwrap()"

  ESaveRon data_ path ->
    "std::fs::write(" ++ codegenExpr i path ++ ", ron::to_string(&"
    ++ codegenExpr i data_ ++ ").unwrap()).unwrap()"

  ERange start end step ->
    case (start, end, step) of
      (Nothing, Just e, Nothing) ->
        "(0.." ++ codegenExpr i e ++ ")"
      (Just s, Just e, Nothing) ->
        "(" ++ codegenExpr i s ++ ".." ++ codegenExpr i e ++ ")"
      (Just s, Just e, Just st) ->
        "(" ++ codegenExpr i s ++ ".." ++ codegenExpr i e ++ ").step_by(" ++ codegenExpr i st ++ " as usize)"
      _ -> "(0..)"

codegenArm :: Indent -> MatchArm -> String
codegenArm i (MatchArm pat guard body) =
  let patS   = codegenPat pat
      guardS = maybe "" (\g -> " if " ++ codegenExpr i g) guard
      bodyS  = case body of
                 EString s -> codegenString s ++ ".to_string()"
                 _         -> codegenExpr (i+1) body
  in ind (i+1) ++ patS ++ guardS ++ " => " ++ bodyS ++ ","

codegenPat :: Pat -> String
codegenPat PWild           = "_"
codegenPat PNone           = "None"
codegenPat (PVar n)        = n
codegenPat (PLit e)        = codegenExpr 0 e
codegenPat (PTuple pats)   = "(" ++ intercalate ", " (map codegenPat pats) ++ ")"
codegenPat (PEnum n Nothing)  = n
codegenPat (PEnum n (Just p)) = n ++ "(" ++ codegenPat p ++ ")"

-- ─────────────────────────────────────────
-- Types
-- ─────────────────────────────────────────

codegenType :: TypeExpr -> String
codegenType (TName "int")    = "i32"
codegenType (TName "float")  = "f32"
codegenType (TName "bool")   = "bool"
codegenType (TName "str")    = "String"
codegenType (TName "none")   = "Option<_>"
codegenType (TName n)        = n
codegenType (TList t)        = "Vec<" ++ codegenType t ++ ">"
codegenType (TDict k v)      = "std::collections::HashMap<" ++ codegenType k ++ ", " ++ codegenType v ++ ">"
codegenType (TSet t)         = "std::collections::HashSet<" ++ codegenType t ++ ">"
codegenType (TOption t)      = "Option<" ++ codegenType t ++ ">"
codegenType (TTuple ts)      = "(" ++ intercalate ", " (map codegenType ts) ++ ")"
codegenType TVoid            = "()"
codegenType TInfer           = "_"

-- ─────────────────────────────────────────
-- String interpolation
-- ─────────────────────────────────────────

-- | Homun  "Hello, ${name}!"  →  Rust  format!("Hello, {}!", name)
codegenString :: String -> String
codegenString s =
  let (fmt, args) = parseInterp s
  in if null args
       then show s  -- no interpolation, plain string literal
       else "format!(\"" ++ fmt ++ "\", " ++ intercalate ", " args ++ ")"

-- | Parse interpolated string into format string + argument list
parseInterp :: String -> (String, [String])
parseInterp [] = ("", [])
parseInterp ('{':'{':rest) = -- escaped brace
  let (f, a) = parseInterp rest in ("{{" ++ f, a)
parseInterp ('$':'{':rest) =
  let (expr, after) = span (/= '}') rest
      rest'         = drop 1 after  -- skip closing }
      (f, a)        = parseInterp rest'
  in ("{}" ++ f, expr : a)
parseInterp (c:rest) =
  let (f, a) = parseInterp rest
      c' = if c == '"' then "\\\"" else [c]
  in (c' ++ f, a)

-- ─────────────────────────────────────────
-- Utilities
-- ─────────────────────────────────────────

toUpper :: String -> String
toUpper []     = []
toUpper (c:cs) = (if c >= 'a' && c <= 'z' then toEnum (fromEnum c - 32) else c) : toUpper cs

-- | Homun built-in names that emit as Rust macros (name!)
homunMacros :: [String]
homunMacros = ["range", "len", "filter", "map", "reduce"]

-- | Clone variable arguments to preserve Homun's value semantics in Rust.
--   Primitives (i32, f32, bool) are Copy so .clone() is a no-op.
--   Collections (Vec, HashMap) get properly cloned.
codegenArgClone :: Indent -> Expr -> String
codegenArgClone i (EVar n) = n ++ ".clone()"
codegenArgClone i e = codegenExpr i e

-- | Codegen a struct field value — auto .to_string() for string literals
codegenStructField :: Indent -> Expr -> String
codegenStructField i (EString s) = codegenString s ++ ".to_string()"
codegenStructField i e = codegenExpr i e

-- | Is this a string literal?
isStringLit :: Expr -> Bool
isStringLit (EString _) = True
isStringLit _ = False

-- | Heuristic: does this expression produce a Vec?
isListExpr :: Expr -> Bool
isListExpr (EList _) = True
isListExpr (ESlice _ _ _ _) = True
isListExpr (ECall (EVar "filter") _) = True
isListExpr (ECall (EVar "map") _) = True
isListExpr (EPipe _ (ECall (EVar "filter") _)) = True
isListExpr (EPipe _ (ECall (EVar "map") _)) = True
isListExpr (EBinOp OpAdd l r) = isListExpr l || isListExpr r
isListExpr _ = False