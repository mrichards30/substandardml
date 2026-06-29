// use crate::ast::{CExpr, Value};
//
// pub fn print_value(val: &Value) -> String {
//     match val {
//         Value::Var(n) => n.to_string(),
//         Value::Label(l) => l.to_string(),
//         Value::Num(n) => n.to_string(),
//         Value::String(s) => format!("\"{}\"", s),
//     }
// }
//
// pub fn print_cexpr(expr: &CExpr) -> String {
//     match expr {
//         CExpr::App(f, args) => {
//             let args_str = args.iter().map(|x| print_value(&x.inner)).collect::<Vec<_>>().join(", ");
//             format!("APP({}, [{}])", print_value(f), args_str)
//         }
//         CExpr::Fix(fs, body) => {
//             let fs_str = fs.iter().map(|(name, params, body)| {
//                 format!("fn {}({}) = {}", name, params.join(", "), print_cexpr(body))
//             }).collect::<Vec<_>>().join("\n");
//             format!("FIX [\n{}\n] IN\n{}", fs_str, print_cexpr(body))
//         }
//         CExpr::PrimOp(op, args, results, conts) => {
//             let args_str = args.iter().map(print_value).collect::<Vec<_>>().join(", ");
//             let results_str = results.join(", ");
//             let conts_str = conts.iter().map(|c| print_cexpr(c)).collect::<Vec<_>>().join(", ");
//             format!("PRIMOP({:?}, [{}], [{}], [{}])", op, args_str, results_str, conts_str)
//         }
//         CExpr::Switch(v, conts) => {
//             let conts_str = conts.iter().map(|c| print_cexpr(c)).collect::<Vec<_>>().join(", ");
//             format!("SWITCH({}, [{}])", print_value(v), conts_str)
//         }
//     }
// }