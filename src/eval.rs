use std::rc::Rc;

use crate::syntax::{name, Level, Tm};
use crate::value::{Closure, Elim, Env, Head, Spine, Val};

pub fn eval(env: &Env, tm: &Tm) -> Val {
    match tm {
        Tm::Var(ix) => env.lookup(*ix),
        Tm::Lam(n, body) => Val::Lam(n.clone(), Closure::Body(env.clone(), body.clone())),
        Tm::App(f, x) => apply(eval(env, f), eval(env, x)),
        Tm::Pi(n, a, b) => Val::Pi(
            n.clone(),
            Rc::new(eval(env, a)),
            Closure::Body(env.clone(), b.clone()),
        ),
        Tm::Sigma(n, a, b) => Val::Sigma(
            n.clone(),
            Rc::new(eval(env, a)),
            Closure::Body(env.clone(), b.clone()),
        ),
        Tm::Pair(a, b) => Val::Pair(Rc::new(eval(env, a)), Rc::new(eval(env, b))),
        Tm::Fst(t) => do_fst(eval(env, t)),
        Tm::Snd(t) => do_snd(eval(env, t)),
        Tm::U => Val::U,
        Tm::Nat => Val::Nat,
        Tm::Zero => Val::Zero,
        Tm::Suc(n) => Val::Suc(Rc::new(eval(env, n))),
        Tm::NatRec(p, z, s, n) => do_natrec(eval(env, p), eval(env, z), eval(env, s), eval(env, n)),
        Tm::Bool => Val::Bool,
        Tm::BTrue => Val::BTrue,
        Tm::BFalse => Val::BFalse,
        Tm::BoolRec(p, t, f, b) => {
            do_boolrec(eval(env, p), eval(env, t), eval(env, f), eval(env, b))
        }
        Tm::Unit => Val::Unit,
        Tm::TT => Val::TT,
        Tm::Empty => Val::Empty,
        Tm::EmptyRec(p, e) => do_emptyrec(eval(env, p), eval(env, e)),
        Tm::Eq(a, x, y) => eq_val(eval(env, a), eval(env, x), eval(env, y)),
        Tm::Refl => Val::Refl,
        Tm::Coe(a, b, p, t) => coe_val(eval(env, a), eval(env, b), eval(env, p), eval(env, t)),
        Tm::Let(_, _, t, body) => {
            let v = eval(env, t);
            eval(&env.extend(v), body)
        }
    }
}

pub fn closure_apply(cl: &Closure, v: Val) -> Val {
    match cl {
        Closure::Body(env, body) => eval(&env.extend(v), body),
        Closure::EqPi(cod, f, g) => {
            let bv = closure_apply(cod, v.clone());
            let fv = apply((**f).clone(), v.clone());
            let gv = apply((**g).clone(), v);
            eq_val(bv, fv, gv)
        }
        Closure::ReflPi(_) => Val::Refl,
        Closure::Const(v) => (**v).clone(),
    }
}

pub fn apply(f: Val, x: Val) -> Val {
    match f {
        Val::Lam(_, cl) => closure_apply(&cl, x),
        Val::Stuck(h, mut sp) => {
            sp.push(Elim::App(x));
            Val::Stuck(h, sp)
        }
        Val::Coe(..) => panic!("apply on stuck coercion: unsupported"),
        v => panic!("apply: not a function: {v:?}"),
    }
}

pub fn do_fst(v: Val) -> Val {
    match v {
        Val::Pair(a, _) => (*a).clone(),
        Val::Stuck(h, mut sp) => {
            sp.push(Elim::Fst);
            Val::Stuck(h, sp)
        }
        v => panic!("fst: not a pair: {v:?}"),
    }
}

pub fn do_snd(v: Val) -> Val {
    match v {
        Val::Pair(_, b) => (*b).clone(),
        Val::Stuck(h, mut sp) => {
            sp.push(Elim::Snd);
            Val::Stuck(h, sp)
        }
        v => panic!("snd: not a pair: {v:?}"),
    }
}

