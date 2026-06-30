use compiler::ast::Type::{Bool, Fn, Num, Tyvar};
use compiler::ast::{lower, Ast, Type, TypeEnv, TypeError};
use compiler::parser;
use compiler::typecheck::typecheck;

#[test]
fn test_atom_types() {
    assert_type_ok("1", Num);
    assert_type_ok("false", Bool);
    assert_type_ok("true", Bool);
}

#[test]
fn test_if_else_types() {
    assert_type_ok("if true then 1 else 2", Num);
    assert_type_ok("if 1 + 2 >= 3 then 1 else 2", Num);
    assert_type_ok(
        "if true then (if true then 3 else 4) else (if true then 5 else 6)",
        Num,
    );
    assert_type_err(
        "if true then 1 else true",
        TypeError::TypeMismatch {
            found: Bool,
            expected: Num,
        },
    );
}

#[test]
fn test_let_in_types() {
    assert_type_ok("let x: num = 1 in x", Num);
    assert_type_ok("let x = 1 in x", Num);
    assert_type_ok("let x = 1 in true", Bool);
    assert_type_ok("let x = 1 in 1 >= x", Bool);
    assert_type_err(
        "let x: bool = 3 in false",
        TypeError::TypeMismatch {
            found: Bool,
            expected: Num,
        },
    );
}

#[test]
fn test_monomorphic_fn_types() {
    assert_type_ok("fn x: num => 3", Fn(Box::new(Num), Box::new(Num)));
    assert_type_ok("fn x: num => x", Fn(Box::new(Num), Box::new(Num)));
    assert_type_err(
        "(fn x: num => 3) true",
        TypeError::TypeMismatch {
            found: Bool,
            expected: Num,
        },
    );
}

#[test]
fn test_polymorphic_fn_types() {
    assert_type_ok(
        "fn x: 'a => 3",
        Fn(Box::new(Tyvar("a".to_string())), Box::new(Num)),
    );
    assert_type_ok(
        "fn x: 'a -> 'b => 3",
        Fn(
            Box::new(Fn(
                Box::new(Tyvar("a".to_string())),
                Box::new(Tyvar("b".to_string())),
            )),
            Box::new(Num),
        ),
    );
    assert_type_ok(
        "fn x => 3",
        Fn(Box::new(Tyvar("a".to_string())), Box::new(Num)),
    );
    assert_type_ok("(fn x => 3) true", Num);
    assert_type_ok(
        "fn x => fn y => 0",
        Fn(
            Box::new(Tyvar("c".to_string())),
            Box::new(Fn(Box::new(Tyvar("d".to_string())), Box::new(Num))),
        ),
    );
    assert_type_ok(
        "(fn x => fn y => 0) 5",
        Fn(Box::new(Tyvar("f".to_string())), Box::new(Num)),
    );
}

// TODO these need to use unification to check equality, I think, to prevent them from being brittle.
fn assert_type_ok(src: &str, ty: Type) {
    let res = parser::parse(src).unwrap();
    let ast = &mut Ast::new();
    let id = lower(ast, res);
    assert_eq!(
        typecheck(ast, id, &TypeEnv::new()),
        Ok(ty),
        "{}",
        src
    );
}

fn assert_type_err(src: &str, err: TypeError) {
    let res = parser::parse(src).unwrap();
    let ast = &mut Ast::new();
    let id = lower(ast, res);
    assert_eq!(
        typecheck(ast, id, &TypeEnv::new()),
        Err(err),
        "{}",
        src
    )
}
