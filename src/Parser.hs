module Parser (parseHomun) where

import Lexer (Token(..), TokenKind(..))
import AST

type ParseError = String

newtype Parser a = Parser { runParser :: [Token] -> Either ParseError (a, [Token]) }

instance Functor Parser where
  fmap f (Parser p) = Parser $ \ts -> case p ts of
    Left e -> Left e; Right (a,ts') -> Right (f a, ts')

instance Applicative Parser where
  pure a = Parser $ \ts -> Right (a, ts)
  Parser pf <*> Parser pa = Parser $ \ts -> do
    (f,ts') <- pf ts; (a,ts'') <- pa ts'; return (f a, ts'')

instance Monad Parser where
  return = pure
  Parser pa >>= f = Parser $ \ts -> do
    (a,ts') <- pa ts; runParser (f a) ts'

peek :: Parser Token
peek = Parser $ \ts -> case ts of [] -> Left "EOF"; (t:_) -> Right (t,ts)

advance :: Parser Token
advance = Parser $ \ts -> case ts of [] -> Left "EOF"; (t:rest) -> Right (t,rest)

expect :: TokenKind -> Parser Token
expect k = do
  t <- advance
  if tokenKind t == k then return t
  else failP $ "Expected " ++ show k ++ " but got " ++ show (tokenKind t)

failP :: String -> Parser a
failP msg = Parser $ \_ -> Left msg

tryP :: Parser a -> Parser (Maybe a)
tryP (Parser p) = Parser $ \ts -> case p ts of
  Left _       -> Right (Nothing, ts)
  Right (a,ts') -> Right (Just a, ts')

check :: TokenKind -> Parser Bool
check k = fmap ((== k) . tokenKind) peek

consume :: TokenKind -> Parser Bool
consume k = do b <- check k; if b then advance >> return True else return False

-- ─── Entry point ─────────────────────────────────────────────

parseHomun :: [Token] -> Either ParseError Program
parseHomun tokens = do
  (prog, _) <- runParser parseProgram tokens
  return prog

parseProgram :: Parser Program
parseProgram = do
  stmts <- parseMany parseTopStmt
  expect TEOF
  return stmts

parseMany :: Parser (Maybe a) -> Parser [a]
parseMany p = do
  mx <- p
  case mx of
    Nothing -> return []
    Just x  -> fmap (x:) (parseMany p)

-- ─── Top-level statements ────────────────────────────────────

parseTopStmt :: Parser (Maybe Stmt)
parseTopStmt = do
  t <- peek
  case tokenKind t of
    TEOF     -> return Nothing
    TRBrace  -> return Nothing
    TUse     -> fmap Just parseUse
    TIdent _ -> fmap Just parseTopBind
    _        -> return Nothing

parseUse :: Parser Stmt
parseUse = do
  expect TUse
  path <- parseModPath
  return (SUse path)

parseModPath :: Parser [Name]
parseModPath = do
  t <- advance
  case tokenKind t of
    TIdent n -> do
      b <- check TColon
      if b then do
        advance; b2 <- check TColon
        if b2 then do advance; rest <- parseModPath; return (n:rest)
        else return [n]
      else return [n]
    _ -> failP "Expected module name"

parseTopBind :: Parser Stmt
parseTopBind = do
  t <- advance
  let name = case tokenKind t of TIdent n -> n; _ -> "_"
  expect TAssign
  t2 <- peek
  case tokenKind t2 of
    TStruct -> do advance; fields <- parseBraceFields; return (SStructDef name fields)
    TEnum   -> do advance; variants <- parseBraceVariants; return (SEnumDef name variants)
    _       -> do rhs <- parseExpr; return (SBind name rhs)

parseBraceFields :: Parser [FieldDef]
parseBraceFields = do
  expect TLBrace
  fields <- parseFieldList
  expect TRBrace
  return fields

parseFieldList :: Parser [FieldDef]
parseFieldList = do
  t <- peek
  case tokenKind t of
    TRBrace  -> return []
    TIdent n -> do
      advance; expect TColon; ty <- parseTypeExpr; _ <- consume TComma
      rest <- parseFieldList
      return (FieldDef n (Just ty) : rest)
    _ -> return []

parseBraceVariants :: Parser [VariantDef]
parseBraceVariants = do
  expect TLBrace; vs <- parseVariantList; expect TRBrace; return vs

parseVariantList :: Parser [VariantDef]
parseVariantList = do
  t <- peek
  case tokenKind t of
    TRBrace  -> return []
    TIdent n -> do
      advance
      payload <- do
        b <- check TLParen
        if b then do
          advance; ty <- parseTypeExpr; expect TRParen; return (Just ty)
        else return Nothing
      _ <- consume TComma
      rest <- parseVariantList
      return (VariantDef n payload : rest)
    _ -> return []

-- ─── Block statements ────────────────────────────────────────

parseBlockStmts :: Parser [Stmt]
parseBlockStmts = do
  t <- peek
  case tokenKind t of
    TRBrace  -> return []
    TEOF     -> return []
    TIdent _ -> do s <- parseBlockBind; rest <- parseBlockStmts; return (s:rest)
    _        -> do e <- parseExpr; rest <- parseBlockStmts; return (SExprStmt e : rest)

parseBlockBind :: Parser Stmt
parseBlockBind = do
  mBind <- tryP $ do
    t <- advance
    let name = case tokenKind t of TIdent n -> n; _ -> "_"
    expect TAssign
    return name
  case mBind of
    Just name -> do rhs <- parseExpr; return (SBind name rhs)
    Nothing   -> do e <- parseExpr; return (SExprStmt e)

parseBlockStmts' :: Parser [Stmt]
parseBlockStmts' = do expect TLBrace; stmts <- parseBlockStmts; expect TRBrace; return stmts

splitBlock :: [Stmt] -> ([Stmt], Expr)
splitBlock [] = ([], ETuple [])
splitBlock stmts = case last stmts of
  SExprStmt e -> (init stmts, e)
  _           -> (stmts, ETuple [])

-- ─── Expressions ─────────────────────────────────────────────

parseExpr :: Parser Expr
parseExpr = parsePipe

parsePipe :: Parser Expr
parsePipe = do
  lhs <- parseOr
  b <- check TPipe
  if b then do advance; rhs <- parsePostfix; parsePipeTail (EPipe lhs rhs)
  else return lhs
  where
    parsePipeTail e = do
      b <- check TPipe
      if b then do advance; rhs <- parsePostfix; parsePipeTail (EPipe e rhs)
      else return e

parseOr :: Parser Expr
parseOr = do
  lhs <- parseAnd
  b <- check TOr
  if b then do advance; rhs <- parseOr; return (EBinOp OpOr lhs rhs)
  else return lhs

parseAnd :: Parser Expr
parseAnd = do
  lhs <- parseNot
  b <- check TAnd
  if b then do advance; rhs <- parseAnd; return (EBinOp OpAnd lhs rhs)
  else return lhs

parseNot :: Parser Expr
parseNot = do
  b <- check TNot
  if b then do advance; e <- parseNot; return (EUnOp OpNot e)
  else parseCmp

parseCmp :: Parser Expr
parseCmp = do
  lhs <- parseAddSub
  t <- peek
  case tokenKind t of
    TEq  -> do advance; rhs <- parseAddSub; return (EBinOp OpEq  lhs rhs)
    TNeq -> do advance; rhs <- parseAddSub; return (EBinOp OpNeq lhs rhs)
    TLt  -> do advance; rhs <- parseAddSub; return (EBinOp OpLt  lhs rhs)
    TGt  -> do advance; rhs <- parseAddSub; return (EBinOp OpGt  lhs rhs)
    TLe  -> do advance; rhs <- parseAddSub; return (EBinOp OpLe  lhs rhs)
    TGe  -> do advance; rhs <- parseAddSub; return (EBinOp OpGe  lhs rhs)
    TIn  -> do advance; rhs <- parseAddSub; return (EBinOp OpIn  lhs rhs)
    TNot -> do advance; expect TIn; rhs <- parseAddSub; return (EBinOp OpNotIn lhs rhs)
    _    -> return lhs

parseAddSub :: Parser Expr
parseAddSub = do lhs <- parseMulDiv; go lhs
  where
    go lhs = do
      t <- peek
      case tokenKind t of
        TPlus  -> do advance; rhs <- parseMulDiv; go (EBinOp OpAdd lhs rhs)
        TMinus -> do advance; rhs <- parseMulDiv; go (EBinOp OpSub lhs rhs)
        _      -> return lhs

parseMulDiv :: Parser Expr
parseMulDiv = do lhs <- parseUnary; go lhs
  where
    go lhs = do
      t <- peek
      case tokenKind t of
        TStar    -> do advance; rhs <- parseUnary; go (EBinOp OpMul lhs rhs)
        TSlash   -> do advance; rhs <- parseUnary; go (EBinOp OpDiv lhs rhs)
        TPercent -> do advance; rhs <- parseUnary; go (EBinOp OpMod lhs rhs)
        _        -> return lhs

parseUnary :: Parser Expr
parseUnary = do
  t <- peek
  case tokenKind t of
    TMinus -> do advance; e <- parseUnary; return (EUnOp OpNeg e)
    _      -> parsePostfix

parsePostfix :: Parser Expr
parsePostfix = do base <- parseAtom; go base
  where
    go e = do
      t <- peek
      case tokenKind t of
        TDot -> do
          advance; t2 <- advance
          case tokenKind t2 of
            TIdent n -> do
              b <- check TLParen
              if b then do args <- parseArgList; go (ECall (EField e n) args)
              else go (EField e n)
            _ -> failP "Expected field name after '.'"
        TLBracket -> do
          advance; result <- parseSliceOrIndex; expect TRBracket
          case result of
            Left idx        -> go (EIndex e idx)
            Right (a, b, c) -> go (ESlice e a b c)
        TLParen -> do args <- parseArgList; go (ECall e args)
        _       -> return e

parseSliceOrIndex :: Parser (Either Expr (Maybe Expr, Maybe Expr, Maybe Expr))
parseSliceOrIndex = do
  t <- peek
  case tokenKind t of
    TColon -> do advance; parseSliceRest Nothing
    _ -> do
      e <- parseExpr
      t2 <- peek
      case tokenKind t2 of
        TColon -> do advance; parseSliceRest (Just e)
        _      -> return (Left e)

parseSliceRest :: Maybe Expr -> Parser (Either Expr (Maybe Expr, Maybe Expr, Maybe Expr))
parseSliceRest start = do
  end  <- parseOptSlice
  step <- do b <- consume TColon; if b then parseOptSlice else return Nothing
  return (Right (start, end, step))

parseOptSlice :: Parser (Maybe Expr)
parseOptSlice = do
  t <- peek
  case tokenKind t of
    TRBracket -> return Nothing
    TColon    -> return Nothing
    _         -> fmap Just parseExpr

-- ─── Atoms ───────────────────────────────────────────────────

parseAtom :: Parser Expr
parseAtom = do
  t <- peek
  case tokenKind t of
    TInt n    -> advance >> return (EInt n)
    TFloat n  -> advance >> return (EFloat n)
    TBool b   -> advance >> return (EBool b)
    TString s -> advance >> return (EString s)
    TNone     -> advance >> return ENone

    -- Either a lambda  (params) -> { }  or a parenthesised expr  (e)
    -- Key: try lambda first (it needs ->  after the )), fall back to paren expr
    TLParen   -> parseLambdaOrParen

    TAt         -> parseCollection
    TIf         -> parseIfExpr
    TMatch      -> parseMatchExpr
    TFor        -> parseForExpr
    TWhile      -> parseWhileExpr
    TBreak      -> parseBreakExpr
    TContinue   -> advance >> return EContinue
    TLBrace     -> parseInlineBlock
    TUnderscore -> advance >> return (EVar "_")

    TIdent n -> do
      advance
      b <- check TLBrace
      if b && isUpperFirst n then do fields <- parseStructLitFields; return (EStruct (Just n) fields)
      else return (EVar n)

    _ -> failP $ "Unexpected token: " ++ show (tokenKind t)

-- ─── Lambda vs parenthesised expr ────────────────────────────
--
-- Strategy: consume  (params)  speculatively.
-- If the token immediately after ) is  ->  it's a lambda.
-- Otherwise restore and parse as  (expr)  or  (e1, e2, ...)  tuple.

