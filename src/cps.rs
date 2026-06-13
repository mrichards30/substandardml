// use crate::ast::{Value, CExpr, Expr, BinOp, Decl};
// 
// fn fresh(counter: &mut u64) -> String {
//     let name =s format!("k{}", counter);
//     *counter += 1;
//     name
// }
// 
// pub fn to_cps(decl: &Decl, k: Value, counter: &mut u64) -> CExpr {
//     match decl {
//         Decl::Let(_, _, expr) => expr_to_cps(expr, k, counter), // TODO
//         Decl::LetRec(_, _, expr) => expr_to_cps(expr, k, counter), // TODO
//         Decl::Expr(expr) => expr_to_cps(expr, k, counter)
//     }
// }
// 
// pub fn expr_to_cps(expr: &Expr, k: Value, counter: &mut u64) -> CExpr {
//     match expr {
//         Expr::Var(p) => CExpr::App(k, vec![Value::Var(p.clone())]),
//         Expr::Int(n) => CExpr::App(k, vec![Value::Int(*n)]),
//         Expr::Bool(true) => CExpr::App(k, vec![Value::Int(1)]),
//         Expr::Bool(false) => CExpr::App(k, vec![Value::Int(0)]),
//         Expr::Unit => CExpr::App(k, vec![Value::Int(0)]),
//         Expr::If(cond, then, else_) => {
//             let k1 = fresh(counter);
//             let v1 = fresh(counter);
//             CExpr::Fix(
//                 vec![(k1.clone(), vec![v1.clone()],
//                       Box::new(CExpr::PrimOp(
//                           BinOp::Eq,
//                           vec![Value::Var(v1), Value::Int(1)],
//                           vec![],
//                           vec![
//                               expr_to_cps(else_, k.clone(), counter),
//                               expr_to_cps(then, k, counter),
//                           ]
//                       )))],
//                 Box::new(expr_to_cps(cond, Value::Var(k1), counter)))
//         }
//         Expr::LetIn(x, ty, body, in_) =>
//             // so lazy... write a custom translation?
//             expr_to_cps(&Expr::App(
//                 Box::new(Expr::Fn(x.clone(), ty.clone(), in_.clone())),
//                 body.clone()), k, counter),
//         Expr::App(f, arg) => {
//             let k1 = fresh(counter);
//             let fv = fresh(counter);
//             let k2 = fresh(counter);
//             let av = fresh(counter);
//             CExpr::Fix(
//                 vec![(k1.clone(), vec![fv.clone()],
//                       Box::new(CExpr::Fix(
//                           vec![(k2.clone(), vec![av.clone()],
//                                 Box::new(CExpr::App(
//                                     Value::Var(fv),
//                                     vec![Value::Var(av), k]  // pass continuation as last arg
//                                 )))],
//                           Box::new(expr_to_cps(arg, Value::Var(k2), counter)))))],
//                 Box::new(expr_to_cps(f, Value::Var(k1), counter)))
//         }
//         Expr::Seq(lhs, rhs) => {
//             let k1 = fresh(counter);
//             let v1 = fresh(counter);
//             CExpr::Fix(
//                 vec![(k1.clone(), vec![v1],
//                       Box::new(expr_to_cps(rhs, k, counter)))],
//                 Box::new(to_cps(lhs, Value::Var(k1), counter)))
//         }
//         Expr::BinOp(op, lhs, rhs) => {
//             let k1 = fresh(counter);
//             let v1 = fresh(counter);
//             let k2 = fresh(counter);
//             let v2 = fresh(counter);
//             let primop_cexp = match op {
//                 BinOp::Plus | BinOp::Minus | BinOp::Times | BinOp::Div => {
//                     let result = fresh(counter);
//                     CExpr::PrimOp(
//                         op.clone(),
//                         vec![Value::Var(v1.clone()), Value::Var(v2.clone())],
//                         vec![result.clone()],
//                         vec![CExpr::App(k, vec![Value::Var(result)])]
//                     )
//                 }
//                 BinOp::Eq | BinOp::Neq | BinOp::Geq | BinOp::Gt | BinOp::Leq | BinOp::Le => {
//                     CExpr::PrimOp(
//                         op.clone(),
//                         vec![Value::Var(v1.clone()), Value::Var(v2.clone())],
//                         vec![],
//                         vec![
//                             CExpr::App(k.clone(), vec![Value::Int(0)]), // false
//                             CExpr::App(k, vec![Value::Int(1)]),          // true
//                         ]
//                     )
//                 }
//             };
//             CExpr::Fix(
//                 vec![(k1.clone(), vec![v1],
//                       Box::new(CExpr::Fix(
//                           vec![(k2.clone(), vec![v2],
//                                 Box::new(primop_cexp))],
//                           Box::new(expr_to_cps(rhs, Value::Var(k2), counter)))))],
//                 Box::new(expr_to_cps(lhs, Value::Var(k1), counter)))
//         }
//         Expr::Fn(x, _ty, body) => {
//             let f = fresh(counter);
//             let k_param = fresh(counter);
//             CExpr::Fix(
//                 vec![(f.clone(), vec![x.clone(), k_param.clone()],
//                       Box::new(expr_to_cps(body, Value::Var(k_param), counter)))],
//                 Box::new(CExpr::App(k, vec![Value::Var(f)])))
//         }
//     }
// }