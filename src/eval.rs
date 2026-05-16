use std::rc::Rc;

use crate::syntax::{name, Level, Name, Tm, TAG_TT};
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
        Tm::TagType(ls) => Val::TagType(ls.clone()),
        Tm::Tag(l) => Val::Tag(l.clone()),
        Tm::TagRec(p, cases, t) => {
            let cases_v: Vec<(Name, Val)> = cases
                .iter()
                .map(|(l, body)| (l.clone(), eval(env, body)))
                .collect();
            do_tagrec(eval(env, p), cases_v, eval(env, t))
        }
        Tm::Eq(a, x, y) => eq_val(eval(env, a), eval(env, x), eval(env, y)),
        Tm::Refl => Val::Refl,
        Tm::Coe(a, b, p, t) => coe_val(eval(env, a), eval(env, b), eval(env, p), eval(env, t)),
        Tm::Subst(_a, p, x, y, _pr, px) => {
            let p_val = eval(env, p);
            let x_val = eval(env, x);
            let y_val = eval(env, y);
            let px_val = eval(env, px);
            let p_x = apply(p_val.clone(), x_val);
            let p_y = apply(p_val, y_val);
            coe_val(p_x, p_y, Val::Refl, px_val)
        }
        Tm::Coh(_, _, _, _) => Val::Refl,
        Tm::Let(_, _, t, body) => {
            let v = eval(env, t);
            eval(&env.extend(v), body)
        }
        Tm::Irrel => Val::Refl,
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
        Closure::EqUPiCod {
            dom_l,
            cod_l,
            dom_r,
            cod_r,
        } => {
            let body = Closure::EqUPiBody {
                dom_l: dom_l.clone(),
                cod_l: cod_l.clone(),
                dom_r: dom_r.clone(),
                cod_r: cod_r.clone(),
                eq_dom: Rc::new(v),
            };
            Val::Pi(name("a"), dom_r.clone(), body)
        }
        Closure::EqUPiBody {
            dom_l,
            cod_l,
            dom_r,
            cod_r,
            eq_dom,
        } => {
            let a = coe_val(
                (**dom_r).clone(),
                (**dom_l).clone(),
                (**eq_dom).clone(),
                v.clone(),
            );
            let lhs = closure_apply(cod_l, a);
            let rhs = closure_apply(cod_r, v);
            eq_val(Val::U, lhs, rhs)
        }
        Closure::EqUSigmaCod {
            dom_l,
            cod_l,
            dom_r,
            cod_r,
        } => {
            let body = Closure::EqUSigmaBody {
                dom_l: dom_l.clone(),
                cod_l: cod_l.clone(),
                dom_r: dom_r.clone(),
                cod_r: cod_r.clone(),
                eq_dom: Rc::new(v),
            };
            Val::Pi(name("a"), dom_l.clone(), body)
        }
        Closure::EqUSigmaBody {
            dom_l,
            cod_l,
            dom_r,
            cod_r,
            eq_dom,
        } => {
            let a_r = coe_val(
                (**dom_l).clone(),
                (**dom_r).clone(),
                (**eq_dom).clone(),
                v.clone(),
            );
            let lhs = closure_apply(cod_l, v);
            let rhs = closure_apply(cod_r, a_r);
            eq_val(Val::U, lhs, rhs)
        }
        Closure::CoePiBody {
            dom_l,
            cod_l,
            dom_r,
            cod_r,
            proof,
            fun,
        } => {
            let fst_p = do_fst((**proof).clone());
            let snd_p = do_snd((**proof).clone());
            let a = coe_val((**dom_r).clone(), (**dom_l).clone(), fst_p, v.clone());
            let f_a = apply((**fun).clone(), a.clone());
            let b_l_at = closure_apply(cod_l, a);
            let b_r_at = closure_apply(cod_r, v.clone());
            let snd_p_at = apply(snd_p, v);
            coe_val(b_l_at, b_r_at, snd_p_at, f_a)
        }
    }
}

