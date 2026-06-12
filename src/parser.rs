use chumsky::input::MappedInput;
use chumsky::pratt::{infix, left};
use crate::ast::{BinOp, Expr, Token};
use chumsky::prelude::*;
use chumsky::span::Spanned;

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
            just("+").to(Token::Plus),
            just("-").to(Token::Minus),
            just("*").to(Token::Asterisk),
            just("/").to(Token::Slash),
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
    MappedInput<'tokens, Token<'src>, SimpleSpan, &'tokens [Token<'src>]>,
    Spanned<Expr>,
    extra::Err<Rich<'tokens, Token<'src>>>,
> {
    recursive(|expr| {
        let ident = select_ref! { Token::Ident(x) => x };
        let atom = choice((
            // select_ref! { Token::Num(x) => Expr::Int(*x) },
            // just(Token::True).to(Expr::Bool(true)),
            // just(Token::False).to(Expr::Bool(false)),
            // ident.map(|s| Expr::Var(s.to_string())),
            // let x = y in z
            just(Token::Let)
                .ignore_then(ident)
                .then_ignore(just(Token::Colon))
                .then(select_ref! { Token::Ident(x) => x }.or_not())
                .then_ignore(just(Token::Eq))
                .then(expr.clone())
                .then_ignore(just(Token::In))
                .then(expr.clone())
                .map(|(((lhs, ty), rhs), then)| Expr::LetIn (
                    lhs, ty, Box::new(rhs), Box::new(then)
                )))
        );

        choice((
            atom.spanned(),
            // fn x y = z
            just(Token::Fn).ignore_then(ident.spanned().repeated().foldr_with(
                just(Token::Eq).ignore_then(expr.clone()),
                |arg, body, e| {
                    Expr::Fn(arg, Box::new(body))
                },
            )),
            // ( x )
            expr.nested_in(select_ref! { Token::Parens(ts) = e => ts }),
        ))
            .pratt((
                // Multiply
                infix(left(10), just(Token::Asterisk), |x, _, y, e| {
                    Expr::BinOp(BinOp::Times, Box::new(x), Box::new(y)).with_span(e.span())
                }),
                // Add
                infix(left(9), just(Token::Plus), |x, _, y, e| {
                    Expr::BinOp(BinOp::Plus, Box::new(x), Box::new(y)).with_span(e.span())
                }),
                // Calls
                infix(left(1), empty(), |x, _, y, e| {
                    Expr::App(Box::new(x), Box::new(y))
                }),
            ))
            .labelled("expression")
            .as_context()
    })
}
