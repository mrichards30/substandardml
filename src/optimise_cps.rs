use crate::ast::{CExpr, Value};
use crate::ast::CExpr::App;

// fn optimise_cps(expr: CExpr) -> CExpr {
//     match expr {
//         CExpr::App(_, _) => expr,
//         CExpr::Fix(vs, body) => match *body {
//             App((Value::String(f)), x) => {
//                 if vs.iter().any(|(f_name, _, _)| *f_name == f) {
//                     remove H
//                 } else {
//                     expr.clone()
//                 }
//             }
//             _ => expr.clone()
//         },
//         CExpr::Fix(_, _) => expr,
//         CExpr::PrimOp(_, _, _, _) => expr,
//         CExpr::Switch(_, _) => expr
//     }
// }