use crate::ast::{BinOp, Decl, PExpr, Pattern, Token, Type};
use ariadne::{sources, Color, Label, Report, ReportKind};
use chumsky::input::MappedInput;
use chumsky::pratt::{infix, left, prefix};
use chumsky::prelude::*;
use chumsky::span::Spanned;
use std::fmt;

// adapted from https://codeberg.org/zesterer/chumsky/src/branch/main/examples/mini_ml.rs

fn lexer<'src>(
) -> impl Parser<'src, &'src str, Vec<Spanned<Token<'src>>>, extra::Err<Rich<'src, char>>> {
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
                "match" => Token::Match,
                "with" => Token::With,
                s if is_first_letter_lowercase(s) => Token::LIdent(s),
                s => Token::UIdent(s),
            }),
            // Operators
            just("_").to(Token::Underscore),
            just("=>").to(Token::ThickArrow),
            just("->").to(Token::ThinArrow),
            just("'").to(Token::SingleQuote),
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
        ))
            .spanned()
            .padded()
    })
        .repeated()
        .collect()
}

fn is_first_letter_lowercase(s: &str) -> bool {
    s.chars().next().map_or(false, |c| c.is_lowercase())
}

fn decl_parser<'tokens, 'src: 'tokens>() -> impl Parser<
    'tokens,
    MappedInput<'tokens, Token<'src>, SimpleSpan, &'tokens [Spanned<Token<'src>>]>,
    Spanned<Decl<'src>>,
    extra::Err<Rich<'tokens, Token<'src>>>,
> {
    let ident = select_ref! { Token::LIdent(x) => *x };
    let parse_type = recursive(|typ| {
        choice((
            just(Token::LIdent("num")).to(Type::Num),
            just(Token::LIdent("bool")).to(Type::Bool),
            just(Token::LIdent("unit")).to(Type::Unit),
            just(Token::SingleQuote)
                .ignore_then(ident.clone())
                .map(|s| Type::Tyvar(s.to_string())),
            typ.nested_in(select_ref! { Token::Parens(ts) = e => ts.split_spanned(e.span()) }),
        ))
            .pratt(infix(
                left(1),
                just(Token::ThinArrow).map_with(|_, e| e.span()),
                |l, _, r, _| Type::Fn(Box::new(l), Box::new(r)),
            ))
    });
    choice((
        just(Token::Let)
            .ignore_then(ident.spanned())
            .then(just(Token::Colon).ignore_then(parse_type.clone()).or_not())
            .then(ident.spanned().then(just(Token::Colon).ignore_then(parse_type.clone()).or_not()).repeated().collect::<Vec<_>>())
            .then_ignore(just(Token::Eq))
            .then(parser())
            .map(|(((lhs, typ), vs), rhs)| {
                Decl::Let(lhs, typ, vs, Box::new(rhs))
            })
            .spanned(),
        parser().map(|e| Decl::Expr(e.clone()).with_span(e.span))
    ))
}

fn parser<'tokens, 'src: 'tokens>() -> impl Parser<
    'tokens,
    MappedInput<'tokens, Token<'src>, SimpleSpan, &'tokens [Spanned<Token<'src>>]>,
    Spanned<PExpr<'src>>,
    extra::Err<Rich<'tokens, Token<'src>>>,