pub fn apply(f: Val, x: Val) -> Val {
    match f {
        Val::Lam(_, cl) => closure_apply(&cl, x),
        Val::Stuck(h, mut sp) => {
            sp.push(Elim::App(x));
            Val::Stuck(h, sp)
        }
        Val::Refl => Val::Refl,
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
        Val::Refl => Val::Refl,
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
        Val::Refl => Val::Refl,
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

pub fn do_tagrec(p: Val, cases: Vec<(Name, Val)>, t: Val) -> Val {
    match t {
        Val::Tag(ref l) => match cases.iter().find(|(cl, _)| cl == l) {
            Some((_, body)) => body.clone(),
            None => panic!("tagrec: missing case for tag `{l}"),
        },
        Val::Stuck(h, mut sp) => {
            sp.push(Elim::TagRec(p, cases));
            Val::Stuck(h, sp)
        }
        v => panic!("tagrec: not a tag: {v:?}"),
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
            (Val::Zero, Val::Zero) => val_unit(),
            (Val::Zero, Val::Suc(_)) | (Val::Suc(_), Val::Zero) => val_empty(),
            (Val::Suc(m), Val::Suc(n)) => eq_val(Val::Nat, (*m).clone(), (*n).clone()),
            _ => Val::Eq(Rc::new(a), Rc::new(x), Rc::new(y)),
        },
        Val::TagType(_) => match (x.clone(), y.clone()) {
            (Val::Tag(a_l), Val::Tag(b_l)) => {
                if a_l == b_l {
                    val_unit()
                } else {
                    val_empty()
                }
            }
            _ => Val::Eq(Rc::new(a), Rc::new(x), Rc::new(y)),
        },
        Val::U => eq_at_universe(x, y),
        Val::Eq(_, _, _) => val_unit(),
        _ => Val::Eq(Rc::new(a), Rc::new(x), Rc::new(y)),
    }
}

/// Pujet-Tabareau Figure 4: reduction of $\mathsf{Eq}\, \mathcal{U}\, X\, Y$
/// by recursion on the heads of $X$ and $Y$.
fn eq_at_universe(x: Val, y: Val) -> Val {
    if struct_eq(&x, &y) {
        return val_unit();
    }
    match (&x, &y) {
        (Val::U, Val::U) | (Val::Nat, Val::Nat) => val_unit(),
        (Val::TagType(l1), Val::TagType(l2)) => {
            if l1 == l2 {
                val_unit()
            } else {
                val_empty()
            }
        }
        (Val::Pi(_, ad, bd), Val::Pi(_, ac, bc)) => {
            let eq_dom = eq_at_universe((**ac).clone(), (**ad).clone());
            let cod = Closure::EqUPiCod {
                dom_l: ad.clone(),
                cod_l: Rc::new(bd.clone()),
                dom_r: ac.clone(),
                cod_r: Rc::new(bc.clone()),
            };
            Val::Sigma(name("e0"), Rc::new(eq_dom), cod)
        }
        (Val::Sigma(_, ad, bd), Val::Sigma(_, ac, bc)) => {
            let eq_dom = eq_at_universe((**ad).clone(), (**ac).clone());
            let cod = Closure::EqUSigmaCod {
                dom_l: ad.clone(),
                cod_l: Rc::new(bd.clone()),
                dom_r: ac.clone(),
                cod_r: Rc::new(bc.clone()),
            };
            Val::Sigma(name("e0"), Rc::new(eq_dom), cod)
        }
        (Val::Eq(..), Val::Eq(..)) => val_unit(),
        _ => {
            if is_canonical_type(&x) && is_canonical_type(&y) {
                val_empty()
            } else {
                Val::Eq(Rc::new(Val::U), Rc::new(x), Rc::new(y))
            }
        }
    }
}

fn is_canonical_type(v: &Val) -> bool {
    matches!(
        v,
        Val::U | Val::Nat | Val::TagType(_) | Val::Pi(..) | Val::Sigma(..) | Val::Eq(..)
    )
}

pub fn val_unit() -> Val {
    Val::TagType(vec![name(TAG_TT)])
}

pub fn val_empty() -> Val {
    Val::TagType(vec![])
}

pub fn coe_val(a: Val, b: Val, p: Val, t: Val) -> Val {
    if struct_eq(&a, &b) {
        return t;
    }
    match (&a, &b) {
        (Val::Pi(_, ad, bd), Val::Pi(_, ac, bc)) => Val::Lam(
            name("a"),
            Closure::CoePiBody {
                dom_l: ad.clone(),
                cod_l: Rc::new(bd.clone()),
                dom_r: ac.clone(),
                cod_r: Rc::new(bc.clone()),
                proof: Rc::new(p),
                fun: Rc::new(t),
            },
        ),
        (Val::Sigma(_, ad, bd), Val::Sigma(_, ac, bc)) => match t.clone() {
            Val::Pair(t_a, t_b) => {
                let fst_p = do_fst(p.clone());
                let snd_p = do_snd(p);
                let a_r = coe_val((**ad).clone(), (**ac).clone(), fst_p, (*t_a).clone());
                let b_l_at = closure_apply(bd, (*t_a).clone());
                let b_r_at = closure_apply(bc, a_r.clone());
                let snd_p_at = apply(snd_p, (*t_a).clone());
                let b_r = coe_val(b_l_at, b_r_at, snd_p_at, (*t_b).clone());
                Val::Pair(Rc::new(a_r), Rc::new(b_r))
            }
            _ => Val::Coe(Rc::new(a), Rc::new(b), Rc::new(p), Rc::new(t)),
        },
        _ => Val::Coe(Rc::new(a), Rc::new(b), Rc::new(p), Rc::new(t)),
    }
}

fn struct_eq(x: &Val, y: &Val) -> bool {
    match (x, y) {
        (Val::U, Val::U)
        | (Val::Nat, Val::Nat)
        | (Val::Zero, Val::Zero)
        | (Val::Refl, Val::Refl) => true,
        (Val::Suc(a), Val::Suc(b)) => struct_eq(a, b),
        (Val::TagType(a), Val::TagType(b)) => a == b,
        (Val::Tag(a), Val::Tag(b)) => a == b,
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
        (Elim::NatRec(p1, z1, s1), Elim::NatRec(p2, z2, s2)) => {
            struct_eq(p1, p2) && struct_eq(z1, z2) && struct_eq(s1, s2)
        }
        (Elim::TagRec(p1, cs1), Elim::TagRec(p2, cs2)) => {
            if !struct_eq(p1, p2) || cs1.len() != cs2.len() {
                return false;
            }
            cs1.iter()
                .zip(cs2.iter())
                .all(|((l1, v1), (l2, v2))| l1 == l2 && struct_eq(v1, v2))
        }
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
        Val::TagType(ls) => Tm::TagType(ls.clone()),
        Val::Tag(l) => Tm::Tag(l.clone()),
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
            Elim::TagRec(p, cs) => {
                let cases_tm: Vec<(Name, Rc<Tm>)> = cs
                    .iter()
                    .map(|(l, v)| (l.clone(), Rc::new(quote(lvl, v))))
                    .collect();
                Tm::TagRec(Rc::new(quote(lvl, p)), cases_tm, Rc::new(acc))
            }
        };
    }
    acc
}

