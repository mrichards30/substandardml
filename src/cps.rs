use crate::ast::{Ast, BinOp, CExpr, CodeLoc, CpsAst, Expr, ExprId, Value, ValueId};

fn fresh(counter: &mut u64, cps_ast: &mut CpsAst, span: CodeLoc) -> ValueId {
    let name = format!("k{}", counter);
    cps_ast.push_val(Value::Var(name), span)
}

pub fn expr_to_cps(ast: &Ast, expr_id: ExprId, cps_ast: &mut CpsAst, k: ValueId, counter: &mut u64) -> ExprId {
    let span = ast.spans[expr_id];
    match &ast.exprs[expr_id] {
        Expr::Var(p) => {
            let v = cps_ast.push_val(Value::Var(p.to_string()), ast.spans[expr_id]);
            cps_ast.push(CExpr::App(k, vec![v]), span)
        }
        Expr::Num(n) => {
            let v = cps_ast.push_val(Value::Num(*n), ast.spans[expr_id]);
            cps_ast.push(CExpr::App(k, vec![v]), span)
        }
        Expr::Bool(true) => {
            let v = cps_ast.push_val(Value::Num(1f64), ast.spans[expr_id]);
            cps_ast.push(CExpr::App(k, vec![v]), span)
        }
        Expr::Bool(false) => {
            let v = cps_ast.push_val(Value::Num(0f64), ast.spans[expr_id]);
            cps_ast.push(CExpr::App(k, vec![v]), span)
        }
        Expr::Unit => {
            let v = cps_ast.push_val(Value::Num(0f64), ast.spans[expr_id]);
            cps_ast.push(CExpr::App(k, vec![v]), span)
        }
        Expr::If(cond, then, else_) => {
            let k1 = fresh(counter, cps_ast, span);
            let v1 = fresh(counter, cps_ast, span);
            let v2 = cps_ast.push_val(Value::Num(1f64), ast.spans[expr_id]);
            let else_cps = expr_to_cps(ast, *else_, cps_ast, k, counter);
            let then_cps = expr_to_cps(ast, *then, cps_ast, k, counter);
            let cond_cps = expr_to_cps(ast, *cond, cps_ast, k1, counter);
            let cexpr =
                CExpr::Fix(vec![(k1, vec![v1],
                                 cps_ast.push(CExpr::PrimOp(
                                     BinOp::Eq,
                                     vec![v1, v2],
                                     vec![],
                                     vec![else_cps, then_cps]), span))],
                           cond_cps);
            cps_ast.push(cexpr, span)
        }
        Expr::LetIn(x, ty, body, in_) => {
            // TODO make this case better
            let ast1 = &mut ast.clone();
            let rand = ast1.push(Expr::Fn(x, ty.clone(), *in_), span);
            let app = ast1.push(Expr::App(rand, *body), span);
            expr_to_cps(ast1, app, cps_ast, k, counter)
        },
        Expr::App(f, arg) => {
            let k1 = fresh(counter, cps_ast, span);
            let fv = fresh(counter, cps_ast, span);
            let k2 = fresh(counter, cps_ast, span);
            let av = fresh(counter, cps_ast, span);
            let app_ce = cps_ast.push(CExpr::App(fv, vec![av]), span);
            let rator_ce = expr_to_cps(ast, *arg, cps_ast, k2, counter);
            let rand_ce = expr_to_cps(ast, *f, cps_ast, k1, counter);
            let cexpr =
                CExpr::Fix(
                    vec![(k1, vec![fv],
                          cps_ast.push(CExpr::Fix(vec![(k2, vec![av], app_ce)],
                              rator_ce), span))],
                    rand_ce);
            cps_ast.push(cexpr, span)
        }
        Expr::Seq(lhs, rhs) => {
            let k1 = fresh(counter, cps_ast, span);
            let v1 = fresh(counter, cps_ast, span);
            let cexpr =
                CExpr::Fix(vec![(k1, vec![v1], expr_to_cps(ast, *rhs, cps_ast, k, counter))],
                           expr_to_cps(ast, *lhs, cps_ast, k1, counter));
            cps_ast.push(cexpr, span)
        }
        Expr::Neg(e) => {
            // TODO change this to use primop directly
            let ast1 = &mut ast.clone();
            let zero = ast1.push(Expr::Num(0f64), span);
            let app = ast1.push(Expr::BinOp(BinOp::Minus, zero, *e), span);
            expr_to_cps(ast1, app, cps_ast, k, counter)
        }
        Expr::BinOp(op, lhs, rhs) => {
            let k1 = fresh(counter, cps_ast, span);
            let v1 = fresh(counter, cps_ast, span);
            let k2 = fresh(counter, cps_ast, span);
            let v2 = fresh(counter, cps_ast, span);
            let primop_cexp = match op {
                BinOp::Plus | BinOp::Minus | BinOp::Times | BinOp::Div => {
                    let result = fresh(counter, cps_ast, span);
                    CExpr::PrimOp(
                        *op,
                        vec![v1, v2],
                        vec![result],
                        vec![cps_ast.push(CExpr::App(k, vec![result]), span)]
                    )
                }
                BinOp::Eq | BinOp::Neq | BinOp::Geq | BinOp::Gt | BinOp::Leq | BinOp::Lt => {
                    let zero_ce = cps_ast.push_val(Value::Num(0f64), span);
                    let one_ce = cps_ast.push_val(Value::Num(1f64), span);
                    CExpr::PrimOp(
                        op.clone(),
                        vec![v1, v2],
                        vec![],
                        vec![
                            cps_ast.push(CExpr::App(k, vec![zero_ce]), span),
                            cps_ast.push(CExpr::App(k, vec![one_ce]), span),
                        ]
                    )
                }
            };
            let primop_cexp_id = cps_ast.push(primop_cexp, span);
            let rhs_ce = expr_to_cps(ast, *rhs, cps_ast, k2, counter);
            let lhs_ce = expr_to_cps(ast, *lhs, cps_ast, k1, counter);
            let cexpr =
                CExpr::Fix(
                    vec![(k1, vec![v1],
                          cps_ast.push(CExpr::Fix(
                              vec![(k2, vec![v2], primop_cexp_id)],
                              rhs_ce), span))],
                    lhs_ce);
            cps_ast.push(cexpr, span)
        }
        Expr::Fn(x, _ty, body) => {
            let f = fresh(counter, cps_ast, span);
            let k_param = fresh(counter, cps_ast, span);
            let xv = cps_ast.push_val(Value::Var(x.to_string()), span);
            let cexpr =
                CExpr::Fix(
                    vec![(f, vec![xv, k_param],
                          expr_to_cps(ast, *body, cps_ast, k_param, counter))],
                    cps_ast.push(CExpr::App(k, vec![f]), span));
            cps_ast.push(cexpr, span)
        },
    }
}