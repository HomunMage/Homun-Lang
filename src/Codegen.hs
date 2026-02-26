-- | Code generator: walks the Homun AST and emits Rust source text.
--
-- Philosophy: Homun is a *template-instantiation* language.
-- We emit textual Rust and let rustc do the monomorphization.
-- Every Homun construct maps 1-to-1 to a Rust construct.
module Codegen
  ( codegenProgram
  ) where

import AST
import Data.List (intercalate)
import qualified Data.Set as Set

-- ─────────────────────────────────────────
-- Context threaded through codegen
-- ─────────────────────────────────────────

type Indent = Int
type Scope  = Set.Set Name

ind :: Indent -> String
ind n = replicate (n * 4) ' '

-- ─────────────────────────────────────────
-- Entry point
-- ─────────────────────────────────────────

codegenProgram :: Program -> String
codegenProgram = unlines . map (codegenTopLevel 0)

codegenTopLevel :: Indent -> Stmt -> String
codegenTopLevel i s = case s of
  SUse ["std"] ->
    "include!(\"std.rs\");"
  SUse path ->
    "use " ++ intercalate "::" path ++ ";"

  SStructDef name fields ->
    unlines $ [ "#[derive(Debug, Clone)]"
              , "pub struct " ++ name ++ " {" ]
              ++ map (codegenField (i+1)) fields ++ [ "}" ]

  SEnumDef name variants ->
    unlines $ [ "#[derive(Debug, Clone)]"
              , "pub enum " ++ name ++ " {" ]
              ++ map (codegenVariant (i+1)) variants ++ [ "}" ]

  SBind name (ELambda params retTy voidMark stmts fe) ->
    codegenFn i name params retTy voidMark stmts fe

  SBind name expr ->
    "pub const " ++ toUpper name ++ ": _ = " ++ cgExpr i Set.empty expr ++ ";"

  SExprStmt e ->
    cgExpr i Set.empty e ++ ";"

codegenField :: Indent -> FieldDef -> String
codegenField i (FieldDef name ty) =
  ind i ++ "pub " ++ name ++ ": " ++ maybe "_" codegenType ty ++ ","

codegenVariant :: Indent -> VariantDef -> String
codegenVariant i (VariantDef name Nothing)   = ind i ++ name ++ ","
codegenVariant i (VariantDef name (Just ty)) = ind i ++ name ++ "(" ++ codegenType ty ++ "),"

-- ─────────────────────────────────────────
-- Functions
-- ─────────────────────────────────────────

codegenFn :: Indent -> Name -> [Param] -> Maybe TypeExpr -> Maybe TypeExpr
          -> [Stmt] -> Expr -> String
codegenFn i name params retTy voidMark stmts fe =
  let scope0   = Set.fromList [ n | Param n _ <- params ]
      paramStr = codegenParamsMut params
      retStr   = case voidMark of
                   Just _  -> ""
                   Nothing -> maybe "" ((" -> " ++) . codegenType) retTy
      generics = inferGenerics params
      genStr   = if null generics then ""
                 else "<" ++ intercalate ", " generics ++ ">"
      bodyLines = cgBody (i+1) scope0 stmts fe
  in unlines $ [ "pub fn " ++ name ++ genStr ++ "(" ++ paramStr ++ ")" ++ retStr ++ " {" ]
               ++ bodyLines ++ [ "}" ]

-- ─────────────────────────────────────────
-- Body / statement codegen with scope
-- ─────────────────────────────────────────