> {
    recursive(|expr| {
        let ident = select_ref! { Token::LIdent(x) => *x };
        let upper_ident = select_ref! { Token::UIdent(x) => *x };
        let parse_type = recursive(|typ| {
            choice((
                just(Token::LIdent("num")).to(Type::Num),
                just(Token::LIdent("bool")).to(Type::Bool),
                just(Token::LIdent("unit")).to(Type::Unit),
                just(Token::SingleQuote)
                    .ignore_then(ident.clone())
                    .map(|s| Type::Tyvar(s.to_string())),
                typ.nested_in(select_ref! { Token::Parens(ts) = e => ts.split_spanned(e.span()) }),
            ))
                .pratt(infix(
                    left(1),
                    just(Token::ThinArrow).map_with(|_, e| e.span()),
                    |l, _, r, _| Type::Fn(Box::new(l), Box::new(r)),
                ))
        });
        let parse_pattern = recursive(|pat| {
            let atom = choice((
                select_ref! { Token::Num(x) => Pattern::Num(*x) }.spanned(),
                just(Token::True).to(Pattern::Bool(true)).spanned(),
                just(Token::False).to(Pattern::Bool(false)).spanned(),
                just(Token::Underscore).to(Pattern::Wildcard).spanned(),
                ident.map(|s| Pattern::Var(s)).spanned(),
                pat.nested_in(select_ref! { Token::Parens(ts) = e => ts.split_spanned(e.span()) })
            ));
            let constructor = upper_ident.then(atom.clone().repeated().collect::<Vec<_>>())
                .map(|(x, ps)| Pattern::Constructor(x, ps))
                .spanned();
            choice((atom, constructor))
        });
        let atom = choice((
            select_ref! { Token::Num(x) => PExpr::Num(*x) }.spanned(),
            just(Token::True).to(PExpr::Bool(true)).spanned(),
            just(Token::False).to(PExpr::Bool(false)).spanned(),
            ident.map(|s| PExpr::Var(s)).spanned(),
            // let x = y in z
            just(Token::Let)
                .ignore_then(ident.spanned())
                .then(just(Token::Colon).ignore_then(parse_type.clone()).or_not())
                .then(ident.spanned().then(just(Token::Colon).ignore_then(parse_type.clone()).or_not()).repeated().collect::<Vec<_>>())
                .then_ignore(just(Token::Eq))
                .then(expr.clone())
                .then_ignore(just(Token::In))
                .then(expr.clone())
                .map(|((((lhs, typ), vs), rhs), then)| {
                    PExpr::LetIn(lhs, typ, vs, Box::new(rhs), Box::new(then))
                })
                .spanned(),
            // fn x: typ => y
            just(Token::Fn)
                .ignore_then(ident.spanned().then(just(Token::Colon).ignore_then(parse_type.clone()).or_not()).repeated().at_least(1).collect::<Vec<_>>())
                .then_ignore(just(Token::ThickArrow))
                .then(expr.clone())
                .map(|(vs, rhs)| PExpr::Fn(vs, Box::new(rhs)))
                .spanned(),
            // if x then y else z
            just(Token::If)
                .ignore_then(expr.clone())
                .then_ignore(just(Token::Then))
                .then(expr.clone())
                .then_ignore(just(Token::Else))
                .then(expr.clone())
                .map(|((cond, then_), else_)| {
                    PExpr::If(Box::new(cond), Box::new(then_), Box::new(else_))
                })
                .spanned(),
            just(Token::Match)
                .ignore_then(expr.clone())
                .then_ignore(just(Token::With))
                .then((parse_pattern.then_ignore(just(Token::ThickArrow)).then(expr.clone()))
                    .repeated().at_least(1).collect::<Vec<_>>())
                .map(|(x, branches)| {
                    PExpr::Match(Box::new(x), branches)
                })
                .spanned(),
            expr.nested_in(select_ref! { Token::Parens(ts) = e => ts.split_spanned(e.span()) }),
        ));
        atom.pratt((
            infix(
                left(2),
                just(Token::Plus).map_with(|_, e| e.span()),
                |l, op, r, e| {
                    PExpr::BinOp(BinOp::Plus.with_span(op), Box::new(l), Box::new(r))
                        .with_span(e.span())
                },
            ),
            infix(
                left(2),
                just(Token::Minus).map_with(|_, e| e.span()),
                |l, op, r, e| {
                    PExpr::BinOp(BinOp::Minus.with_span(op), Box::new(l), Box::new(r))
                        .with_span(e.span())
                },
            ),
            infix(
                left(3),
                just(Token::Asterisk).map_with(|_, e| e.span()),
                |l, op, r, e| {
                    PExpr::BinOp(BinOp::Times.with_span(op), Box::new(l), Box::new(r))
                        .with_span(e.span())
                },
            ),
            infix(
                left(3),
                just(Token::Slash).map_with(|_, e| e.span()),
                |l, op, r, e| {
                    PExpr::BinOp(BinOp::Div.with_span(op), Box::new(l), Box::new(r))
                        .with_span(e.span())
                },
            ),
            infix(
                left(1),
                just(Token::Geq).map_with(|_, e| e.span()),
                |l, op, r, e| {
                    PExpr::BinOp(BinOp::Geq.with_span(op), Box::new(l), Box::new(r))
                        .with_span(e.span())
                },
            ),
            infix(
                left(1),
                just(Token::Gt).map_with(|_, e| e.span()),
                |l, op, r, e| {
                    PExpr::BinOp(BinOp::Gt.with_span(op), Box::new(l), Box::new(r))
                        .with_span(e.span())
                },
            ),
            infix(
                left(1),
                just(Token::Leq).map_with(|_, e| e.span()),
                |l, op, r, e| {
                    PExpr::BinOp(BinOp::Leq.with_span(op), Box::new(l), Box::new(r))
                        .with_span(e.span())
                },
            ),
            infix(
                left(1),
                just(Token::Lt).map_with(|_, e| e.span()),
                |l, op, r, e| {
                    PExpr::BinOp(BinOp::Lt.with_span(op), Box::new(l), Box::new(r))
                        .with_span(e.span())
                },
            ),
            infix(
                left(1),
                just(Token::Eq).map_with(|_, e| e.span()),
                |l, op, r, e| {
                    PExpr::BinOp(BinOp::Eq.with_span(op), Box::new(l), Box::new(r))
                        .with_span(e.span())
                },
            ),
            infix(
                left(1),
                just(Token::Neq).map_with(|_, e| e.span()),
                |l, op, r, e| {
                    PExpr::BinOp(BinOp::Neq.with_span(op), Box::new(l), Box::new(r))
                        .with_span(e.span())
                },
            ),
            infix(left(10), empty(), |l, _, r, e| {
                PExpr::App(Box::new(l), Box::new(r)).with_span(e.span())
            }),
            prefix(2, just(Token::Minus), |_, body, e| {
                PExpr::Neg(Box::new(body)).with_span(e.span())
            }),
            infix(
                left(0),
                just(Token::Semicolon).map_with(|_, e| e.span()),
                |l, op, r, e| {
                    PExpr::Seq(Box::new(l), Box::new(r)).with_span(e.span())
                },
            ),
        ))
    })
}

fn failure(
    msg: String,
    label: (String, SimpleSpan),
    extra_labels: impl IntoIterator<Item = (String, SimpleSpan)>,
    src: &str,
) {
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
}

fn parse_failure(err: &Rich<impl fmt::Display>, src: &str) {
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

pub fn parse(src: &str) -> Result<Spanned<PExpr<'_>>, Vec<Rich<'_, String>>> {
    let tokens = lexer().parse(src).into_result().map_err(|errs| {
        parse_failure(&errs[0], src);
        errs.into_iter()
            .map(|e| e.map_token(|c| c.to_string()))
            .collect::<Vec<Rich<'_, String>>>()
    })?;

    let tokens = tokens.split_spanned((0..src.len()).into());

    let expr = parser().parse(tokens).into_result().map_err(|errs| {
        parse_failure(&errs[0], src);
        errs.into_iter()
            .map(|e| e.map_token(|tok| tok.to_string()).into_owned())
            .collect::<Vec<Rich<'_, String>>>()
    });

    expr
}
