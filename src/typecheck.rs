use std::sync::atomic::{AtomicI32, Ordering};

use crate::ast::Type::Tyvar;
use crate::ast::{Ast, BinOp, Expr, ExprId, Scheme, Type, TypeEnv, TypeError};
use im::{HashMap, HashSet};
use itertools::Itertools;

static TYPE_VARIABLE: AtomicI32 = AtomicI32::new(0);

type Constraint = (Type, Type);

fn typecheck_expr(
    ast: &Ast,
    expr_id: ExprId,
    env: &TypeEnv,
) -> Result<(Type, HashSet<Constraint>), TypeError> {
    match &ast.exprs[expr_id] {
        Expr::Var(name) => {
            if let Some(scheme) = env.get_env(name.to_string()) {
                Ok((inst(scheme), HashSet::new()))
            } else {
                Err(TypeError::UnboundVariable(name.to_string()))
            }
        }
        Expr::Num(_) => Ok((Type::Num, HashSet::new())),
        Expr::Bool(_) => Ok((Type::Bool, HashSet::new())),
        Expr::Unit => Ok((Type::Unit, HashSet::new())),
        Expr::If(t1, t2, t3) => {
            let (ty1, c1) = typecheck_expr(ast, *t1, env)?;
            let (ty2, c2) = typecheck_expr(ast, *t2, env)?;
            let (ty3, c3) = typecheck_expr(ast, *t3, env)?;
            let new_cs = c1
                .union(c2)
                .union(c3)
                .update((ty1, Type::Bool))
                .update((ty2.clone(), ty3));
            Ok((ty2, new_cs))
        }
        Expr::Fn(v, Some(ty_provided), body) => {
            let (ty_body, cs) = typecheck_expr(ast, *body, &env.upd_env(v.to_string(), (vec![], ty_provided.clone())))?;
            Ok((Type::Fn(Box::new(ty_provided.clone()), Box::new(ty_body)), cs))
        }
        Expr::Fn(v, None, body) => {
            let x = gen_tyvar();
            let (t, c) = typecheck_expr(ast, *body, &env.upd_env(v.to_string(), (vec![], x.clone())))?;
            Ok((Type::Fn(Box::new(x.clone()), Box::new(t)), c))
        }
        Expr::App(t1, t2) => {
            let (ty1, c1) = typecheck_expr(ast, *t1, env)?;
            let (ty2, c2) = typecheck_expr(ast, *t2, env)?;
            let fresh_tyvar = gen_tyvar();
            let new_cs = c1
                .union(c2)
                .update((ty1, Type::Fn(Box::new(ty2), Box::new(fresh_tyvar.clone()))));
            Ok((fresh_tyvar, new_cs))
        }
        Expr::Seq(lhs, rhs) => {
            let (ty1, _) = typecheck_expr(ast, *lhs, env)?;
            let (ty2, cs) = typecheck_expr(ast, *rhs, env)?;
            Ok((ty2, cs.update((ty1, Type::Unit))))
        }
        Expr::BinOp(op, e1, e2) if is_comparison_op(op) => {
            let (ty1, c1) = typecheck_expr(ast, *e1, env)?;
            let (ty2, c2) = typecheck_expr(ast, *e2, env)?;
            let new_cs = c1.union(c2).update((ty1, ty2));
            Ok((Type::Bool, new_cs))
        }
        Expr::BinOp(_, e1, e2) => {
            // i.e., num ops like plus, minus, etc.
            let (ty1, c1) = typecheck_expr(ast, *e1, env)?;
            let (ty2, c2) = typecheck_expr(ast, *e2, env)?;
            let new_cs = c1
                .union(c2)
                .update((ty1, Type::Num))
                .update((ty2, Type::Num));
            Ok((Type::Num, new_cs))
        }
        Expr::Neg(e) => typecheck_expr(ast, *e, env),
        Expr::LetIn(x, Some(ty), t1, t2) => {
            let (ty1, _) = typecheck_expr(ast, *t1, env)?;
            let (ty2, c2) = typecheck_expr(ast, *t2, &env.upd_env(x.to_string(), (vec![], ty.clone())))?;
            Ok((ty2, c2.update((ty1, ty.clone()))))
        }
        Expr::LetIn(x, None, t1, t2) => {
            let (s1, c1) = typecheck_expr(ast, *t1, env)?;
            let sigma = unify1(c1)?;
            let pt1 = type_subst(s1, sigma);
            let pts1 = generalise(env, pt1);
            typecheck_expr(ast, *t2, &env.upd_env(x.to_string(), pts1))
        }
    }
}

fn unify1(cs: HashSet<Constraint>) -> Result<HashMap<String, Type>, TypeError> {
    match unify(cs) {
        Err((expected, found)) => Err(TypeError::TypeMismatch { expected, found }),
        Ok(sigma) => {
            let res: HashMap<String, Type> = sigma.iter().map(|e|
                match &e {
                    (Tyvar(n), r) => (n.to_string(), r.clone()),
                    (l, Tyvar(n)) => (n.to_string(), l.clone()),
                    _ => panic!("neither side is a tyvar"),
                }).collect();
            Ok(res)
        }
    }
}

