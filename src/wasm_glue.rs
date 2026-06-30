use crate::ast::{lower, Ast, CpsAst, TypeEnv, Value};
use crate::{codegen, cps, parser, typecheck};
use std::panic;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(getter_with_clone)]
pub struct JsResult {
    pub js: String,
    pub errors: Vec<String>,
}

#[wasm_bindgen(start)]
pub fn init_compiler() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
}

#[wasm_bindgen]
pub fn compile_to_js(src: String) -> JsResult {
    let ast = &mut Ast::new();
    let cps_ast = &mut CpsAst::new();
    let decl = parser::parse(&src).map(|e| lower(ast, e));
    match decl {
        Ok(expr_id) => {
            match typecheck::typecheck(ast, expr_id, &mut TypeEnv::new()) {
                Ok(_) => {
                    let logger = cps_ast.push_val(Value::Var("console.log".to_string()), (0, 0));
                    let cps = cps::expr_to_cps(
                        ast,
                        expr_id,
                        cps_ast,
                        logger,
                        &mut 0,
                    );
                    JsResult {
                        js: codegen::gen_program(cps_ast, cps),
                        errors: vec![],
                    }
                }
                Err(_) => JsResult {
                    js: "false".to_string(),
                    errors: vec!["type error".to_string()],
                },
            }
        },
        Err(err) => JsResult {
            js: "false".to_string(),
            errors: err.iter().map(|e| e.to_string()).collect(),
        },
    }
}