pub fn do_natrec(p: Val, z: Val, s: Val, n: Val) -> Val {
    match n {
        Val::Zero => z,
        Val::Suc(m) => {
            let rec = do_natrec(p, z, s.clone(), (*m).clone());
            apply(apply(s, (*m).clone()), rec)
        }
        Val::Stuck(h, mut sp) => {
            sp.push(Elim::NatRec(p, z, s));
            Val::Stuck(h, sp)
        }
        v => panic!("natrec: not a Nat: {v:?}"),
    }
}

pub fn do_boolrec(p: Val, t: Val, f: Val, b: Val) -> Val {
    match b {
        Val::BTrue => t,
        Val::BFalse => f,
        Val::Stuck(h, mut sp) => {
            sp.push(Elim::BoolRec(p, t, f));
            Val::Stuck(h, sp)
        }
        v => panic!("boolrec: not a Bool: {v:?}"),
    }
}

pub fn do_emptyrec(p: Val, e: Val) -> Val {
    match e {
        Val::Stuck(h, mut sp) => {
            sp.push(Elim::EmptyRec(p));
            Val::Stuck(h, sp)
        }
        v => panic!("emptyrec: not an Empty: {v:?}"),
    }
}

pub fn eq_val(a: Val, x: Val, y: Val) -> Val {
    match a.clone() {
        Val::Pi(n, dom, cod) => {
            let cod2 = Closure::EqPi(Rc::new(cod), Rc::new(x), Rc::new(y));
            Val::Pi(n, dom, cod2)
        }
        Val::Sigma(_, dom, cod) => {
            let xa = do_fst(x.clone());
            let xb = do_snd(x);
            let ya = do_fst(y.clone());
            let yb = do_snd(y);
            let cod_at = closure_apply(&cod, xa.clone());
            let eq_a = eq_val((*dom).clone(), xa, ya);
            let eq_b = eq_val(cod_at, xb, yb);
            Val::Sigma(name("_"), Rc::new(eq_a), Closure::Const(Rc::new(eq_b)))
        }
        Val::Nat => match (x.clone(), y.clone()) {
            (Val::Zero, Val::Zero) => Val::Unit,
            (Val::Zero, Val::Suc(_)) | (Val::Suc(_), Val::Zero) => Val::Empty,
            (Val::Suc(m), Val::Suc(n)) => eq_val(Val::Nat, (*m).clone(), (*n).clone()),
            _ => Val::Eq(Rc::new(a), Rc::new(x), Rc::new(y)),
        },
        Val::Bool => match (x.clone(), y.clone()) {
            (Val::BTrue, Val::BTrue) | (Val::BFalse, Val::BFalse) => Val::Unit,
            (Val::BTrue, Val::BFalse) | (Val::BFalse, Val::BTrue) => Val::Empty,
            _ => Val::Eq(Rc::new(a), Rc::new(x), Rc::new(y)),
        },
        Val::U => {
            if struct_eq(&x, &y) {
                Val::Unit
            } else {
                Val::Eq(Rc::new(a), Rc::new(x), Rc::new(y))
            }
        }
        Val::Unit => Val::Unit,
        Val::Empty => Val::Unit,
        Val::Eq(_, _, _) => Val::Unit,
        _ => Val::Eq(Rc::new(a), Rc::new(x), Rc::new(y)),
    }
}

pub fn coe_val(a: Val, b: Val, p: Val, t: Val) -> Val {
    if struct_eq(&a, &b) {
        t
    } else {
        Val::Coe(Rc::new(a), Rc::new(b), Rc::new(p), Rc::new(t))
    }
}

