// use std::process::Command;
// use crate::{codegen, cps, parser, typecheck};
// use crate::ast::{TypeEnv, Value};
// 
// pub fn run(src: &str) -> String {
//     let decl = parser::(src).unwrap();
//     typecheck::typecheck(&decl, &TypeEnv::new()).unwrap();
//     let cps = cps::to_cps(&decl, Value::Var("console.log".to_string()), &mut 0);
//     let js = codegen::gen_program(&cps);
// 
//     let output = Command::new("node")
//         .arg("-e")
//         .arg(&js)
//         .output()
//         .unwrap();
// 
//     String::from_utf8(output.stdout)
//         .unwrap()
//         .trim()
//         .to_string()
// }