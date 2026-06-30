use compiler::node;

/*
 * These tests pipe the generated code into node to execute 
 * the js and assert against their outputs.
 */

const TRUE: &str = "1";
const FALSE: &str = "0";

#[test]
fn test_addition() {
    assert_eq!(node::run("1 + 2 + 3"), "6");
    assert_eq!(node::run("1 + 2 + 3 + 4 + 5"), "15");
}

#[test]
fn test_if() {
    assert_eq!(node::run("if true then 1 else 2"), "1");
    assert_eq!(node::run("if false then 1 else 2"), "2");
    assert_eq!(node::run("if 1 + 2 >= 3 then true else false"), "1"); // no bools in our cps for now
}

#[test]
fn test_fn() {
    assert_eq!(node::run("(fn x: num => x) 3"), "3");
    assert_eq!(node::run("(fn x: num => if x >= 0 then true else false) 6"), TRUE);
    assert_eq!(node::run("(fn x: num => if x >= 0 then true else false) (-4)"), FALSE);
}

#[test]
fn test_let() {
    assert_eq!(node::run("let x = 5 in x"), "5");
}

#[test]
fn test_big() {
    assert_eq!(node::run("let f: num -> num = fn x: num => x + 1 in f 4"), "5");
}
