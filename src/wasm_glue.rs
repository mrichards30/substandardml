use crate::ast::{Decl, TypeEnv, Value};
use crate::{codegen, cps, parser, typecheck};
use std::panic;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(getter_with_clone)]
struct JsResult {
    pub js: String,
    pub errors: Vec<String>
}

#[wasm_bindgen(start)]
pub fn init_compiler() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
}

#[wasm_bindgen]
pub fn compile_to_js(src: String) -> JsResult {
    let decl = parser::safe_prs(&src);
    match decl {
        Ok(e) => {
            match typecheck::typecheck_expr(&e, &TypeEnv::new()) {
                Ok(typ) => {
                    let cps = cps::to_cps(&Decl::Expr(e), Value::Var("console.log".to_string()), &mut 0);
                    JsResult{ js: codegen::gen_program(&cps), errors: vec![] }
                }
                Err(err) => {
                    JsResult{ js: "false".to_string(), errors: vec!["type error".to_string()] }
                }
            }
        }
        Err(err) => {
            JsResult{ js: "false".to_string(), errors: vec![err.to_string()] }
        }
    }
}