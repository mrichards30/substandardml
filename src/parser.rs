use crate::ast::{BinOp, Expr, Token, Type};
use ariadne::{sources, Color, Label, Report, ReportKind};
use chumsky::input::MappedInput;
use chumsky::prelude::*;
use chumsky::span::Spanned;
use std::fmt;
use chumsky::pratt::{infix, left};

// adapted from https://codeberg.org/zesterer/chumsky/src/branch/main/examples/mini_ml.rs

fn lexer<'src>()
    -> impl Parser<'src, &'src str, Vec<Spanned<Token<'src>>>, extra::Err<Rich<'src, char>>> {
    recursive(|token| {
        choice((
            // Keywords
            text::ident().map(|s| match s {
                "let" => Token::Let,
                "in" => Token::In,
                "fn" => Token::Fn,
                "true" => Token::True,
                "false" => Token::False,
                "if" => Token::If,
                "then" => Token::Then,
                "else" => Token::Else,
                s => Token::Ident(s),
            }),
            // Operators
            just("=>").to(Token::ThickArrow),
            just("->").to(Token::ThinArrow),
            just("+").to(Token::Plus),
            just("-").to(Token::Minus),
            just("*").to(Token::Asterisk),
            just("/").to(Token::Slash),
            just(":").to(Token::Colon),
            just(";").to(Token::Semicolon),
            just("=").to(Token::Eq),
            just("!=").to(Token::Neq),
            just(">=").to(Token::Geq),
            just(">").to(Token::Gt),
            just("<=").to(Token::Leq),
            just("<").to(Token::Lt),
            // Numbers
            text::int(10)
                .then(just('.').then(text::digits(10)).or_not())
                .to_slice()
                .map(|s: &str| Token::Num(s.parse().unwrap())),
            token
                .repeated()
                .collect()
                .delimited_by(just('('), just(')'))
                .labelled("token tree")
                .as_context()
                .map(Token::Parens),
        )).spanned().padded()
    })
        .repeated()
        .collect()
}

fn parser<'tokens, 'src: 'tokens>() -> impl Parser<
    'tokens,
    MappedInput<'tokens, Token<'src>, SimpleSpan, &'tokens [Spanned<Token<'src>>]>,
    Spanned<Expr<'src>>,
    extra::Err<Rich<'tokens, Token<'src>>>,
