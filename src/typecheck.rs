use crate::ast::Type::Tyvar;
use crate::ast::{BinOp, Decl, Expr, Type, TypeEnv, TypeError};
use chumsky::prelude::Spanned;
use chumsky::span::SpanWrap;
use im::{HashMap, HashSet};

pub fn typecheck(decl: &Decl, env: &mut TypeEnv) -> Result<(Type, HashSet<Constraint>), TypeError> {
    match decl {
        Decl::Let(_, _, body) => typecheck_expr(body, env),
        Decl::LetRec(f, ty, body) => {
            env.upd_env(f.to_string(), ty.clone());
            typecheck_expr(body, env)
        }
        Decl::Expr(e) => typecheck_expr(e, env),
    }
}

// TODO better type here; no reflexivity in set<constraint> atm
type Constraint = (Type, Type);

pub fn typecheck_expr(
    expr: &Spanned<Expr>,
    env: &mut TypeEnv,
) -> Result<(Type, HashSet<Constraint>), TypeError> {
    let (ty, constraints) = match &expr.inner {
        Expr::Var(name) => {
            if let Some(ty) = env.get_env(name.to_string()) {
                Ok((ty, HashSet::new()))
            } else {
                Err(TypeError::UnboundVariable(name.to_string()))
            }
        }
        Expr::Num(_) => Ok((Type::Num, HashSet::new())),
        Expr::Bool(_) => Ok((Type::Bool, HashSet::new())),
        Expr::Unit => Ok((Type::Unit, HashSet::new())),
        Expr::If(t1, t2, t3) => {
            let (ty1, c1) = typecheck_expr(t1, env)?;
            let (ty2, c2) = typecheck_expr(t2, env)?;
            let (ty3, c3) = typecheck_expr(t3, env)?;
            let new_cs = c1
                .union(c2)
                .union(c3)
                .update((ty1, Type::Bool))
                .update((ty2.clone(), ty3));
            Ok((ty2, new_cs))
        }
        Expr::Fn(v, ty, body) => {
            let ty_provided = ty.clone().unwrap_or(gen_tyvar(body.clone(), env));
            env.upd_env(v.to_string(), ty_provided.clone());
            let (ty_body, cs) = typecheck_expr(&**body, env)?;
            Ok((
                Type::Fn(Box::new(ty_provided.clone()), Box::new(ty_body)),
                cs,
            ))
        }
        Expr::App(t1, t2) => {
            // FIXME these should not be mutating env
            let (ty1, c1) = typecheck_expr(t1, env)?;
            let (ty2, c2) = typecheck_expr(t2, env)?;
            // TODO transcribe the lots of side conditions from figure 22-1 from pierce
            // TODO also genvar needs to be over a set of terms and the below line fixed
            let fresh_tyvar = gen_tyvar(t1.clone(), env);
            let new_cs = c1
                .union(c2)
                .update((ty1, Type::Fn(Box::new(ty2), Box::new(fresh_tyvar.clone()))));
            Ok((fresh_tyvar, new_cs))
        }
        Expr::Seq(lhs, rhs) => {
            // FIXME these should not be mutating env
            typecheck(lhs, env)?;
            typecheck_expr(rhs, env)
        }
        Expr::BinOp(op, e1, e2) if is_comparison_op(op) => {
            let (ty1, c1) = typecheck_expr(e1, env)?;
            let (ty2, c2) = typecheck_expr(e2, env)?;
            let new_cs = c1.union(c2).update((ty1, ty2));
            Ok((Type::Bool, new_cs))
        }
        Expr::BinOp(_, e1, e2) => {
            // i.e., num ops like plus, minus, etc.
            let (ty1, c1) = typecheck_expr(e1, env)?;
            let (ty2, c2) = typecheck_expr(e2, env)?;
            let new_cs = c1
                .union(c2)
                .update((ty1, Type::Num))
                .update((ty2, Type::Num));
            Ok((Type::Num, new_cs))
        }
        Expr::Neg(e) => typecheck_expr(e, env),
        Expr::LetIn(v, ty, t1, t2) => {
            let (_, _) = typecheck_expr(t1, env)?; // otherwise t1 isnt checked if v not in fvs of t2
            let (ty2, c) = typecheck_expr(&vsubst(v, t1, t2).with_span(expr.span), env)?;
            Ok((ty2, c))
        }
    }?;
    match unify(constraints.clone()) {
        None => todo!(),
        Some(unification) => Ok((ty, constraints)),
    }
}

fn vsubst<'a>(
    v: &'a Spanned<&'a str>,
    t1: &'a Box<Spanned<Expr<'a>>>,
    t2: &'a Box<Spanned<Expr<'a>>>,
) -> Expr<'a> {
    use Expr::*;
    match &t2.inner {
        Var(v2) => {
            if v.inner == *v2 {
                t1.inner.clone()
            } else {
                Var(v2)
            }
        }
        Num(_) | Bool(_) | Unit => t2.inner.clone(),
        If(cond_, then_, else_) => If(
            Box::new(vsubst(v, t1, cond_).with_span(cond_.span)),
            Box::new(vsubst(v, t1, then_).with_span(then_.span)),
            Box::new(vsubst(v, t1, else_).with_span(else_.span)),
        ),
        LetIn(spanned, _, spanned1, spanned2) => todo!(),
        Fn(spanned, _, spanned1) => todo!(),
        App(a, b) => App(
            Box::new(vsubst(v, t1, a).with_span(a.span)),
            Box::new(vsubst(v, t1, b).with_span(b.span)),
        ),
        Seq(a, b) => Seq(
            Box::new(vsubst_decl(v, t1, a)),
            Box::new(vsubst(v, t1, b).with_span(b.span)),
        ),
        Neg(expr) => Neg(Box::new(vsubst(v, t1, expr).with_span(expr.span))),
        BinOp(op, lhs, rhs) => BinOp(
            op.clone(),
            Box::new(vsubst(v, t1, lhs).with_span(lhs.span)),
            Box::new(vsubst(v, t1, rhs).with_span(rhs.span)),
        ),
    }
    // todo!()
}

