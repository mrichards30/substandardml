use std::fmt;
use im::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Token<'src> {
    Ident(&'src str),
    Num(f64),
    Parens(Vec<Self>),

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

    // Keywords
    Let,
    In,
    Fn,
    True,
    False,
    If,
    Then,
    Else
}

impl fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Ident(x) => write!(f, "{x}"),
            Token::Num(x) => write!(f, "{x}"),
            Token::Parens(_) => write!(f, "(...)"),
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
            Token::Else => write!(f, "else")
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Bool,
    Unit,
    Fn(Box<Type>, Box<Type>),
    Tyvar(String)
}

#[derive(Debug, Clone)]
pub enum Decl {
    Let(String, Type, Expr),
    LetRec(String, Type, Expr),
    Expr(Expr),
}

#[derive(Debug, Clone)]
pub enum BinOp {
    Plus, Minus, Times, Div, 
    Eq, Neq, Geq, Gt, Leq, Lt
}

#[derive(Debug, Clone)]
pub enum Expr {
    Var(String),
    Int(i64),
    Bool(bool),
    Unit,
    If(Box<Self>, Box<Self>, Box<Self>),
    LetIn(String, Option<Type>, Box<Self>, Box<Self>),
    Fn(String, Option<Type>, Box<Self>),
    App(Box<Self>, Box<Self>),
    Seq(Box<Decl>, Box<Self>),
    BinOp(BinOp, Box<Self>, Box<Self>),
}

pub type TypeEnv = HashMap<String, Type>;

#[derive(Debug, Clone)]
pub enum TypeError {
    UnboundVariable(String),
    TypeMismatch { expected: Type, found: Type },
    NotAFunction(Type),
}

#[derive(Debug, Clone)]
pub enum Value {
    Var(String),
    Label(String),
    Int(i64),
    String(String),
}

#[derive(Debug, Clone)]
pub enum CExpr {
    App(Value, Vec<Value>),
    Fix(Vec<(String, Vec<String>, Box<CExpr>)>, Box<CExpr>),
    PrimOp(BinOp, Vec<Value>, Vec<String>, Vec<CExpr>),
    Switch(Value, Vec<CExpr>),
}