parseLambdaOrParen :: Parser Expr
parseLambdaOrParen = do
  -- Try to parse as lambda: must see -> right after closing )
  mL <- tryP $ do
    expect TLParen
    params <- parseParamList
    expect TRParen
    -- The crucial test: -> must follow immediately
    t <- peek
    case tokenKind t of
      TArrow -> advance
      _      -> failP "not a lambda"
    -- Return type (optional)
    t2 <- peek
    (retTy, voidMark) <- case tokenKind t2 of
      TLBrace     -> return (Nothing, Nothing)
      TUnderscore -> do advance; return (Nothing, Just TVoid)
      _           -> do ty <- parseTypeExpr; return (Just ty, Nothing)
    stmts <- parseBlockStmts'
    let (ss, fe) = splitBlock stmts
    return (ELambda params retTy voidMark ss fe)
  case mL of
    Just l  -> return l
    Nothing -> do
      -- Parenthesised expression or tuple  (e)  /  (e1, e2, ...)
      expect TLParen
      e <- parseExpr
      t <- peek
      case tokenKind t of
        TComma -> do
          advance
          rest <- parseExprSep TRParen
          expect TRParen
          return (ETuple (e : rest))
        _ -> do expect TRParen; return e

parseParamList :: Parser [Param]
parseParamList = do
  t <- peek
  case tokenKind t of
    TRParen -> return []
    _       -> do p <- parseOneParam; rest <- parseParamTail; return (p:rest)

