use chumsky::span::{SpanWrap, Spanned};
use crate::ast::{Value, CExpr, Expr, BinOp, Decl};

fn fresh(counter: &mut u64) -> String {
    let name = format!("k{}", counter);
    *counter += 1;
    name
}

pub fn to_cps(decl: &Decl, k: Value, counter: &mut u64) -> Spanned<CExpr> {
    match decl {
        Decl::Let(_, _, expr) => expr_to_cps(expr, k, counter), // TODO
        Decl::LetRec(_, _, expr) => expr_to_cps(expr, k, counter), // TODO
        Decl::Expr(expr) => expr_to_cps(expr, k, counter)
    }
}

pub fn expr_to_cps(expr: &Spanned<Expr>, k: Value, counter: &mut u64) -> Spanned<CExpr> {
    let k_span = k.with_span(expr.span);
    match &expr.inner {
        Expr::Var(p) => CExpr::App(k_span, vec![Value::Var(p.to_string()).with_span(expr.span)]).with_span(expr.span),
        Expr::Num(n) => CExpr::App(k_span, vec![Value::Num(*n).with_span(expr.span)]).with_span(expr.span),
        Expr::Bool(true) => CExpr::App(k_span, vec![Value::Num(1f64).with_span(expr.span)]).with_span(expr.span),
        Expr::Bool(false) => CExpr::App(k_span, vec![Value::Num(0f64).with_span(expr.span)]).with_span(expr.span),
        Expr::Unit => CExpr::App(k_span, vec![Value::Num(0f64).with_span(expr.span)]).with_span(expr.span),
        Expr::If(cond, then, else_) => {
            let k1 = fresh(counter);
            let v1 = fresh(counter);
            CExpr::Fix(
                vec![(k1.to_string(), vec![v1.to_string()],
                      Box::new(CExpr::PrimOp(
                          BinOp::Eq.with_span(cond.span),
                          vec![Value::Var(v1), Value::Num(1f64)],
                          vec![],
                          vec![
                              expr_to_cps(&else_, k, counter),
                              expr_to_cps(&then, k.clone(), counter),
                          ]
                      ).with_span(expr.span)))],
                Box::new(expr_to_cps(cond, Value::Var(k1), counter))).with_span(expr.span)
        }
        Expr::LetIn(x, ty, body, in_) =>
            // so lazy... write a custom translation?
            expr_to_cps(&Expr::App(
                Box::new(Expr::Fn(x.clone(), ty.clone(), in_.clone()).with_span(body.span)),
                body.clone()).with_span(expr.span), k, counter),
        Expr::App(f, arg) => {
            let k1 = fresh(counter);
            let fv = fresh(counter);
            let k2 = fresh(counter);
            let av = fresh(counter);
            CExpr::Fix(
                vec![(k1.clone(), vec![fv.clone()],
                      Box::new(CExpr::Fix(
                          vec![(k2.clone(), vec![av.clone()],
                                Box::new(CExpr::App(
                                    Value::Var(fv).with_span(f.span),
                                    vec![Value::Var(av).with_span(f.span), k_span]  // pass continuation as last arg
                                ).with_span(f.span)))],
                          Box::new(expr_to_cps(arg, Value::Var(k2), counter))).with_span(expr.span)))],
                Box::new(expr_to_cps(f, Value::Var(k1), counter))).with_span(expr.span)
        }
        Expr::Seq(lhs, rhs) => {
            let k1 = fresh(counter);
            let v1 = fresh(counter);
            CExpr::Fix(
                vec![(k1.clone(), vec![v1],
                      Box::new(expr_to_cps(rhs, k, counter)))],
                Box::new(to_cps(lhs, Value::Var(k1), counter))).with_span(expr.span)
        }
        Expr::BinOp(op, lhs, rhs) => {
            let k1 = fresh(counter);
            let v1 = fresh(counter);
            let k2 = fresh(counter);
            let v2 = fresh(counter);
            let primop_cexp = match op.inner {
                BinOp::Plus | BinOp::Minus | BinOp::Times | BinOp::Div => {
                    let result = fresh(counter);
                    CExpr::PrimOp(
                        op.clone(),
                        vec![Value::Var(v1.clone()), Value::Var(v2.clone())],
                        vec![result.clone()],
                        vec![CExpr::App(k_span, vec![Value::Var(result).with_span(expr.span)])
                            .with_span(expr.span)]
                    )
                }
                BinOp::Eq | BinOp::Neq | BinOp::Geq | BinOp::Gt | BinOp::Leq | BinOp::Lt => {
                    CExpr::PrimOp(
                        op.clone(),
                        vec![Value::Var(v1.clone()), Value::Var(v2.clone())],
                        vec![],
                        vec![
                            CExpr::App(k_span, vec![Value::Num(0f64).with_span(expr.span)]).with_span(expr.span), // false
                            CExpr::App(k_span, vec![Value::Num(1f64).with_span(expr.span)]).with_span(expr.span), // true
                        ]
                    )
                }
            }.with_span(expr.span);
            CExpr::Fix(
                vec![(k1.clone(), vec![v1],
                      Box::new(CExpr::Fix(
                          vec![(k2.clone(), vec![v2],
                                Box::new(primop_cexp))],
                          Box::new(expr_to_cps(rhs, Value::Var(k2), counter))).with_span(expr.span)))],
                Box::new(expr_to_cps(lhs, Value::Var(k1), counter))).with_span(expr.span)
        }
        Expr::Fn(x, _ty, body) => {
            let f = fresh(counter);
            let k_param = fresh(counter);
            CExpr::Fix(
                vec![(f.clone(), vec![x.to_string(), k_param.clone()],
                      Box::new(expr_to_cps(body, Value::Var(k_param), counter)))],
                Box::new(CExpr::App(k_span, vec![Value::Var(f).with_span(x.span)]).with_span(expr.span)))
                .with_span(expr.span)
        }
    }
}