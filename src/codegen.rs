use crate::ast::{CExpr, Value, BinOp, CpsAst, ValueId, ExprId};
use itertools::Itertools;

pub fn gen_value(ast: &CpsAst, val: ValueId) -> String {
    match &ast.vals[val] {
        Value::Var(n) => n.clone(),
        Value::Label(l) => l.clone(),
        Value::Num(n) => n.to_string(),
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

fn dest_var(ast: &CpsAst, v: ValueId) -> String {
    match &ast.vals[v] {
        Value::Var(n) => n.clone(),
        _ => panic!("not a var")
    }
}

pub fn gen_cexpr(ast: &CpsAst, id: ExprId) -> String {
    match &ast.exprs[id] {
        CExpr::App(f, xs) =>
            format!("{}({})", gen_value(ast, *f), xs.iter().map(|x| gen_value(ast, *x)).collect::<Vec<_>>().join(", ")),
        CExpr::Fix(vs, body) =>
            format!("{}\n{}",
                    vs.iter().map(|(f, params, b)| {
                        let f_name = dest_var(ast, *f);
                        format!("function {}({}) {{ {} }}", f_name, params.iter().map(|v| dest_var(ast, *v)).join(", "), gen_cexpr(ast, *b))
                    }).collect::<Vec<_>>().join("\n"),
                    gen_cexpr(ast, *body)),
        CExpr::PrimOp(op, inputs, outputs, cs) => {
            let val = inputs.iter().map(|e| gen_value(ast, *e)).collect::<Vec<_>>().join(&format!(" {} ", gen_binop(*op)));
            if cs.len() == 2 {
                format!("if ({}) {{ {} }} else {{ {} }}", val, gen_cexpr(ast, cs[1]), gen_cexpr(ast, cs[0]))
            } else {
                format!("let {} = {}\n{};", dest_var(ast, outputs[0]), val, gen_cexpr(ast, cs[0]))
            }
        }
        CExpr::Switch(_x, _branches) => "{}".to_string()
    }
}

pub fn gen_program(ast: &CpsAst, id: ExprId) -> String {
    format!("\"use strict\";\n{}", gen_cexpr(ast, id))
}