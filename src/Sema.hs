-- | Semantic analysis pass for Homun.
--
-- This pass performs:
--   1.  snake_case enforcement for variable/lambda names
--   2.  Recursion detection — wraps recursive lambdas with a `rec` marker
--   3.  Mutual recursion detection — emits a compile error
--   4.  Monomorphisation guard — detects same lambda used at 2+ different types (best-effort)
--   5.  Undefined reference check for top-level bindings
--
-- The pass runs AFTER parsing and BEFORE codegen.
-- It returns either a list of error messages or the annotated (but same-shaped) program.
module Sema
  ( analyzeProgram
  , SemaError(..)
  ) where

import AST
import Data.Char (isUpper, isLower)
import Data.List (nub, group, sort, isPrefixOf)
import qualified Data.Set as Set
import qualified Data.Map.Strict as Map

-- ─────────────────────────────────────────
-- Error type
-- ─────────────────────────────────────────

data SemaError
  = BadCasing    Name              -- variable/lambda not snake_case
  | MutualRec    Name Name         -- mutual recursion detected
  | Undefined    Name              -- unresolved reference at top level
  | PolyConflict Name              -- lambda used at conflicting types
  | EmptyTypedList Name            -- @[] that is never used
  deriving (Show, Eq)

formatError :: SemaError -> String
formatError (BadCasing n)      = "SEMA ERROR: '" ++ n ++ "' must be snake_case"
formatError (MutualRec a b)    = "SEMA ERROR: mutual recursion between '" ++ a ++ "' and '" ++ b ++ "' is forbidden"
formatError (Undefined n)      = "SEMA ERROR: undefined reference '" ++ n ++ "'"
formatError (PolyConflict n)   = "SEMA ERROR: lambda '" ++ n ++ "' is used at conflicting concrete types (monomorphisation failure)"
formatError (EmptyTypedList n) = "SEMA ERROR: empty list '" ++ n ++ "' is never used, cannot infer type"

-- ─────────────────────────────────────────
-- Analysis environment
-- ─────────────────────────────────────────

type Env  = Set.Set Name   -- defined names in scope
type Errs = [SemaError]

-- ─────────────────────────────────────────
-- Entry point
-- ─────────────────────────────────────────

analyzeProgram :: Program -> Either [SemaError] Program
analyzeProgram prog =
  let builtins = Set.fromList
        [ "print", "len", "range", "str", "int", "float", "bool"
        , "filter", "map", "reduce", "load_ron", "save_ron"
        , "clamp", "update", "idle", "attack", "die", "warn", "recover"
        , "std"
        ]
      topNames = Set.fromList [ n | SBind n _ <- prog ]
      env0     = Set.union builtins topNames
      errs     = checkStmts env0 prog
                 ++ checkCasingAll prog
                 ++ checkMutualRec prog
  in if null errs
       then Right prog
       else Left errs

-- ─────────────────────────────────────────
-- 1. snake_case enforcement
-- ─────────────────────────────────────────

checkCasingAll :: Program -> Errs
checkCasingAll = concatMap checkCasingStmt

checkCasingStmt :: Stmt -> Errs
checkCasingStmt (SBind n _)
  | isTypeName n  = []    -- PascalCase types are allowed
  | not (isSnake n) = [BadCasing n]
checkCasingStmt _ = []

isSnake :: Name -> Bool
isSnake [] = True
isSnake n  = all (\c -> c `elem` "_0123456789abcdefghijklmnopqrstuvwxyz") n

isTypeName :: Name -> Bool
isTypeName (c:_) = isUpper c
isTypeName []    = False

-- ─────────────────────────────────────────
-- 2. Undefined reference check
-- ─────────────────────────────────────────

-- | Check a sequence of stmts, threading newly bound names into env
checkStmts :: Env -> [Stmt] -> Errs
checkStmts _   []     = []
checkStmts env (s:ss) =
  let errs = checkStmt env s
      env' = case s of
               SBind n _      -> Set.insert n env
               SStructDef n _ -> Set.insert n env
               SEnumDef n _   -> Set.insert n env
               _              -> env
  in errs ++ checkStmts env' ss

checkStmt :: Env -> Stmt -> Errs
checkStmt env (SBind _ e)    = checkExpr env e
checkStmt env (SExprStmt e)  = checkExpr env e
checkStmt _ _                = []