fn vsubst_decl<'a>(
    v: &'a Spanned<&'a str>,
    t1: &'a Box<Spanned<Expr<'a>>>,
    t2: &'a Box<Decl<'a>>,
) -> Decl<'a> {
    todo!()
}

fn is_comparison_op(op: &Spanned<BinOp>) -> bool {
    use BinOp::*;
    match op.inner {
        Eq | Neq | Geq | Gt | Leq | Lt => true,
        _ => false,
    }
}

fn tyvars_in_decl(decl: Box<Decl>, env: &TypeEnv) -> HashSet<String> {
    match *decl {
        Decl::Let(_, _, _) => HashSet::new(),
        Decl::LetRec(_, _, _) => HashSet::new(),
        Decl::Expr(e1) => tyvars_in(Box::new(e1), env),
    }
}

fn tyvars_in(expr: Box<Spanned<Expr>>, env: &TypeEnv) -> HashSet<String> {
    match expr.inner {
        Expr::Var(n) => {
            if let Some(v) = env.get_env(n.to_string()) {
                tyvars_in_ty(v)
            } else {
                HashSet::new()
            }
        }
        Expr::Num(_) => HashSet::new(),
        Expr::Bool(_) => HashSet::new(),
        Expr::Unit => HashSet::new(),
        Expr::If(t1, t2, t3) => tyvars_in(t1, env)
            .union(tyvars_in(t2, env))
            .union(tyvars_in(t3, env)),
        Expr::LetIn(_, ty, t1, t2) => {
            let acc = tyvars_in(t1, env).union(tyvars_in(t2, env));
            if let Some(v) = ty {
                acc.union(tyvars_in_ty(v))
            } else {
                acc
            }
        }
        Expr::Fn(_, ty, t1) => {
            let acc = tyvars_in(t1, env);
            if let Some(v) = ty {
                acc.union(tyvars_in_ty(v))
            } else {
                acc
            }
        }
        Expr::App(t1, t2) => tyvars_in(t1, env).union(tyvars_in(t2, env)),
        Expr::Seq(t1, t2) => tyvars_in_decl(t1, env).union(tyvars_in(t2, env)),
        Expr::Neg(t1) => tyvars_in(t1, env),
        Expr::BinOp(_, t1, t2) => tyvars_in(t1, env).union(tyvars_in(t2, env)),
    }
}

fn tyvars_in_ty(ty: Type) -> HashSet<String> {
    match ty {
        Type::Num => HashSet::new(),
        Type::Bool => HashSet::new(),
        Type::Unit => HashSet::new(),
        Type::Fn(t1, t2) => HashSet::new()
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
    let tyvars = tyvars_in(expr, env);
    let mut idx = 0;
    while tyvars.contains(&idx_to_tvar(idx)) {
        idx += 1;
    }
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

fn decls_fvs(decl: &Decl, acc: HashSet<String>) -> HashSet<String> {
    match decl.clone() {
        Decl::Let(v, _, e) => fvs(&e, acc).without(&v),
        Decl::LetRec(v, _, e) => fvs(&e, acc).without(&v),
        Decl::Expr(e) => fvs(&e, acc),
    }
}

fn fvs(expr: &Spanned<Expr>, acc: HashSet<String>) -> HashSet<String> {
    match expr.inner {
        Expr::Var(n) => acc.update(n.to_string()),
        Expr::Num(_) => acc,
        Expr::Bool(_) => acc,
        Expr::Unit => acc,
        Expr::If(ref e1, ref e2, ref e3) => fvs(&*e1, fvs(&*e2, fvs(&*e3, acc))),
        Expr::LetIn(v, _, ref e1, ref e2) => fvs(&e1, fvs(&e2, acc)).without(v.inner),
        Expr::Fn(v, _, ref e1) => fvs(&e1, acc).without(v.inner),
        Expr::App(ref e1, ref e2) => fvs(&*e1, fvs(&*e2, acc)),
        Expr::Seq(ref e1, ref e2) => decls_fvs(&*e1, fvs(&*e2, acc)),
        Expr::Neg(ref e1) => fvs(&*e1, acc),
        Expr::BinOp(_, ref e1, ref e2) => fvs(&*e1, fvs(&*e2, acc)),
    }
}

// TODO tests
#[test]
fn test_fvs() {
    // todo!()
}

fn unify(c: HashSet<Constraint>) -> Option<Vec<(Type, Type)>> {
    let mut c = c.into_iter();
    if let Some((s, t)) = c.next() {
        if s == t {
            return unify(c.collect());
        } else {
            todo!()
        }
    } else {
        return Some(vec![]);
    }
}