parseParamTail :: Parser [Param]
parseParamTail = do
  b <- consume TComma
  if b then do
    t <- peek
    case tokenKind t of
      TRParen -> return []
      _       -> do p <- parseOneParam; rest <- parseParamTail; return (p:rest)
  else return []

parseOneParam :: Parser Param
parseOneParam = do
  t <- advance
  case tokenKind t of
    TIdent n    -> do
      b <- consume TColon
      if b then do ty <- parseTypeExpr; return (Param n (Just ty))
      else return (Param n Nothing)
    TUnderscore -> return (Param "_" Nothing)
    _           -> failP $ "Expected param name, got " ++ show (tokenKind t)

parseArgList :: Parser [Expr]
parseArgList = do
  expect TLParen; args <- parseExprSep TRParen; expect TRParen; return args

parseExprSep :: TokenKind -> Parser [Expr]
parseExprSep stop = do
  t <- peek
  if tokenKind t == stop then return [] else do
    e <- parseExpr
    t2 <- peek
    case tokenKind t2 of
      TComma -> do advance; rest <- parseExprSep stop; return (e:rest)
      _      -> return [e]

-- ─── Collections ─────────────────────────────────────────────

parseCollection :: Parser Expr
parseCollection = do
  expect TAt; t <- peek
  case tokenKind t of
    TLBracket -> parseList
    TLBrace   -> parseDict
    TLParen   -> parseSet
    _         -> failP $ "Expected [, {{ or ( after @"

