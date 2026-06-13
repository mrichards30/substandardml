use compiler::parser;

fn main() {
    println!("{:?}", parser::prs("let x: num = 5 in x + 1"));
    println!("{:?}", parser::prs("let x: bool = true in if x then true else false"));
    println!("{:?}", parser::prs("fn x: bool => if x then true else false"));
    println!("{:?}", parser::prs("fn x => if x then true else false"));
    println!("{:?}", parser::prs("fn x: num => if x >= 5 then true else false"));
    println!("{:?}", parser::prs("(fn x: num => if x >= 5 then true else false) 6"));
    println!("{:?}", parser::prs("(fn x: num => if x >= 5 then true else false)(6)"));
    println!("{:?}", parser::prs("(fn x: num => if x >= 5 then true else false)6"));

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