fn struct_eq(x: &Val, y: &Val) -> bool {
    match (x, y) {
        (Val::U, Val::U)
        | (Val::Nat, Val::Nat)
        | (Val::Bool, Val::Bool)
        | (Val::Unit, Val::Unit)
        | (Val::Empty, Val::Empty)
        | (Val::Zero, Val::Zero)
        | (Val::BTrue, Val::BTrue)
        | (Val::BFalse, Val::BFalse)
        | (Val::TT, Val::TT)
        | (Val::Refl, Val::Refl) => true,
        (Val::Suc(a), Val::Suc(b)) => struct_eq(a, b),
        (Val::Pair(a1, b1), Val::Pair(a2, b2)) => struct_eq(a1, a2) && struct_eq(b1, b2),
        (Val::Stuck(Head::Var(l1), sp1), Val::Stuck(Head::Var(l2), sp2)) => {
            l1 == l2 && spine_struct_eq(sp1, sp2)
        }
        (Val::Eq(a1, x1, y1), Val::Eq(a2, x2, y2)) => {
            struct_eq(a1, a2) && struct_eq(x1, x2) && struct_eq(y1, y2)
        }
        _ => false,
    }
}

fn spine_struct_eq(s1: &Spine, s2: &Spine) -> bool {
    if s1.len() != s2.len() {
        return false;
    }
    s1.iter().zip(s2.iter()).all(|(a, b)| match (a, b) {
        (Elim::App(x), Elim::App(y)) => struct_eq(x, y),
        (Elim::Fst, Elim::Fst) | (Elim::Snd, Elim::Snd) => true,
        _ => false,
    })
}

pub fn quote(lvl: Level, v: &Val) -> Tm {
    match v {
        Val::Lam(n, cl) => {
            let body = closure_apply(cl, Val::var(lvl));
            Tm::Lam(n.clone(), Rc::new(quote(lvl + 1, &body)))
        }
        Val::Pi(n, dom, cod) => {
            let body = closure_apply(cod, Val::var(lvl));
            Tm::Pi(
                n.clone(),
                Rc::new(quote(lvl, dom)),
                Rc::new(quote(lvl + 1, &body)),
            )
        }
        Val::Sigma(n, dom, cod) => {
            let body = closure_apply(cod, Val::var(lvl));
            Tm::Sigma(
                n.clone(),
                Rc::new(quote(lvl, dom)),
                Rc::new(quote(lvl + 1, &body)),
            )
        }
        Val::Pair(a, b) => Tm::Pair(Rc::new(quote(lvl, a)), Rc::new(quote(lvl, b))),
        Val::U => Tm::U,
        Val::Nat => Tm::Nat,
        Val::Zero => Tm::Zero,
        Val::Suc(n) => Tm::Suc(Rc::new(quote(lvl, n))),
        Val::Bool => Tm::Bool,
        Val::BTrue => Tm::BTrue,
        Val::BFalse => Tm::BFalse,
        Val::Unit => Tm::Unit,
        Val::TT => Tm::TT,
        Val::Empty => Tm::Empty,
        Val::Eq(a, x, y) => Tm::Eq(
            Rc::new(quote(lvl, a)),
            Rc::new(quote(lvl, x)),
            Rc::new(quote(lvl, y)),
        ),
        Val::Refl => Tm::Refl,
        Val::Coe(a, b, p, t) => Tm::Coe(
            Rc::new(quote(lvl, a)),
            Rc::new(quote(lvl, b)),
            Rc::new(quote(lvl, p)),
            Rc::new(quote(lvl, t)),
        ),
        Val::Stuck(h, sp) => quote_stuck(lvl, h, sp),
    }
}

fn quote_stuck(lvl: Level, h: &Head, sp: &Spine) -> Tm {
    let head_tm = match h {
        Head::Var(l) => Tm::Var(lvl - l - 1),
    };
    let mut acc = head_tm;
    for e in sp {
        acc = match e {
            Elim::App(v) => Tm::App(Rc::new(acc), Rc::new(quote(lvl, v))),
            Elim::Fst => Tm::Fst(Rc::new(acc)),
            Elim::Snd => Tm::Snd(Rc::new(acc)),
            Elim::NatRec(p, z, s) => Tm::NatRec(
                Rc::new(quote(lvl, p)),
                Rc::new(quote(lvl, z)),
                Rc::new(quote(lvl, s)),
                Rc::new(acc),
            ),
            Elim::BoolRec(p, t, f) => Tm::BoolRec(
                Rc::new(quote(lvl, p)),
                Rc::new(quote(lvl, t)),
                Rc::new(quote(lvl, f)),
                Rc::new(acc),
            ),
            Elim::EmptyRec(p) => Tm::EmptyRec(Rc::new(quote(lvl, p)), Rc::new(acc)),
        };
    }
    acc
}