parseList :: Parser Expr
parseList = do
  expect TLBracket; t <- peek
  case tokenKind t of
    TRBracket -> advance >> return (EList [])
    _ -> do items <- parseExprSep TRBracket; expect TRBracket; return (EList items)

parseDict :: Parser Expr
parseDict = do
  expect TLBrace; t <- peek
  case tokenKind t of
    TRBrace -> advance >> return (EDict [])
    _       -> do pairs <- parseDictPairs; expect TRBrace; return (EDict pairs)

parseDictPairs :: Parser [(Expr, Expr)]
parseDictPairs = do
  k <- parseExpr; expect TColon; v <- parseExpr
  t <- peek
  case tokenKind t of
    TComma -> do advance; rest <- parseDictPairs; return ((k,v):rest)
    _      -> return [(k,v)]

parseSet :: Parser Expr
parseSet = do
  expect TLParen; items <- parseExprSep TRParen; expect TRParen; return (ESet items)

parseStructLitFields :: Parser [(Name, Expr)]
parseStructLitFields = do
  expect TLBrace; fields <- go; expect TRBrace; return fields
  where
    go = do
      t <- peek
      case tokenKind t of
        TRBrace  -> return []
        TIdent n -> do
          advance; expect TColon; v <- parseExpr; _ <- consume TComma
          rest <- go; return ((n,v):rest)
        _ -> return []

-- ─── Control flow ────────────────────────────────────────────

parseIfExpr :: Parser Expr
parseIfExpr = do
  expect TIf; expect TLParen; cond <- parseExpr; expect TRParen; expect TDo
  thenStmts <- parseBlockStmts'
  let (ts, te) = splitBlock thenStmts
  t <- peek
  elseClause <- case tokenKind t of
    TElse -> do
      advance; elseStmts <- parseBlockStmts'
      let (es, ee) = splitBlock elseStmts
      return (Just (es, ee))
    _ -> return Nothing
  return (EIf cond ts te elseClause)

parseMatchExpr :: Parser Expr
parseMatchExpr = do
  expect TMatch; scrut <- parseExpr; expect TLBrace
  arms <- parseMatchArms; expect TRBrace
  return (EMatch scrut arms)

