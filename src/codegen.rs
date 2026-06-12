use crate::ast::{CExpr, Value, BinOp};

pub fn gen_value(val: &Value) -> String {
    match val {
        Value::Var(n) => n.clone(),
        Value::Label(l) => l.clone(),
        Value::Int(n) => n.to_string(),
        Value::String(s) => format!("\"{}\"", s),
    }
}

pub fn gen_binop(op: BinOp) -> String {
    match op {
        BinOp::Plus => "+".to_string(),
        BinOp::Minus => "-".to_string(),
        BinOp::Times => "*".to_string(),
        BinOp::Div => "/".to_string(),
        BinOp::Eq => "===".to_string(),
        BinOp::Neq => "!==".to_string(),
        BinOp::Geq => ">=".to_string(),
        BinOp::Gt => ">".to_string(),
        BinOp::Leq => "<=".to_string(),
        BinOp::Lt => "<".to_string()
    }
}

pub fn gen_cexpr(expr: &CExpr) -> String {
    match expr {
        CExpr::App(f, xs) =>
            format!("{}({})", gen_value(f), xs.iter().map(gen_value).collect::<Vec<_>>().join(", ")),
        CExpr::Fix(vs, body) =>
            format!("{}\n{}",
                    vs.iter().map(|(f, params, b)| {
                        format!("function {}({}) {{ {} }}", f, params.join(", "), gen_cexpr(b))
                    }).collect::<Vec<_>>().join("\n"),
                    gen_cexpr(body)),
        CExpr::PrimOp(op, inputs, outputs, cs) => {
            let val = inputs.iter().map(gen_value).collect::<Vec<_>>().join(&format!(" {} ", gen_binop(op.clone())));
            if cs.len() == 2 {
                format!("if ({}) {{ {} }} else {{ {} }}", val, gen_cexpr(&cs[1]), gen_cexpr(&cs[0]))
            } else {
                format!("let {} = {}\n{};", outputs[0], val, gen_cexpr(&cs[0]))
            }
        }
        CExpr::Switch(_x, _branches) => "{}".to_string()
    }
}

pub fn gen_program(expr: &CExpr) -> String {
    format!("\"use strict\";\n{}", gen_cexpr(expr))
}