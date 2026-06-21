use std::fmt;
use chumsky::span::Spanned;
use im::HashMap;

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
    Else
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

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Num,
    Bool,
    Unit,
    Fn(Box<Type>, Box<Type>),
    Tyvar(String)
}

#[derive(Debug, Clone)]
pub enum Decl<'src> {
    Let(String, Type, Spanned<Expr<'src>>),
    LetRec(String, Type, Spanned<Expr<'src>>),
    Expr(Spanned<Expr<'src>>),
}

#[derive(Debug, Clone)]
pub enum BinOp {
    Plus, Minus, Times, Div, 
    Eq, Neq, Geq, Gt, Leq, Lt
}

#[derive(Debug, Clone)]
pub enum Expr<'src> {
    Var(&'src str),
    Num(f64),
    Bool(bool),
    Unit,
    If(Box<Spanned<Self>>, Box<Spanned<Self>>, Box<Spanned<Self>>),
    LetIn(Spanned<&'src str>, Option<Type>, Box<Spanned<Self>>, Box<Spanned<Self>>),
    Fn(Spanned<&'src str>, Option<Type>, Box<Spanned<Self>>),
    App(Box<Spanned<Self>>, Box<Spanned<Self>>),
    Seq(Box<Decl<'src>>, Box<Spanned<Self>>),
    Neg(Box<Spanned<Self>>),
    BinOp(Spanned<BinOp>, Box<Spanned<Self>>, Box<Spanned<Self>>),
}

pub struct TypeEnv {
    constraints: HashMap<String, Type>,
    env: HashMap<String, Type>,
}

impl TypeEnv {
    pub fn new() -> Self {
        TypeEnv {
            constraints: Default::default(),
            env: Default::default(),
        }
    }

    pub fn upd_env(&self, s: String, v: Type) -> Self {
        TypeEnv {
            env: self.env.update(s, v),
            constraints: self.constraints.clone()
        }
    }

    pub fn get_env(&self, s: String) -> Option<Type> {
        self.env.get(&s).cloned()
    }

    pub fn upd_constraint(&self, s: String, v: Type) -> Self {
        TypeEnv {
            env: self.env.clone(),
            constraints: self.constraints.update(s, v)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeError {
    UnboundVariable(String),
    TypeMismatch { expected: Type, found: Type },
    NotAFunction(Type),
}

#[derive(Debug, Clone)]
pub enum Value {
    Var(String),
    Label(String),
    Num(f64),
    String(String),
}

#[derive(Debug, Clone)]
pub enum CExpr {
    App(Spanned<Value>, Vec<Spanned<Value>>),
    Fix(Vec<(String, Vec<String>, Box<Spanned<Self>>)>, Box<Spanned<Self>>),
    PrimOp(Spanned<BinOp>, Vec<Value>, Vec<String>, Vec<Spanned<Self>>),
    Switch(Spanned<Value>, Vec<Spanned<Self>>),
}