parseMatchArms :: Parser [MatchArm]
parseMatchArms = do
  t <- peek
  case tokenKind t of
    TRBrace -> return []
    _       -> do arm <- parseMatchArm; rest <- parseMatchArms; return (arm:rest)

-- | A match arm pattern can be:
--   - a bare comma list:  0, _, _  =>  (parsed as PTuple)
--   - a single pattern:   _  /  42  /  Enum.Var(p)
--   Both forms may have an optional  if guard  before  =>
parseMatchArm :: Parser MatchArm
parseMatchArm = do
  -- Parse first pattern
  p0 <- parsePat
  -- Check if more comma-separated patterns follow (bare tuple pattern)
  pat <- do
    b <- check TComma
    if b then do
      advance
      rest <- parseMorePats
      return (PTuple (p0 : rest))
    else return p0
  -- Optional guard
  guard_ <- do
    b <- check TIf
    if b then do advance; fmap Just parseExpr else return Nothing
  expect TFatArrow
  body <- parseExpr
  return (MatchArm pat guard_ body)

-- Keep parsing comma-separated patterns until we hit  if  or  =>
parseMorePats :: Parser [Pat]
parseMorePats = do
  p <- parsePat
  t <- peek
  case tokenKind t of
    TComma -> do advance; rest <- parseMorePats; return (p:rest)
    _      -> return [p]

parsePat :: Parser Pat
parsePat = do
  t <- peek
  case tokenKind t of
    TUnderscore -> advance >> return PWild
    TNone       -> advance >> return PNone
    TIdent n    -> do
      advance
      b <- check TDot
      if b then do
        advance; t2 <- advance
        case tokenKind t2 of
          TIdent v -> do
            b2 <- check TLParen
            if b2 then do
              advance; p <- parsePat; expect TRParen
              return (PEnum (n ++ "." ++ v) (Just p))
            else return (PEnum (n ++ "." ++ v) Nothing)
          _ -> failP "Expected variant name after '.'"
      else return (PVar n)
    TInt n    -> advance >> return (PLit (EInt n))
    TFloat n  -> advance >> return (PLit (EFloat n))
    TBool b   -> advance >> return (PLit (EBool b))
    TString s -> advance >> return (PLit (EString s))
    _         -> failP $ "Expected pattern, got " ++ show (tokenKind t)

parseForExpr :: Parser Expr
parseForExpr = do
  expect TFor; t <- advance
  varName <- case tokenKind t of
    TIdent n    -> return n
    TUnderscore -> return "_"
    _           -> failP "Expected loop variable"
  expect TIn; iter <- parseExpr; expect TDo
  stmts <- parseBlockStmts'
  let (ss, fe) = splitBlock stmts
  return (EFor varName iter ss (Just fe))

parseWhileExpr :: Parser Expr
parseWhileExpr = do
  expect TWhile; expect TLParen; cond <- parseExpr; expect TRParen; expect TDo
  stmts <- parseBlockStmts'
  let (ss, fe) = splitBlock stmts
  return (EWhile cond ss (Just fe))

parseBreakExpr :: Parser Expr
parseBreakExpr = do
  expect TBreak
  b <- check TFatArrow
  if b then do advance; e <- parseExpr; return (EBreak (Just e))
  else return (EBreak Nothing)

parseInlineBlock :: Parser Expr
parseInlineBlock = do
  stmts <- parseBlockStmts'
  let (ss, fe) = splitBlock stmts
  return (EBlock ss fe)

-- ─── Types ───────────────────────────────────────────────────

parseTypeExpr :: Parser TypeExpr
parseTypeExpr = do
  t <- peek
  case tokenKind t of
    TAt -> do
      advance; t2 <- peek
      case tokenKind t2 of
        TLBracket -> do advance; ty <- parseTypeExpr; expect TRBracket; return (TList ty)
        TLBrace   -> do advance; k <- parseTypeExpr; expect TColon; v <- parseTypeExpr; expect TRBrace; return (TDict k v)
        TLParen   -> do advance; ty <- parseTypeExpr; expect TRParen; return (TSet ty)
        _         -> return TInfer
    TUnderscore -> advance >> return TVoid
    TIdent n    -> advance >> return (TName n)
    _           -> return TInfer

-- ─── Utilities ───────────────────────────────────────────────

isUpperFirst :: String -> Bool
isUpperFirst (c:_) = c >= 'A' && c <= 'Z'
isUpperFirst []    = False