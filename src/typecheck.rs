use crate::ast::{BinOp, Decl, Expr, Type, TypeEnv, TypeError};
use chumsky::prelude::Spanned;

pub fn typecheck(decl: &Decl, env: &TypeEnv) -> Result<Type, TypeError> {
    match decl {
        Decl::Let(_, _, body) => typecheck_expr(body, env),
        Decl::LetRec(f, ty, body) => typecheck_expr(body, &env.update(f.to_string(), ty.clone())),
        Decl::Expr(e) => typecheck_expr(e, env)
    }
}

pub fn typecheck_expr(expr: &Spanned<Expr>, env: &TypeEnv) -> Result<Type, TypeError> {
    match &expr.inner {
        Expr::Var(name) =>
            env.get(name.clone()).cloned()
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
                    let ty_body = typecheck_expr(&**body, &env.update(v.to_string(), ty_provided.clone()))?;
                    Ok(Type::Fn(Box::new(ty_provided.clone()), Box::new(ty_body)))
                }
                None => panic!("not yet implemented: please label types in fn!"),
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
            typecheck_expr(in_, &env.update(v.to_string(), v_ty?))
        }
    }
}