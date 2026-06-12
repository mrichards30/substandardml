use crate::ast::{BinOp, Decl, Expr, Type, TypeEnv, TypeError};

pub fn typecheck(decl: &Decl, env: &TypeEnv) -> Result<Type, TypeError> {
    match decl {
        Decl::Let(_, _, body) => typecheck_expr(body, env),
        Decl::LetRec(f, ty, body) => typecheck_expr(body, &env.update(f.to_string(), ty.clone())),
        Decl::Expr(e) => typecheck_expr(e, env)
    }
}

pub fn typecheck_expr(expr: &Expr, env: &TypeEnv) -> Result<Type, TypeError> {
    match expr {
        Expr::Var(name) => 
            env.get(name).cloned()
            .ok_or_else(|| TypeError::UnboundVariable(name.clone())),
        Expr::Int(_) => Ok(Type::Int),
        Expr::Bool(_) => Ok(Type::Bool),
        Expr::Unit => Ok(Type::Unit),
        Expr::If(cond_, then_, else_) => {
            let cond_type = typecheck_expr(cond_, env)?;
            if cond_type != Type::Bool {
                return Err(TypeError::TypeMismatch { expected: Type::Bool, found: cond_type });
            }
            let then_type = typecheck_expr(then_, env)?;
            let else_type = typecheck_expr(else_, env)?;
            if then_type != else_type {
                return Err(TypeError::TypeMismatch { expected: then_type, found: else_type });
            }
            Ok(then_type)
        }
        Expr::Fn(v, ty, body) => {
            let body_ty = typecheck_expr(body, &env.update(v.clone(), ty.clone()))?;
            Ok(Type::Fn(Box::new(ty.clone()), Box::new(body_ty)))
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
                (Type::Int, Type::Int) => match op {
                    BinOp::Eq | BinOp::Neq | BinOp::Geq | BinOp::Gt | BinOp::Leq | BinOp::Lt => Ok(Type::Bool),
                    BinOp::Plus | BinOp::Minus | BinOp::Times | BinOp::Div => Ok(Type::Int),
                },
                (Type::Int, t) => Err(TypeError::TypeMismatch { expected: Type::Int, found: t }),
                (t, _) => Err(TypeError::TypeMismatch { expected: Type::Int, found: t }),
            }
        }
        Expr::LetIn(x, ty, body, in_) => {
            let ty_found = typecheck_expr(body, env)?;
            if ty.clone() == ty_found {
                typecheck_expr(in_, &env.update(x.clone(), ty.clone()))
            } else {
                Err(TypeError::TypeMismatch { expected: ty.clone(), found: ty_found })
            }
        }
    }
}