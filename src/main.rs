use compiler::ast::TypeEnv;
use compiler::parser;
use compiler::typecheck::typecheck_expr;

fn main() {
    // println!("{:?}", parser::prs("let x: num = 5 in x + 1"));
    // println!("{:?}", parser::prs("let x: bool = true in if x then true else false"));
    // println!("{:?}", parser::prs("fn x: bool => if x then true else false"));
    // println!("{:?}", parser::prs("fn x => if x then true else false"));
    // println!("{:?}", parser::prs("fn x: num => if x >= 5 then true else false"));
    // println!("{:?}", parser::prs("(fn x: num => if x >= 5 then true else false) 6"));
    // println!("{:?}", parser::prs("(fn x: num => if x >= 5 then true else false)(6)"));
    // println!("{:?}", parser::prs("(fn x: num => if x >= 5 then true else false)6"));

    println!("{:?}", typecheck_expr(&parser::prs("let x: num = 5 in x + 1"), &TypeEnv::new()));

}