pub fn nf(env: &Env, tm: &Tm) -> Tm {
    quote(env.len(), &eval(env, tm))
}

pub fn conv(lvl: Level, v1: &Val, v2: &Val) -> bool {
    match (v1, v2) {
        (Val::U, Val::U)
        | (Val::Nat, Val::Nat)
        | (Val::Bool, Val::Bool)
        | (Val::Unit, Val::Unit)
        | (Val::Empty, Val::Empty)
        | (Val::Zero, Val::Zero)
        | (Val::BTrue, Val::BTrue)
        | (Val::BFalse, Val::BFalse)
        | (Val::TT, Val::TT)
        | (Val::Refl, Val::Refl) => true,
        (Val::Suc(a), Val::Suc(b)) => conv(lvl, a, b),
        (Val::Pi(_, d1, c1), Val::Pi(_, d2, c2))
        | (Val::Sigma(_, d1, c1), Val::Sigma(_, d2, c2)) => {
            if !conv(lvl, d1, d2) {
                return false;
            }
            let a = Val::var(lvl);
            conv(
                lvl + 1,
                &closure_apply(c1, a.clone()),
                &closure_apply(c2, a),
            )
        }
        (Val::Lam(_, c1), Val::Lam(_, c2)) => {
            let a = Val::var(lvl);
            conv(
                lvl + 1,
                &closure_apply(c1, a.clone()),
                &closure_apply(c2, a),
            )
        }
        (Val::Lam(_, c), other) | (other, Val::Lam(_, c)) => {
            let a = Val::var(lvl);
            let lhs = closure_apply(c, a.clone());
            let rhs = apply(other.clone(), a);
            conv(lvl + 1, &lhs, &rhs)
        }
        (Val::Pair(a1, b1), Val::Pair(a2, b2)) => conv(lvl, a1, a2) && conv(lvl, b1, b2),
        (Val::Pair(a, b), other) | (other, Val::Pair(a, b)) => {
            conv(lvl, a, &do_fst(other.clone())) && conv(lvl, b, &do_snd(other.clone()))
        }
        (Val::Eq(a1, x1, y1), Val::Eq(a2, x2, y2)) => {
            conv(lvl, a1, a2) && conv(lvl, x1, x2) && conv(lvl, y1, y2)
        }
        (Val::Coe(a1, b1, _, t1), Val::Coe(a2, b2, _, t2)) => {
            conv(lvl, a1, a2) && conv(lvl, b1, b2) && conv(lvl, t1, t2)
        }
        (Val::Stuck(h1, s1), Val::Stuck(h2, s2)) => match (h1, h2) {
            (Head::Var(l1), Head::Var(l2)) => l1 == l2 && conv_spine(lvl, s1, s2),
        },
        _ => false,
    }
}

fn conv_spine(lvl: Level, s1: &Spine, s2: &Spine) -> bool {
    if s1.len() != s2.len() {
        return false;
    }
    s1.iter().zip(s2.iter()).all(|(a, b)| match (a, b) {
        (Elim::App(x), Elim::App(y)) => conv(lvl, x, y),
        (Elim::Fst, Elim::Fst) | (Elim::Snd, Elim::Snd) => true,
        (Elim::NatRec(p1, z1, s1), Elim::NatRec(p2, z2, s2)) => {
            conv(lvl, p1, p2) && conv(lvl, z1, z2) && conv(lvl, s1, s2)
        }
        (Elim::BoolRec(p1, t1, f1), Elim::BoolRec(p2, t2, f2)) => {
            conv(lvl, p1, p2) && conv(lvl, t1, t2) && conv(lvl, f1, f2)
        }
        (Elim::EmptyRec(p1), Elim::EmptyRec(p2)) => conv(lvl, p1, p2),
        _ => false,
    })
}