pub fn nf(env: &Env, tm: &Tm) -> Tm {
    quote(env.len(), &eval(env, tm))
}

/// Type-aware quoting.
///
/// Whenever the type whnf-reduces to $\mathsf{Eq}$, the value is erased
/// to [`Tm::Irrel`] (Pujet-Tabareau proof-irrelevance for equality).
/// Eta-decomposes through concrete $\Pi$/$\Sigma$ introductions and
/// falls back to [`quote`] elsewhere.
pub fn quote_typed(lvl: Level, ty: &Val, v: &Val) -> Tm {
    match ty {
        Val::Eq(_, _, _) => Tm::Irrel,
        Val::Pi(_, _, cod) => match v {
            Val::Lam(n, cl) => {
                let arg = Val::var(lvl);
                let body = closure_apply(cl, arg.clone());
                let body_ty = closure_apply(cod, arg);
                Tm::Lam(n.clone(), Rc::new(quote_typed(lvl + 1, &body_ty, &body)))
            }
            _ => quote(lvl, v),
        },
        Val::Sigma(_, dom, cod) => match v {
            Val::Pair(a, b) => {
                let a_v = (**a).clone();
                let b_v = (**b).clone();
                let cod_at = closure_apply(cod, a_v.clone());
                Tm::Pair(
                    Rc::new(quote_typed(lvl, dom, &a_v)),
                    Rc::new(quote_typed(lvl, &cod_at, &b_v)),
                )
            }
            _ => quote(lvl, v),
        },
        _ => quote(lvl, v),
    }
}

pub fn conv(lvl: Level, v1: &Val, v2: &Val) -> bool {
    match (v1, v2) {
        (Val::U, Val::U)
        | (Val::Nat, Val::Nat)
        | (Val::Zero, Val::Zero)
        | (Val::Refl, Val::Refl) => true,
        (Val::TagType(a), Val::TagType(b)) => a == b,
        (Val::Tag(a), Val::Tag(b)) => a == b,
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
        (Elim::TagRec(p1, cs1), Elim::TagRec(p2, cs2)) => {
            if !conv(lvl, p1, p2) || cs1.len() != cs2.len() {
                return false;
            }
            cs1.iter()
                .zip(cs2.iter())
                .all(|((l1, v1), (l2, v2))| l1 == l2 && conv(lvl, v1, v2))
        }
        _ => false,
    })
}
