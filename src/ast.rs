use chumsky::span::{Spanned};
use im::HashMap;
use std::fmt;
use chumsky::prelude::SimpleSpan;

#[derive(Debug, Clone, PartialEq)]
pub enum Token<'src> {
    Ident(&'src str),
    Num(f64),
    Parens(Vec<Spanned<Self>>),
    Colon,
    Semicolon,

    // Ops
    Eq,
    Neq,
    Leq,
    Lt,
    Geq,
    Gt,
    Plus,
    Minus,
    Asterisk,
    Slash,
    ThickArrow,
    ThinArrow,
    SingleQuote,

    // Keywords
    Let,
    In,
    Fn,
    True,
    False,
    If,
    Then,
    Else,
}

impl fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Ident(x) => write!(f, "{x}"),
            Token::Num(x) => write!(f, "{x}"),
            Token::Parens(_) => write!(f, "(...)"),
            Token::Colon => write!(f, ":"),
            Token::Semicolon => write!(f, ";"),
            Token::Eq => write!(f, "="),
            Token::Plus => write!(f, "+"),
            Token::Asterisk => write!(f, "*"),
            Token::Let => write!(f, "let"),
            Token::In => write!(f, "in"),
            Token::Fn => write!(f, "fn"),
            Token::True => write!(f, "true"),
            Token::False => write!(f, "false"),
            Token::Neq => write!(f, "!="),
            Token::Leq => write!(f, "!="),
            Token::Lt => write!(f, "<"),
            Token::Geq => write!(f, ">="),
            Token::Gt => write!(f, ">"),
            Token::Minus => write!(f, "-"),
            Token::Slash => write!(f, "/"),
            Token::If => write!(f, "if"),
            Token::Then => write!(f, "then"),
            Token::Else => write!(f, "else"),
            Token::ThickArrow => write!(f, "=>"),
            Token::ThinArrow => write!(f, "->"),
            Token::SingleQuote => write!(f, "'"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Num,
    Bool,
    Unit,
    Fn(Box<Type>, Box<Type>),
    Tyvar(String),
}

pub type Scheme = (Vec<String>, Type);