fn inst(scheme: Scheme) -> Type {
    let (binders, ty) = scheme;
    let substs = binders.iter()
        .map(|e| (e.to_string(), gen_tyvar()))
        .collect();
    type_subst(ty, substs)
}

fn tyvars_in_env(env: &TypeEnv) -> HashSet<String> {
    env.get_env_map().values()
        .map(|(_, e)| tyvars_in_ty(e))
        .flatten()
        .collect()
}

fn generalise(env: &TypeEnv, ty: Type) -> Scheme {
    let env_tvs = tyvars_in_env(env);
    let new_vars = tyvars_in_ty(&ty).iter()
        .filter(|e| !env_tvs.contains(e.to_owned()))
        .map(|e| e.to_string())
        .collect_vec();
    (new_vars, ty)
}

pub fn typecheck(ast: &mut Ast, expr_id: ExprId, env: &TypeEnv) -> Result<Type, TypeError> {
    let (ty, cs) = typecheck_expr(ast, expr_id, env)?;
    let sigma = unify1(cs)?;
    Ok(type_subst(ty, sigma))
}

fn is_comparison_op(op: &BinOp) -> bool {
    use BinOp::*;
    match op {
        Eq | Neq | Geq | Gt | Leq | Lt => true,
        _ => false,
    }
}

fn tyvars_in_ty(ty: &Type) -> HashSet<String> {
    use Type::*;
    match ty {
        Num => HashSet::new(),
        Bool => HashSet::new(),
        Unit => HashSet::new(),
        Fn(t1, t2) => HashSet::new()
            .union(tyvars_in_ty(&**t1))
            .union(tyvars_in_ty(&**t2)),
        Tyvar(s) => HashSet::new().update(s.clone()),
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

pub fn gen_tyvar() -> Type {
    let idx = TYPE_VARIABLE.fetch_add(1, Ordering::Relaxed);
    Tyvar(idx_to_tvar(idx))
}

fn type_subst(ty: Type, substs: HashMap<String, Type>) -> Type {
    match ty {
        Type::Num => ty.clone(),
        Type::Bool => ty.clone(),
        Type::Unit => ty.clone(),
        Type::Fn(ty1, ty2) => Type::Fn(
            Box::new(type_subst(*ty1, substs.clone())),
            Box::new(type_subst(*ty2, substs)),
        ),
        Tyvar(ref x) => substs.get(&*x).unwrap_or(&ty).clone(),
    }
}

// Either returns the substitutions or the failed constraint
fn unify(c: HashSet<Constraint>) -> Result<Vec<(Type, Type)>, Constraint> {
    use Type::*;
    if c.is_empty() {
        return Ok(Vec::new());
    }
    let mut c = c.into_iter();
    let (s, t) = c.next().unwrap();
    let rest: HashSet<Constraint> = c.collect();

    if s == t {
        unify(rest)
    } else if let Tyvar(x) = s.clone()
        && !tyvars_in_ty(&t).contains(&x)
    {
        // S = X and X not in FVs(T)
        unify(
            rest.iter()
                .map(|(ty1, ty2)| {
                    let ty1_prime = match ty1 {
                        Tyvar(v) if *v == x => t.clone(),
                        _ => ty1.clone(),
                    };
                    let ty2_prime = match ty2 {
                        Tyvar(v) if *v == x => t.clone(),
                        _ => ty2.clone(),
                    };
                    (ty1_prime, ty2_prime)
                })
                .collect(),
        )
            .map(|mut subst| {
                subst.push((s.clone(), t.clone()));
                subst
            })
    } else if let Tyvar(x) = t.clone()
        && !tyvars_in_ty(&s).contains(&x)
    {
        // T = X and X not in FVs(S)
        unify(
            rest.iter()
                .map(|(ty1, ty2)| {
                    let ty1_prime = match ty1 {
                        Tyvar(v) if *v == x => s.clone(),
                        _ => ty1.clone(),
                    };
                    let ty2_prime = match ty2 {
                        Tyvar(v) if *v == x => s.clone(),
                        _ => ty2.clone(),
                    };
                    (ty1_prime, ty2_prime)
                })
                .collect(),
        )
            .map(|mut subst| {
                subst.push((s.clone(), t.clone()));
                subst
            })
    } else {
        match (&s, &t) {
            (Fn(s1, s2), Fn(t1, t2)) => unify(
                rest.update((*s1.clone(), *t1.clone()))
                    .update((*s2.clone(), *t2.clone())),
            ),
            _ => Err((s, t)),
        }
    }
}

mod test {
    use im::HashSet;
    use crate::typecheck::unify;

    #[test]
    fn test_unify_concrete() {
        use super::Type::*;
        let constraints = HashSet::new().update((Num, Num));
        let unification = unify(constraints).unwrap();
        assert!(unification.is_empty());
    }

    #[test]
    fn test_unify_concrete_fail() {
        use super::Type::*;
        let constraints = HashSet::new().update((Num, Bool));
        let unification = unify(constraints);
        assert!(unification.is_err())
    }
}
