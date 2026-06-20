use chumsky::container::Seq;
use crate::ast::Type::Tyvar;
use crate::ast::{BinOp, Decl, Expr, Type, TypeEnv, TypeError};
use chumsky::prelude::Spanned;
use im::{HashMap, HashSet};

pub fn typecheck(decl: &Decl, env: &TypeEnv) -> Result<Type, TypeError> {
    match decl {
        Decl::Let(_, _, body) => typecheck_expr(body, env),
        Decl::LetRec(f, ty, body) => typecheck_expr(body, &env.upd_env(f.to_string(), ty.clone())),
        Decl::Expr(e) => typecheck_expr(e, env)
    }
}

pub fn typecheck_expr(expr: &Spanned<Expr>, env: &TypeEnv) -> Result<Type, TypeError> {
    match &expr.inner {
        Expr::Var(name) =>
            env.get_env(name.to_string())
            .ok_or_else(|| TypeError::UnboundVariable(name.to_string())),
        Expr::Num(_) => Ok(Type::Num),
        Expr::Bool(_) => Ok(Type::Bool),
        Expr::Unit => Ok(Type::Unit),
        Expr::If(cond_, then_, else_) => {
            let cond_type = typecheck_expr(&**cond_, env)?;
            if cond_type != Type::Bool {
                return Err(TypeError::TypeMismatch { expected: Type::Bool, found: cond_type });
            }
            let then_type = typecheck_expr(&**then_, env)?;
            let else_type = typecheck_expr(&**else_, env)?;
            if then_type != else_type {
                return Err(TypeError::TypeMismatch { expected: then_type, found: else_type });
            }
            Ok(then_type)
        }
        Expr::Fn(v, ty, body) => {
            match ty {
                Some(ty_provided) => {
                    let ty_body = typecheck_expr(&**body, &env.upd_env(v.to_string(), ty_provided.clone()))?;
                    Ok(Type::Fn(Box::new(ty_provided.clone()), Box::new(ty_body)))
                }
                None => {
                    let new_tyvar = gen_tyvar(body.clone(), env);
                    let ty_body = typecheck_expr(&**body, &env.upd_env(v.to_string(), new_tyvar.clone()))?;
                    Ok(Type::Fn(Box::new(new_tyvar), Box::new(ty_body)))
                },
            }
        }
        Expr::App(e1, e2) => {
            let t1 = typecheck_expr(e1, env)?;
            let t2 = typecheck_expr(e2, env)?;
            match t1 {
                Type::Fn(rand, rator) => {
                    if *rand == t2 {
                        Ok(*rator)
                    } else {
                        Err(TypeError::TypeMismatch { expected: *rand, found: t2 })
                    }
                }
                _ => Err(TypeError::NotAFunction(t1))
            }
        }
        Expr::Seq(lhs, rhs) => {
            typecheck(lhs, env)?;
            typecheck_expr(rhs, env)
        }
        Expr::BinOp(op, e1, e2) => {
            let t1 = typecheck_expr(e1, env)?;
            let t2 = typecheck_expr(e2, env)?;
            match (t1, t2) {
                (Type::Num, Type::Num) => match &op.inner {
                    BinOp::Eq | BinOp::Neq | BinOp::Geq | BinOp::Gt | BinOp::Leq | BinOp::Lt => Ok(Type::Bool),
                    BinOp::Plus | BinOp::Minus | BinOp::Times | BinOp::Div => Ok(Type::Num),
                },
                (Type::Num, t) => Err(TypeError::TypeMismatch { expected: Type::Num, found: t }),
                (t, _) => Err(TypeError::TypeMismatch { expected: Type::Num, found: t }),
            }
        }
        Expr::Neg(e) => typecheck_expr(e, env),
        Expr::LetIn(v, ty, body, in_) => {
            let ty_inferred = typecheck_expr(body, env)?;
            let v_ty = match ty {
                None => Ok(ty_inferred),
                Some(ty_provided) if *ty_provided == ty_inferred => Ok(ty_provided.clone()),
                Some(ty_provided) => Err(TypeError::TypeMismatch { expected: ty_provided.clone(), found: ty_inferred })
            };
            typecheck_expr(in_, &env.upd_env(v.to_string(), v_ty?))
        }
    }
}

