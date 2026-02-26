-- | Abstract Syntax Tree for the Homun language.
-- Every construct maps directly to a Rust construct.
module AST where

type Name = String

-- | A top-level Homun program is a sequence of statements
type Program = [Stmt]

-- ─────────────────────────────────────────
-- Statements
-- ─────────────────────────────────────────

data Stmt
  = SBind     Name   Expr              -- name := expr
  | SUse      [Name]                   -- use engine::physics::{Vec2}
  | SStructDef Name [FieldDef]         -- Player := struct { ... }
  | SEnumDef  Name [VariantDef]        -- Direction := enum { North, South(int) }
  | SExprStmt Expr                     -- bare expression (e.g. function call side-effect)
  deriving (Show)

data FieldDef   = FieldDef   Name (Maybe TypeExpr)  deriving (Show)
data VariantDef = VariantDef Name (Maybe TypeExpr)  deriving (Show)

-- ─────────────────────────────────────────
-- Expressions
-- ─────────────────────────────────────────

data Expr
  -- Literals
  = EInt    Integer
  | EFloat  Double
  | EBool   Bool
  | EString String          -- may contain ${...} interpolations
  | ENone

  -- Variables / field access
  | EVar    Name
  | EField  Expr Name       -- expr.field
  | EIndex  Expr Expr       -- expr[expr]
  | ESlice  Expr (Maybe Expr) (Maybe Expr) (Maybe Expr)   -- expr[a:b:c]

  -- Collections
  | EList   [Expr]                    -- @[...]
  | EDict   [(Expr, Expr)]            -- @{k: v, ...}
  | ESet    [Expr]                    -- @(...)
  | ETuple  [Expr]                    -- (a, b, c)

  -- Struct literal
  | EStruct (Maybe Name) [(Name, Expr)]   -- Player { name: "Aria", hp: 100 }

  -- Operators
  | EBinOp  BinOp Expr Expr
  | EUnOp   UnOp  Expr

  -- Pipe: lhs | f(args)
  | EPipe   Expr Expr                 -- desugars at codegen time

  -- Lambda
  | ELambda [Param] (Maybe TypeExpr) (Maybe TypeExpr) [Stmt] Expr
  --         params  ret-type         void-marker      stmts  final-expr

  -- Application
  | ECall   Expr [Expr]

  -- Control flow as expressions
  | EIf     Expr [Stmt] Expr (Maybe ([Stmt], Expr))  -- if cond do { stmts; expr } else { stmts; expr }
  | EMatch  Expr [MatchArm]

  -- Loops (produce unit / break value)
  | EFor    Name Expr [Stmt] (Maybe Expr)            -- for x in iter do { ... }
  | EWhile  Expr [Stmt] (Maybe Expr)                 -- while cond do { ... }

  -- Bare block { stmts; expr }
  | EBlock  [Stmt] Expr

  -- Break / continue
  | EBreak  (Maybe Expr)
  | EContinue

  -- RON  load/save
  | ELoadRon  Expr TypeExpr   -- load_ron("...") as Type
  | ESaveRon  Expr Expr       -- save_ron(data, "...")

  -- Destructuring bind (used inside block stmts, here for completeness)
  | ERange  (Maybe Expr) (Maybe Expr) (Maybe Expr)   -- range(a,b,c)

  deriving (Show)

-- ─────────────────────────────────────────
-- Match arm
-- ─────────────────────────────────────────

data MatchArm = MatchArm
  { armPat   :: Pat
  , armGuard :: Maybe Expr     -- _ if cond =>
  , armBody  :: Expr
  } deriving (Show)

data Pat
  = PWild                              -- _
  | PVar  Name                         -- variable binding
  | PLit  Expr                         -- literal
  | PTuple [Pat]                       -- (a, b, c)
  | PEnum Name (Maybe Pat)             -- Direction.North  /  Element.Fire(p)
  | PNone                              -- none
  deriving (Show)

-- ─────────────────────────────────────────
-- Types
-- ─────────────────────────────────────────

data TypeExpr
  = TName   Name            -- int, str, bool, float, PlayerState ...
  | TList   TypeExpr        -- @[T]
  | TDict   TypeExpr TypeExpr
  | TSet    TypeExpr
  | TOption TypeExpr        -- none-able
  | TTuple  [TypeExpr]
  | TVoid                   -- _ (no return value)
  | TInfer                  -- unknown, let Rust figure it out
  deriving (Show)

-- ─────────────────────────────────────────
-- Operators
-- ─────────────────────────────────────────

data BinOp
  = OpAdd | OpSub | OpMul | OpDiv | OpMod
  | OpEq  | OpNeq | OpLt | OpGt | OpLe | OpGe
  | OpAnd | OpOr
  | OpIn  | OpNotIn
  deriving (Show, Eq)

data UnOp
  = OpNot | OpNeg
  deriving (Show, Eq)

-- ─────────────────────────────────────────
-- Lambda parameter
-- ─────────────────────────────────────────

data Param = Param
  { paramName :: Name
  , paramType :: Maybe TypeExpr
  } deriving (Show)