#[derive(Debug, Clone)]
pub enum Decl<'src> {
    Let(String, Type, Spanned<PExpr<'src>>),
    LetRec(String, Type, Spanned<PExpr<'src>>),
    Expr(Spanned<PExpr<'src>>),
}

#[derive(Debug, Clone, Copy)]
pub enum BinOp {
    Plus,
    Minus,
    Times,
    Div,
    Eq,
    Neq,
    Geq,
    Gt,
    Leq,
    Lt,
}

#[derive(Debug, Clone)]
pub enum PExpr<'src> {
    Var(&'src str),
    Num(f64),
    Bool(bool),
    Unit,
    If(Box<Spanned<Self>>, Box<Spanned<Self>>, Box<Spanned<Self>>),
    LetIn(
        Spanned<&'src str>,
        Option<Type>,
        Box<Spanned<Self>>,
        Box<Spanned<Self>>,
    ),
    Fn(Spanned<&'src str>, Option<Type>, Box<Spanned<Self>>),
    App(Box<Spanned<Self>>, Box<Spanned<Self>>),
    Seq(Box<Spanned<Self>>, Box<Spanned<Self>>),
    Neg(Box<Spanned<Self>>),
    BinOp(Spanned<BinOp>, Box<Spanned<Self>>, Box<Spanned<Self>>),
}

pub type ExprId = usize;

#[derive(Debug, Clone)]
pub enum Expr<'src> {
    Var(&'src str),
    Num(f64),
    Bool(bool),
    Unit,
    If(ExprId, ExprId, ExprId),
    LetIn(
        &'src str,
        Option<Type>,
        ExprId,
        ExprId,
    ),
    Fn(&'src str, Option<Type>, ExprId),
    App(ExprId, ExprId),
    Seq(ExprId, ExprId),
    Neg(ExprId),
    BinOp(BinOp, ExprId, ExprId),
}

pub type CodeLoc = (usize, usize);

#[derive(Debug, Clone)]
pub struct Ast<'src> {
    pub exprs: Vec<Expr<'src>>,
    pub spans: Vec<CodeLoc>
}

impl<'a> Ast<'a> {
    pub fn new() -> Ast<'a> {
        Ast {
            exprs: vec![],
            spans: vec![]
        }
    }
    pub fn push(&mut self, e: Expr<'a>, l: CodeLoc) -> ExprId {
        self.exprs.push(e);
        self.spans.push(l);
        self.exprs.len() - 1
    }
}

pub fn lower<'src>(ast: &mut Ast<'src>, p: Spanned<PExpr<'src>>) -> ExprId {
    use PExpr::*;
    match p.inner {
        Var(v) => ast.push(Expr::Var(v), to_code_loc(p.span)),
        Num(n) => ast.push(Expr::Num(n), to_code_loc(p.span)),
        Bool(b) => ast.push(Expr::Bool(b), to_code_loc(p.span)),
        Unit => ast.push(Expr::Unit, to_code_loc(p.span)),
        If(e1, e2, e3) => {
            let id1 = lower(ast, *e1);
            let id2 = lower(ast, *e2);
            let id3 = lower(ast, *e3);
            ast.push(Expr::If(id1, id2, id3), to_code_loc(p.span))
        }
        LetIn(v, ty, e1, e2) => {
            let id1 = lower(ast, *e1);
            let id2 = lower(ast, *e2);
            ast.push(Expr::LetIn(v.inner, ty, id1, id2), to_code_loc(p.span))
        }
        Fn(v, ty, e1) => {
            let id1 = lower(ast, *e1);
            ast.push(Expr::Fn(v.inner, ty, id1), to_code_loc(p.span))
        }
        App(e1, e2) => {
            let id1 = lower(ast, *e1);
            let id2 = lower(ast, *e2);
            ast.push(Expr::App(id1, id2), to_code_loc(p.span))
        }
        Seq(e1, e2) => {
            let id1 = lower(ast, *e1);
            let id2 = lower(ast, *e2);
            ast.push(Expr::Seq(id1, id2), to_code_loc(p.span))
        }
        Neg(e) => {
            let id1 = lower(ast, *e);
            ast.push(Expr::Neg(id1), to_code_loc(p.span))
        }
        BinOp(op, e1, e2) => {
            let id1 = lower(ast, *e1);
            let id2 = lower(ast, *e2);
            ast.push(Expr::BinOp(op.inner, id1, id2), to_code_loc(p.span))
        }
    }
}

fn to_code_loc(span: SimpleSpan) -> CodeLoc {
    (span.start, span.end)
}

#[derive(Debug, Clone)]
pub struct TypeEnv {
    env: HashMap<String, Scheme>,
}

impl TypeEnv {
    pub fn new() -> Self {
        TypeEnv {
            env: Default::default(),
        }
    }

    pub fn upd_env(&self, s: String, v: Scheme) -> TypeEnv {
        TypeEnv { env: self.env.update(s, v) }
    }

    pub fn get_env(&self, s: String) -> Option<Scheme> {
        self.env.get(&s).cloned()
    }

    pub fn get_env_map(&self) -> HashMap<String, Scheme> {
        self.env.clone()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeError {
    UnboundVariable(String),
    TypeMismatch { expected: Type, found: Type },
    NotAFunction(Type),
}

#[derive(Debug, Clone)]
pub enum Value<'src> {
    Var(String),
    Label(String),
    Num(f64),
    String(&'src str),
}

pub type ValueId = usize;

#[derive(Debug, Clone)]
pub enum CExpr {
    App(ValueId, Vec<ValueId>),
    Fix(Vec<(ValueId, Vec<ValueId>, ExprId)>, ExprId),
    PrimOp(BinOp, Vec<ValueId>, Vec<ValueId>, Vec<ExprId>),
    Switch(ValueId, ExprId),
}

#[derive(Debug, Clone)]
pub struct CpsAst<'src> {
    pub exprs: Vec<CExpr>,
    pub spans: Vec<CodeLoc>,
    pub vals: Vec<Value<'src>>,
    pub val_spans: Vec<CodeLoc>,
}

impl<'a> CpsAst<'a> {
    pub fn new() -> CpsAst<'a> {
        CpsAst {
            exprs: vec![],
            spans: vec![],
            vals: vec![],
            val_spans: vec![],
        }
    }
    pub fn push(&mut self, e: CExpr, l: CodeLoc) -> ExprId {
        self.exprs.push(e);
        self.spans.push(l);
        self.exprs.len() - 1
    }
    pub fn push_val(&mut self, e: Value<'a>, l: CodeLoc) -> ExprId {
        self.vals.push(e);
        self.val_spans.push(l);
        self.vals.len() - 1
    }
}
