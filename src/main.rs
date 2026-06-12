use compiler::parser;

fn main() {
    println!("{:?}", parser::prs("let x: int = 5 in x"));

    // match typecheck(&parser::prs("(fn x: int => x) 5").unwrap(), &TypeEnv::new()) {
    //     Ok(ty) => println!("type: {:?}", ty),
    //     Err(e) => println!("type error: {:?}", e),
    // }

    // match parser::prs("\
    // (fn x: int => \
    //     if x >= 5 \
    //     then 1 else 0\
    // ) 5") {
    //     None => {}
    //     Some(decl) => {
    //         let top_k = Value::Var("console.log".to_string());
    //         let cps = to_cps(&decl, top_k, &mut 0);
    //         let js = gen_program(&cps);
    //         println!("{}", js);
    //     }
    // }

}
