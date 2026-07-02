use crate::ast::{lower, Ast, CpsAst, Type, TypeEnv, Value};
use crate::{codegen, cps, parser, typecheck};
use std::process::Command;

pub fn run1(src: &str) -> (Type, String) {
    let pexpr = parser::parse(src).unwrap();
    let ast = &mut Ast::new();
    let id = lower(ast, pexpr);
    let ty = typecheck::typecheck(ast, id, &TypeEnv::new()).unwrap();
    let cps_ast = &mut CpsAst::new();
    let k = cps_ast.push_val(Value::Var("console.log".to_string()), (0, 0));
    let cps = cps::expr_to_cps(
        ast,
        id,
        cps_ast,
        k,
        &mut 0,
    );
    let js = codegen::gen_program(cps_ast, cps);

    // println!("{}", js);

    let output = Command::new("node").arg("-e").arg(&js).output().unwrap();

    (ty, String::from_utf8(output.stdout).unwrap().trim().to_string())
}

pub fn run(src: &str) -> String {
    run1(src).1
}