fn tyvars_in_decl(decl: Box<Decl>, env: &TypeEnv, acc: HashSet<String>) -> HashSet<String> {
    match *decl {
        Decl::Let(_, _, _) => HashSet::new(),
        Decl::LetRec(_, _, _) => HashSet::new(),
        Decl::Expr(e1) => tyvars_in(Box::new(e1), env, acc)
    }
}

fn tyvars_in(expr: Box<Spanned<Expr>>, env: &TypeEnv, acc: HashSet<String>) -> HashSet<String> {
    match expr.inner {
        Expr::Var(n) =>
            env.get_env(n.to_string())
                .map(|v| acc.clone().union(tyvars_in_ty(v)))
                .unwrap_or(acc),
        Expr::Num(_) => acc,
        Expr::Bool(_) => acc,
        Expr::Unit => acc,
        Expr::If(t1, t2, t3) =>
            tyvars_in(t1, env, tyvars_in(t2, env, tyvars_in(t3, env, acc))),
        Expr::LetIn(_, ty, t1, t2) =>
            tyvars_in(t1, env, tyvars_in(t2, env, ty
                .map(|v| acc.clone().union(tyvars_in_ty(v)))
                .unwrap_or(acc))),
        Expr::Fn(_, ty, t1) =>
            tyvars_in(t1, env, ty
                .map(|v| acc.clone().union(tyvars_in_ty(v)))
                .unwrap_or(acc)),
        Expr::App(t1, t2) =>
            tyvars_in(t1, env, tyvars_in(t2, env, acc)),
        Expr::Seq(t1, t2) =>
            tyvars_in_decl(t1, env, tyvars_in(t2, env, acc)),
        Expr::Neg(t1) => tyvars_in(t1, env, acc),
        Expr::BinOp(_, t1, t2) =>
            tyvars_in(t1, env, tyvars_in(t2, env, acc))
    }
}

fn tyvars_in_ty(ty: Type) -> HashSet<String> {
    match ty {
        Type::Num => HashSet::new(),
        Type::Bool => HashSet::new(),
        Type::Unit => HashSet::new(),
        Type::Fn(t1, t2) =>
            HashSet::new()
                .union(tyvars_in_ty(*t1))
                .union(tyvars_in_ty(*t2)),
        Tyvar(s) => HashSet::new().update(s),
    }
}

// TODO swap above hashsets for unique sorted lists which preserves semantics
//      but probably skips a lot of this work
fn idx_to_tvar(mut num: i32) -> String {
    if num == 0 {
        return "a".to_string();
    }
    let mut result = String::new();
    while num > 0 {
        num -= 1;
        let remainder = (num % 26) as u8;
        let current = (b'a' + remainder) as char;
        result.push(current);
        num /= 26;
    }
    result.chars().rev().collect()
}

#[test]
fn idx_to_var_tests() {
    assert_eq!(idx_to_tvar(1), "a");
    assert_eq!(idx_to_tvar(5), "e");
    assert_eq!(idx_to_tvar(26), "z");
    assert_eq!(idx_to_tvar(27), "aa");
    assert_eq!(idx_to_tvar(200), "gr");
}

pub fn gen_tyvar(expr: Box<Spanned<Expr>>, env: &TypeEnv) -> Type {
    let tyvars = tyvars_in(expr, env, HashSet::new());
    let mut idx = 0;
    while tyvars.contains(&idx_to_tvar(idx)) {
        idx += 1;
    }
    Tyvar(idx_to_tvar(idx))
}

fn type_subst(ty: Type, substs: HashMap<String, Type>) -> Type {
    match ty.clone() {
        Type::Num => ty,
        Type::Bool => ty,
        Type::Unit => ty,
        Type::Fn(ty1, ty2) =>
        Type::Fn(Box::new(type_subst(*ty1, substs.clone())), Box::new(type_subst(*ty2, substs))),
        Tyvar(x) => substs.get(&x).unwrap_or(&ty).clone(),
    }
}