cgBody :: Indent -> Scope -> [Stmt] -> Expr -> [String]
cgBody i scope stmts fe =
  let (lines_, scope') = cgStmts i scope stmts
  in lines_ ++ [ind i ++ cgExpr i scope' fe]

cgStmts :: Indent -> Scope -> [Stmt] -> ([String], Scope)
cgStmts _ scope []     = ([], scope)
cgStmts i scope (s:ss) =
  let (line, scope')  = cgStmt i scope s
      (rest, scope'') = cgStmts i scope' ss
  in (line : rest, scope'')

cgStmt :: Indent -> Scope -> Stmt -> (String, Scope)
cgStmt i scope s = case s of
  SBind name (ELambda params _ _ stmts fe) ->
    let paramStr   = intercalate ", " (map codegenParam params)
        innerScope = foldr (\(Param n _) sc -> Set.insert n sc) scope params
        bodyLines  = cgBody (i+1) innerScope stmts fe
        line = ind i ++ "let " ++ name ++ " = |" ++ paramStr ++ "| {\n"
               ++ unlines bodyLines ++ ind i ++ "};"
    in (line, Set.insert name scope)

  SBind name expr ->
    let rhs = cgExpr i scope expr
    in if Set.member name scope
         then (ind i ++ name ++ " = " ++ rhs ++ ";", scope)
         else (ind i ++ "let mut " ++ name ++ " = " ++ rhs ++ ";", Set.insert name scope)

  SUse ["std"] ->
    (ind i ++ "include!(\"std.rs\");", scope)
  SUse path ->
    (ind i ++ "use " ++ intercalate "::" path ++ ";", scope)

  SStructDef name fields ->
    let fieldLines = map (codegenField (i+1)) fields
        line = unlines $ [ ind i ++ "#[derive(Debug,Clone)]"
                         , ind i ++ "struct " ++ name ++ " {" ]
                         ++ fieldLines ++ [ind i ++ "}"]
    in (line, Set.insert name scope)

  SEnumDef name variants ->
    let varLines = map (codegenVariant (i+1)) variants
        line = unlines $ [ ind i ++ "#[derive(Debug,Clone)]"
                         , ind i ++ "enum " ++ name ++ " {" ]
                         ++ varLines ++ [ind i ++ "}"]
    in (line, Set.insert name scope)

  SExprStmt e ->
    (ind i ++ cgExpr i scope e ++ ";", scope)

-- ─────────────────────────────────────────
-- Parameters
-- ─────────────────────────────────────────

codegenParam :: Param -> String
codegenParam (Param "_" _)          = "_: _"
codegenParam (Param name Nothing)   = name ++ ": _"
codegenParam (Param name (Just ty)) = name ++ ": " ++ codegenType ty

codegenParamsMut :: [Param] -> String
codegenParamsMut params =
  intercalate ", " $ snd $ foldl go (generics, []) params
  where
    generics = ["T","U","V","W","X","Y","Z"]
    go (ls, acc) (Param "_" _)       = (ls, acc ++ ["_: _"])
    go (ls, acc) (Param n Nothing)   = (tail ls, acc ++ ["mut " ++ n ++ ": " ++ head ls])
    go (ls, acc) (Param n (Just ty)) = (ls, acc ++ ["mut " ++ n ++ ": " ++ codegenType ty])

inferGenerics :: [Param] -> [String]
inferGenerics params =
  let n = length [ () | Param _ Nothing <- params ]
  in map (++ ": Clone") (take n ["T","U","V","W","X","Y","Z"])

-- ─────────────────────────────────────────
-- Expressions (scope-aware)
-- ─────────────────────────────────────────

cgExpr :: Indent -> Scope -> Expr -> String
cgExpr i sc expr = case expr of
  EInt n    -> show n
  EFloat n  -> show n ++ "f32"
  EBool b   -> if b then "true" else "false"
  ENone     -> "None"
  EString s -> codegenString s
  EVar "_"  -> "_"
  EVar "str" -> "str_of"
  EVar n    -> n

  EField e field  -> cgExpr i sc e ++ "." ++ field
  EIndex e idx    -> cgExpr i sc e ++ ".homun_idx(" ++ cgExpr i sc idx ++ ")"

  ESlice e start end step ->
    "slice!(" ++ cgExpr i sc e
    ++ ", " ++ maybe "0" (cgExpr i sc) start
    ++ ", " ++ maybe "i64::MAX" (cgExpr i sc) end
    ++ ", " ++ maybe "1" (cgExpr i sc) step ++ ")"

  EList items  -> "vec![" ++ commas items ++ "]"
  EDict pairs  -> "dict![" ++ intercalate ", " [ cgExpr i sc k ++ " => " ++ cgExpr i sc v | (k,v) <- pairs ] ++ "]"
  ESet items   -> "set![" ++ commas items ++ "]"
  ETuple items -> "(" ++ commas items ++ ")"

  EStruct (Just name) fields ->
    name ++ " { " ++ intercalate ", " [ n ++ ": " ++ structVal e | (n,e) <- fields ] ++ " }"
  EStruct Nothing fields ->
    "(" ++ intercalate ", " [ cgExpr i sc e | (_,e) <- fields ] ++ ")"

  EBinOp op lhs rhs -> cgBinOp i sc op lhs rhs
  EUnOp OpNot e -> "!" ++ cgExpr i sc e
  EUnOp OpNeg e -> "-" ++ cgExpr i sc e

  EPipe lhs (ECall (EVar fn) args) | fn `elem` homunMacros ->
    fn ++ "!(" ++ cgExpr i sc lhs ++ optArgs args ++ ")"
  EPipe lhs (ECall fn args) ->
    cgExpr i sc fn ++ "(" ++ cgExpr i sc lhs ++ optArgs args ++ ")"
  EPipe lhs rhs ->
    cgExpr i sc rhs ++ "(" ++ cgExpr i sc lhs ++ ")"

  ELambda params _ _ stmts fe ->
    let paramStr   = intercalate ", " (map codegenParam params)
        innerScope = foldr (\(Param n _) s -> Set.insert n s) sc params
        bodyLines  = cgBody (i+1) innerScope stmts fe
    in "|" ++ paramStr ++ "| {\n" ++ unlines bodyLines ++ ind i ++ "}"

  ECall (EVar "print") args -> cgPrint i sc args

  ECall (EVar fn) args | fn `elem` homunMacros ->
    fn ++ "!(" ++ commas args ++ ")"

  ECall fn args ->
    cgExpr i sc fn ++ "(" ++ intercalate ", " (map (cloneArg i sc) args) ++ ")"

  EIf cond ts te ec ->
    let thenLines = cgBody (i+1) sc ts te
        elseStr = case ec of
          Nothing -> ""
          Just (es, ee) ->
            " else {\n" ++ unlines (cgBody (i+1) sc es ee) ++ ind i ++ "}"
    in "if " ++ cgExpr i sc cond ++ " {\n" ++ unlines thenLines ++ ind i ++ "}" ++ elseStr

  EMatch scrut arms ->
    "match " ++ cgExpr i sc scrut ++ " {\n"
    ++ unlines (map (cgArm i sc) arms) ++ ind i ++ "}"

  EBlock stmts fe ->
    let bodyLines = cgBody (i+1) sc stmts fe
    in "{\n" ++ unlines bodyLines ++ ind i ++ "}"

  EFor var iter stmts fe ->
    let scope0 = Set.insert var sc
        (bodyLines, _) = cgStmts (i+1) scope0 stmts
        finalLine = case fe of
          Just (ETuple []) -> []
          Just e           -> [ind (i+1) ++ cgExpr (i+1) scope0 e ++ ";"]
          Nothing          -> []
    in "for " ++ var ++ " in " ++ cgExpr i sc iter ++ " {\n"
       ++ unlines (bodyLines ++ finalLine) ++ ind i ++ "}"

  EWhile cond stmts fe ->
    let (bodyLines, _) = cgStmts (i+1) sc stmts
        finalLine = case fe of
          Just e  -> [ind (i+1) ++ cgExpr (i+1) sc e ++ ";"]
          Nothing -> []
    in "while " ++ cgExpr i sc cond ++ " {\n"
       ++ unlines (bodyLines ++ finalLine) ++ ind i ++ "}"

  EBreak Nothing  -> "break"
  EBreak (Just e) -> "return " ++ cgExpr i sc e
  EContinue       -> "continue"

  ELoadRon path ty ->
    "ron::from_str::<" ++ codegenType ty ++ ">(&std::fs::read_to_string("
    ++ cgExpr i sc path ++ ").unwrap()).unwrap()"
  ESaveRon d p ->
    "std::fs::write(" ++ cgExpr i sc p ++ ", ron::to_string(&"
    ++ cgExpr i sc d ++ ").unwrap()).unwrap()"

  ERange start end step ->
    case (start, end, step) of
      (Nothing, Just e, Nothing) -> "(0.." ++ cgExpr i sc e ++ ")"
      (Just s, Just e, Nothing)  -> "(" ++ cgExpr i sc s ++ ".." ++ cgExpr i sc e ++ ")"
      (Just s, Just e, Just st)  -> "(" ++ cgExpr i sc s ++ ".." ++ cgExpr i sc e
                                    ++ ").step_by(" ++ cgExpr i sc st ++ " as usize)"
      _                          -> "(0..)"

  where
    commas = intercalate ", " . map (cgExpr i sc)
    optArgs [] = ""
    optArgs as = ", " ++ commas as
    structVal (EString s) = codegenString s ++ ".to_string()"
    structVal e           = cgExpr i sc e

-- ─────────────────────────────────────────
-- Expression helpers
-- ─────────────────────────────────────────

cgBinOp :: Indent -> Scope -> BinOp -> Expr -> Expr -> String
cgBinOp i sc op lhs rhs =
  let l = cgExpr i sc lhs; r = cgExpr i sc rhs
  in case op of
       OpAdd | isListExpr lhs || isListExpr rhs -> "homun_concat(" ++ l ++ ", " ++ r ++ ")"
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
       OpIn    -> "homun_in!(" ++ l ++ ", " ++ r ++ ")"
       OpNotIn -> "!homun_in!(" ++ l ++ ", " ++ r ++ ")"

cgPrint :: Indent -> Scope -> [Expr] -> String
cgPrint i sc args = case args of
  [EString s] ->
    let (fmt, fmtArgs) = parseInterp s
    in if null fmtArgs then "println!(\"" ++ fmt ++ "\")"
       else "println!(\"" ++ fmt ++ "\", " ++ intercalate ", " fmtArgs ++ ")"
  [e] -> "println!(\"{}\", " ++ cgExpr i sc e ++ ")"
  _   -> "println!(" ++ intercalate ", " (map (cgExpr i sc) args) ++ ")"

cgArm :: Indent -> Scope -> MatchArm -> String
cgArm i sc (MatchArm pat guard body) =
  let patS   = cgPat pat
      guardS = maybe "" (\g -> " if " ++ cgExpr i sc g) guard
      bodyS  = case body of
                 EString s -> codegenString s ++ ".to_string()"
                 _         -> cgExpr (i+1) sc body
  in ind (i+1) ++ patS ++ guardS ++ " => " ++ bodyS ++ ","

cgPat :: Pat -> String
cgPat PWild              = "_"
cgPat PNone              = "None"
cgPat (PVar n)           = n
cgPat (PLit e)           = cgExpr 0 Set.empty e
cgPat (PTuple pats)      = "(" ++ intercalate ", " (map cgPat pats) ++ ")"
cgPat (PEnum n Nothing)  = n
cgPat (PEnum n (Just p)) = n ++ "(" ++ cgPat p ++ ")"

-- ─────────────────────────────────────────
-- Types
-- ─────────────────────────────────────────

codegenType :: TypeExpr -> String
codegenType (TName "int")   = "i32"
codegenType (TName "float") = "f32"
codegenType (TName "bool")  = "bool"
codegenType (TName "str")   = "String"
codegenType (TName "none")  = "Option<_>"
codegenType (TName n)       = n
codegenType (TList t)       = "Vec<" ++ codegenType t ++ ">"
codegenType (TDict k v)     = "std::collections::HashMap<" ++ codegenType k ++ ", " ++ codegenType v ++ ">"
codegenType (TSet t)        = "std::collections::HashSet<" ++ codegenType t ++ ">"
codegenType (TOption t)     = "Option<" ++ codegenType t ++ ">"
codegenType (TTuple ts)     = "(" ++ intercalate ", " (map codegenType ts) ++ ")"
codegenType TVoid           = "()"
codegenType TInfer          = "_"

-- ─────────────────────────────────────────
-- String interpolation
-- ─────────────────────────────────────────

codegenString :: String -> String
codegenString s =
  let (fmt, args) = parseInterp s
  in if null args then show s
     else "format!(\"" ++ fmt ++ "\", " ++ intercalate ", " args ++ ")"

parseInterp :: String -> (String, [String])
parseInterp [] = ("", [])
parseInterp ('{':'{':rest) = let (f, a) = parseInterp rest in ("{{" ++ f, a)
parseInterp ('$':'{':rest) =
  let (expr, after) = span (/= '}') rest
      (f, a)        = parseInterp (drop 1 after)
  in ("{}" ++ f, expr : a)
parseInterp (c:rest) =
  let (f, a) = parseInterp rest
      c' = if c == '"' then "\\\"" else [c]
  in (c' ++ f, a)

-- ─────────────────────────────────────────
-- Utilities
-- ─────────────────────────────────────────

toUpper :: String -> String
toUpper = map (\c -> if c >= 'a' && c <= 'z' then toEnum (fromEnum c - 32) else c)

homunMacros :: [String]
homunMacros = ["range", "len", "filter", "map", "reduce", "slice", "dict", "set"]

isListExpr :: Expr -> Bool
isListExpr (EList _)          = True
isListExpr (ESlice _ _ _ _)   = True
isListExpr (EBinOp OpAdd l r) = isListExpr l || isListExpr r
isListExpr _                  = False

cloneArg :: Indent -> Scope -> Expr -> String
cloneArg _ _ (EVar n) = n ++ ".clone()"
cloneArg i sc e       = cgExpr i sc e