> {
    recursive(|expr| {
        let ident = select_ref! { Token::Ident(x) => *x };
        let parse_type = select_ref! {
            Token::Ident("num") => Type::Num,
            Token::Ident("unit") => Type::Unit,
            Token::Ident("bool") => Type::Bool
        };
        let atom = choice((
            select_ref! { Token::Num(x) => Expr::Num(*x) }.spanned(),
            just(Token::True).to(Expr::Bool(true)).spanned(),
            just(Token::False).to(Expr::Bool(false)).spanned(),
            ident.map(|s| Expr::Var(s)).spanned(),
            // let x = y in z
            just(Token::Let)
                .ignore_then(ident.spanned())
                .then(just(Token::Colon).ignore_then(parse_type).or_not())
                .then_ignore(just(Token::Eq))
                .then(expr.clone())
                .then_ignore(just(Token::In))
                .then(expr.clone())
                .map(|(((lhs, typ), rhs), then)|
                    Expr::LetIn(lhs, typ, Box::new(rhs), Box::new(then)))
                .spanned(),
            // fn x: typ => y
            just(Token::Fn)
                .ignore_then(ident.spanned())
                .then(just(Token::Colon).ignore_then(parse_type).or_not())
                .then_ignore(just(Token::ThickArrow))
                .then(expr.clone())
                .map(|((lhs, typ), rhs)|
                    Expr::Fn(lhs, typ, Box::new(rhs)))
                .spanned(),
            // if x then y else z
            just(Token::If)
                .ignore_then(expr.clone())
                .then_ignore(just(Token::Then))
                .then(expr.clone())
                .then_ignore(just(Token::Else))
                .then(expr.clone())
                .map(|((cond, then_), else_)|
                    Expr::If(Box::new(cond), Box::new(then_), Box::new(else_)))
                .spanned(),
            expr.nested_in(select_ref! { Token::Parens(ts) = e => ts.split_spanned(e.span()) }),
        ));
        atom.pratt((
            infix(left(1), just(Token::Plus).map_with(|_, e| e.span()), |l, op, r, e|
                Expr::BinOp(BinOp::Plus.with_span(op), Box::new(l), Box::new(r)).with_span(e.span())),
            infix(left(1), just(Token::Minus).map_with(|_, e| e.span()), |l, op, r, e|
                Expr::BinOp(BinOp::Minus.with_span(op), Box::new(l), Box::new(r)).with_span(e.span())),
            infix(left(1), just(Token::Asterisk).map_with(|_, e| e.span()), |l, op, r, e|
                Expr::BinOp(BinOp::Times.with_span(op), Box::new(l), Box::new(r)).with_span(e.span())),
            infix(left(1), just(Token::Slash).map_with(|_, e| e.span()), |l, op, r, e|
                Expr::BinOp(BinOp::Div.with_span(op), Box::new(l), Box::new(r)).with_span(e.span())),
            infix(left(1), just(Token::Geq).map_with(|_, e| e.span()), |l, op, r, e|
                Expr::BinOp(BinOp::Geq.with_span(op), Box::new(l), Box::new(r)).with_span(e.span())),
            infix(left(1), just(Token::Gt).map_with(|_, e| e.span()), |l, op, r, e|
                Expr::BinOp(BinOp::Gt.with_span(op), Box::new(l), Box::new(r)).with_span(e.span())),
            infix(left(1), just(Token::Leq).map_with(|_, e| e.span()), |l, op, r, e|
                Expr::BinOp(BinOp::Leq.with_span(op), Box::new(l), Box::new(r)).with_span(e.span())),
            infix(left(1), just(Token::Lt).map_with(|_, e| e.span()), |l, op, r, e|
                Expr::BinOp(BinOp::Lt.with_span(op), Box::new(l), Box::new(r)).with_span(e.span())),
            infix(left(1), just(Token::Eq).map_with(|_, e| e.span()), |l, op, r, e|
                Expr::BinOp(BinOp::Eq.with_span(op), Box::new(l), Box::new(r)).with_span(e.span())),
            infix(left(1), just(Token::Neq).map_with(|_, e| e.span()), |l, op, r, e|
                Expr::BinOp(BinOp::Neq.with_span(op), Box::new(l), Box::new(r)).with_span(e.span())),
            infix(left(1), empty(), |l, op, r, e|
                Expr::App(Box::new(l), Box::new(r)).with_span(e.span())),
        ))
    })
}

fn failure(
    msg: String,
    label: (String, SimpleSpan),
    extra_labels: impl IntoIterator<Item = (String, SimpleSpan)>,
    src: &str,
) -> ! {
    let fname = "example";
    Report::build(ReportKind::Error, (fname, label.1.into_range()))
        .with_config(ariadne::Config::new().with_index_type(ariadne::IndexType::Byte))
        .with_message(&msg)
        .with_label(
            Label::new((fname, label.1.into_range()))
                .with_message(label.0)
                .with_color(Color::Red),
        )
        .with_labels(extra_labels.into_iter().map(|label2| {
            Label::new((fname, label2.1.into_range()))
                .with_message(label2.0)
                .with_color(Color::Yellow)
        }))
        .finish()
        .print(sources([(fname, src)]))
        .unwrap();
    std::process::exit(1)
}

fn parse_failure(err: &Rich<impl fmt::Display>, src: &str) -> ! {
    failure(
        err.reason().to_string(),
        (
            err.found()
                .map(|c| c.to_string())
                .unwrap_or_else(|| "end of input".to_string()),
            *err.span(),
        ),
        err.contexts()
            .map(|(l, s)| (format!("while parsing this {l}"), *s)),
        src,
    )
}

pub fn prs(src: &str) -> Spanned<Expr> {
    let tokens = lexer()
        .parse(src)
        .into_result()
        .unwrap_or_else(|errs| parse_failure(&errs[0], src));

    let expr = parser()
        .parse(tokens[..].split_spanned((0..src.len()).into()))
        .into_result()
        .unwrap_or_else(|errs| parse_failure(&errs[0], src));

    expr
}