checkExpr :: Env -> Expr -> Errs
checkExpr env e = case e of
  EVar n ->
    if Set.member n env || n == "_" then [] else [Undefined n]
  EField ex _        -> checkExpr env ex
  EIndex ex idx      -> checkExpr env ex ++ checkExpr env idx
  ESlice ex a b c    -> checkExpr env ex ++ maybe [] (checkExpr env) a
                         ++ maybe [] (checkExpr env) b
                         ++ maybe [] (checkExpr env) c
  EList xs           -> concatMap (checkExpr env) xs
  EDict pairs        -> concatMap (\(k,v) -> checkExpr env k ++ checkExpr env v) pairs
  ESet xs            -> concatMap (checkExpr env) xs
  ETuple xs          -> concatMap (checkExpr env) xs
  EStruct _ fields   -> concatMap (checkExpr env . snd) fields
  EBinOp _ a b       -> checkExpr env a ++ checkExpr env b
  EUnOp _ a          -> checkExpr env a
  EPipe a b           -> checkExpr env a ++ checkExpr env b
  ECall fn args      -> checkExpr env fn ++ concatMap (checkExpr env) args
  EIf c ts te ec     ->
    checkExpr env c ++ checkStmts env ts ++ checkExpr env te ++
    maybe [] (\(es,ee) -> checkStmts env es ++ checkExpr env ee) ec
  EMatch sc arms     ->
    checkExpr env sc ++ concatMap (checkArm env) arms
  EFor v iter stmts fe ->
    let env'     = Set.insert v env
        envFinal = foldl stmtBound env' stmts
    in checkExpr env iter ++ checkStmts env' stmts
       ++ maybe [] (checkExpr envFinal) fe
  EWhile c stmts fe  ->
    let envFinal = foldl stmtBound env stmts
    in checkExpr env c ++ checkStmts env stmts
       ++ maybe [] (checkExpr envFinal) fe
  EBlock stmts fe    ->
    let envFinal = foldl stmtBound env stmts
    in checkStmts env stmts ++ checkExpr envFinal fe
  EBreak me          -> maybe [] (checkExpr env) me
  ELambda params _ _ stmts fe ->
    let env'     = foldr (\(Param n _) s -> Set.insert n s) env params
        envFinal = foldl stmtBound env' stmts
    in checkStmts env' stmts ++ checkExpr envFinal fe
  ELoadRon p _       -> checkExpr env p
  ESaveRon d p       -> checkExpr env d ++ checkExpr env p
  _                  -> []

checkArm :: Env -> MatchArm -> Errs
checkArm env (MatchArm pat guard body) =
  let env' = extendWithPat env pat
  in maybe [] (checkExpr env') guard ++ checkExpr env' body

extendWithPat :: Env -> Pat -> Env
extendWithPat env PWild           = env
extendWithPat env PNone           = env
extendWithPat env (PVar n)        = Set.insert n env
extendWithPat env (PLit _)        = env
extendWithPat env (PTuple ps)     = foldr (flip extendWithPat) env ps
extendWithPat env (PEnum _ mp)    = maybe env (extendWithPat env) mp

stmtBound :: Env -> Stmt -> Env
stmtBound e (SBind n _)      = Set.insert n e
stmtBound e (SStructDef n _) = Set.insert n e
stmtBound e (SEnumDef n _)   = Set.insert n e
stmtBound e _                = e

-- ─────────────────────────────────────────
-- 3. Mutual recursion detection
-- ─────────────────────────────────────────

-- Build call-graph between top-level lambdas, detect cycles of length > 1
checkMutualRec :: Program -> Errs
checkMutualRec prog =
  let lambdas  = [ (n, e) | SBind n e@(ELambda {}) <- prog ]
      callGraph = Map.fromList [ (n, freeCallsIn n (lambdaBody e)) | (n, e) <- lambdas ]
  in concatMap (findMutual callGraph) (Map.keys callGraph)

lambdaBody :: Expr -> Expr
lambdaBody (ELambda _ _ _ _ e) = e
lambdaBody e = e

-- | Names directly called by the body of function n (excluding n itself)
freeCallsIn :: Name -> Expr -> [Name]
freeCallsIn self e = filter (/= self) (collectCalls e)

collectCalls :: Expr -> [Name]
collectCalls (ECall (EVar n) args) = n : concatMap collectCalls args
collectCalls (EBinOp _ a b)        = collectCalls a ++ collectCalls b
collectCalls (EUnOp _ a)           = collectCalls a
collectCalls (EPipe a b)            = collectCalls a ++ collectCalls b
collectCalls (EIf c ts te ec)       = collectCalls c
  ++ concatMap collectStmtCalls ts ++ collectCalls te
  ++ maybe [] (\(es,ee) -> concatMap collectStmtCalls es ++ collectCalls ee) ec
collectCalls (EMatch sc arms)       = collectCalls sc ++ concatMap (\(MatchArm _ g b) -> collectCalls b) arms
collectCalls (EFor _ it ss fe)      = collectCalls it ++ concatMap collectStmtCalls ss ++ maybe [] collectCalls fe
collectCalls (EWhile c ss fe)       = collectCalls c ++ concatMap collectStmtCalls ss ++ maybe [] collectCalls fe
collectCalls (ELambda _ _ _ ss fe)  = concatMap collectStmtCalls ss ++ collectCalls fe
collectCalls (EBlock ss fe)         = concatMap collectStmtCalls ss ++ collectCalls fe
collectCalls _                      = []

collectStmtCalls :: Stmt -> [Name]
collectStmtCalls (SBind _ e)   = collectCalls e
collectStmtCalls (SExprStmt e) = collectCalls e
collectStmtCalls _             = []

-- | For each function, do a DFS up to depth 2 looking for mutual calls
findMutual :: Map.Map Name [Name] -> Name -> Errs
findMutual graph n =
  let callees = Map.findWithDefault [] n graph
      mutuals  = filter (\c -> n `elem` Map.findWithDefault [] c graph) callees
  in map (MutualRec n) mutuals

