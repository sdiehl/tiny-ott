use crate::eval::{apply, closure_apply, conv, do_fst, eq_val, eval, quote};
use crate::pretty::pretty_tm;
use crate::syntax::{name, Index, Level, Name, Raw, Tm, TAG_TT};
use crate::value::{Closure, Env, Val};
use std::rc::Rc;

#[derive(Clone, Default, Debug)]
pub struct Cxt {
    pub env: Env,
    pub types: Vec<Val>,
    pub names: Vec<Name>,
}

impl Cxt {
    pub fn level(&self) -> Level {
        self.env.len()
    }

    #[must_use]
    pub fn bind(&self, n: Name, ty: Val) -> Self {
        let mut cx = self.clone();
        let lvl = cx.env.len();
        cx.env.0.push(Val::var(lvl));
        cx.types.push(ty);
        cx.names.push(n);
        cx
    }

    #[must_use]
    pub fn define(&self, n: Name, v: Val, ty: Val) -> Self {
        let mut cx = self.clone();
        cx.env.0.push(v);
        cx.types.push(ty);
        cx.names.push(n);
        cx
    }

    pub fn lookup(&self, n: &Name) -> Option<(Index, Val)> {
        for i in (0..self.names.len()).rev() {
            if &self.names[i] == n {
                let ix = self.names.len() - 1 - i;
                return Some((ix, self.types[i].clone()));
            }
        }
        None
    }
}

fn err(cx: &Cxt, msg: &str) -> String {
    format!("{msg} (at depth {})", cx.level())
}

fn show_val(cx: &Cxt, v: &Val) -> String {
    pretty_tm(&cx.names, &quote(cx.level(), v))
}

fn non_dep_closure(cx: &Cxt, body: &Val) -> Closure {
    let body_tm = quote(cx.level() + 1, body);
    Closure::Body(cx.env.clone(), Rc::new(body_tm))
}

pub fn check(cx: &Cxt, raw: &Raw, expected: &Val) -> Result<Tm, String> {
    if matches!(expected, Val::Eq(..)) {
        let _ = check_dispatch(cx, raw, expected)?;
        return Ok(Tm::Irrel);
    }
    check_dispatch(cx, raw, expected)
}

fn check_dispatch(cx: &Cxt, raw: &Raw, expected: &Val) -> Result<Tm, String> {
    match raw {
        Raw::Lam(ns, body) => check_lam(cx, ns, body, expected),
        Raw::Pair(a, b) => match expected {
            Val::Sigma(_, dom, cod) => {
                let a_tm = check(cx, a, dom)?;
                let a_val = eval(&cx.env, &a_tm);
                let cod_at = closure_apply(cod, a_val);
                let b_tm = check(cx, b, &cod_at)?;
                Ok(Tm::Pair(Rc::new(a_tm), Rc::new(b_tm)))
            }
            other => Err(err(
                cx,
                &format!("pair checked against non-Sigma: {}", show_val(cx, other)),
            )),
        },
        Raw::Refl => check_refl(cx, expected),
        Raw::Tag(l) => match expected {
            Val::TagType(ls) => {
                if ls.iter().any(|x| x == l) {
                    Ok(Tm::Tag(l.clone()))
                } else {
                    Err(err(
                        cx,
                        &format!("tag `{l} not a member of {}", show_val(cx, expected)),
                    ))
                }
            }
            other => Err(err(
                cx,
                &format!(
                    "tag `{l} checked against non-TagType: {}",
                    show_val(cx, other)
                ),
            )),
        },
        Raw::Let(n, ty, val, body) => {
            let ty_tm = check(cx, ty, &Val::U)?;
            let ty_val = eval(&cx.env, &ty_tm);
            let val_tm = check(cx, val, &ty_val)?;
            let val_v = eval(&cx.env, &val_tm);
            let cx2 = cx.define(n.clone(), val_v, ty_val);
            let body_tm = check(&cx2, body, expected)?;
            Ok(Tm::Let(
                n.clone(),
                Rc::new(ty_tm),
                Rc::new(val_tm),
                Rc::new(body_tm),
            ))
        }
        _ => {
            let (tm, inferred) = infer(cx, raw)?;
            if conv(cx.level(), &inferred, expected) {
                Ok(tm)
            } else {
                Err(err(
                    cx,
                    &format!(
                        "type mismatch\n  expected: {}\n  inferred: {}",
                        show_val(cx, expected),
                        show_val(cx, &inferred)
                    ),
                ))
            }
        }
    }
}

fn check_lam(cx: &Cxt, ns: &[Name], body: &Raw, expected: &Val) -> Result<Tm, String> {
    if ns.is_empty() {
        return check(cx, body, expected);
    }
    let n = &ns[0];
    let rest = &ns[1..];
    match expected {
        Val::Pi(_, dom, cod) => {
            let lvl = cx.level();
            let cx2 = cx.bind(n.clone(), (**dom).clone());
            let cod_at = closure_apply(cod, Val::var(lvl));
            let body_tm = check_lam(&cx2, rest, body, &cod_at)?;
            Ok(Tm::Lam(n.clone(), Rc::new(body_tm)))
        }
        other => Err(err(
            cx,
            &format!(
                "lambda binder {n} checked against non-Pi: {}",
                show_val(cx, other)
            ),
        )),
    }
}

fn check_refl(cx: &Cxt, expected: &Val) -> Result<Tm, String> {
    match expected {
        Val::TagType(ls) if ls.len() == 1 && ls[0].as_ref() == TAG_TT => Ok(Tm::Tag(name(TAG_TT))),
        Val::TagType(ls) if ls.is_empty() => Err(err(cx, "refl: Empty is not inhabited")),
        Val::Pi(n, dom, cod) => {
            let lvl = cx.level();
            let cx2 = cx.bind(n.clone(), (**dom).clone());
            let body_ty = closure_apply(cod, Val::var(lvl));
            let body_tm = check_refl(&cx2, &body_ty)?;
            Ok(Tm::Lam(n.clone(), Rc::new(body_tm)))
        }
        Val::Sigma(_, dom, cod) => {
            let a_tm = check_refl(cx, dom)?;
            let a_val = eval(&cx.env, &a_tm);
            let cod_at = closure_apply(cod, a_val);
            let b_tm = check_refl(cx, &cod_at)?;
            Ok(Tm::Pair(Rc::new(a_tm), Rc::new(b_tm)))
        }
        Val::Eq(a, x, y) => {
            if conv(cx.level(), x, y) {
                Ok(Tm::Refl)
            } else {
                Err(err(
                    cx,
                    &format!(
                        "refl: terms not convertible\n  lhs: {}\n  rhs: {}\n  at type: {}",
                        show_val(cx, x),
                        show_val(cx, y),
                        show_val(cx, a)
                    ),
                ))
            }
        }
        other => Err(err(
            cx,
            &format!("refl: type does not admit refl: {}", show_val(cx, other)),
        )),
    }
}

pub fn infer(cx: &Cxt, raw: &Raw) -> Result<(Tm, Val), String> {
    match raw {
        Raw::Var(n) => match cx.lookup(n) {
            Some((ix, ty)) => Ok((Tm::Var(ix), ty)),
            None => Err(err(cx, &format!("unbound variable: {n}"))),
        },
        Raw::U => Ok((Tm::U, Val::U)),
        Raw::Nat => Ok((Tm::Nat, Val::U)),
        Raw::Zero => Ok((Tm::Zero, Val::Nat)),
        Raw::TagType(ls) => Ok((Tm::TagType(ls.clone()), Val::U)),
        Raw::Tag(l) => Err(err(
            cx,
            &format!("cannot infer type of tag `{l}; use an annotation"),
        )),
        Raw::NumLit(n) => {
            let mut t = Tm::Zero;
            for _ in 0..*n {
                t = Tm::Suc(Rc::new(t));
            }
            Ok((t, Val::Nat))
        }
        Raw::Suc(t) => {
            let t_tm = check(cx, t, &Val::Nat)?;
            Ok((Tm::Suc(Rc::new(t_tm)), Val::Nat))
        }
        Raw::App(f, x) => {
            let (f_tm, f_ty) = infer(cx, f)?;
            match f_ty {
                Val::Pi(_, dom, cod) => {
                    let x_tm = check(cx, x, &dom)?;
                    let x_val = eval(&cx.env, &x_tm);
                    let res_ty = closure_apply(&cod, x_val);
                    Ok((Tm::App(Rc::new(f_tm), Rc::new(x_tm)), res_ty))
                }
                other => Err(err(
                    cx,
                    &format!("applying non-function of type {}", show_val(cx, &other)),
                )),
            }
        }
        Raw::Pi(ns, dom, body) => {
            let dom_tm = check(cx, dom, &Val::U)?;
            let dom_val = eval(&cx.env, &dom_tm);
            let pi_tm = elab_telescope_pi(cx, ns, &dom_val, body)?;
            let _ = dom_tm;
            Ok((pi_tm, Val::U))
        }
        Raw::Arrow(a, b) => {
            let a_tm = check(cx, a, &Val::U)?;
            let a_val = eval(&cx.env, &a_tm);
            let cx2 = cx.bind(name("_"), a_val);
            let b_tm = check(&cx2, b, &Val::U)?;
            Ok((Tm::Pi(name("_"), Rc::new(a_tm), Rc::new(b_tm)), Val::U))
        }
        Raw::Sigma(ns, dom, body) => {
            let dom_tm = check(cx, dom, &Val::U)?;
            let dom_val = eval(&cx.env, &dom_tm);
            let sig_tm = elab_telescope_sigma(cx, ns, &dom_val, body)?;
            let _ = dom_tm;
            Ok((sig_tm, Val::U))
        }
        Raw::Prod(a, b) => {
            let a_tm = check(cx, a, &Val::U)?;
            let a_val = eval(&cx.env, &a_tm);
            let cx2 = cx.bind(name("_"), a_val);
            let b_tm = check(&cx2, b, &Val::U)?;
            Ok((Tm::Sigma(name("_"), Rc::new(a_tm), Rc::new(b_tm)), Val::U))
        }
        Raw::Pair(a, b) => {
            let (a_tm, a_ty) = infer(cx, a)?;
            let (b_tm, b_ty) = infer(cx, b)?;
            let cod = non_dep_closure(cx, &b_ty);
            Ok((
                Tm::Pair(Rc::new(a_tm), Rc::new(b_tm)),
                Val::Sigma(name("_"), Rc::new(a_ty), cod),
            ))
        }
        Raw::Fst(t) => {
            let (t_tm, t_ty) = infer(cx, t)?;
            match t_ty {
                Val::Sigma(_, dom, _) => Ok((Tm::Fst(Rc::new(t_tm)), (*dom).clone())),
                _ => Err(err(cx, "fst: not a Sigma")),
            }
        }
        Raw::Snd(t) => {
            let (t_tm, t_ty) = infer(cx, t)?;
            match t_ty {
                Val::Sigma(_, _, cod) => {
                    let v = eval(&cx.env, &t_tm);
                    let fst_v = do_fst(v);
                    let cod_at = closure_apply(&cod, fst_v);
                    Ok((Tm::Snd(Rc::new(t_tm)), cod_at))
                }
                _ => Err(err(cx, "snd: not a Sigma")),
            }
        }
        Raw::Eq(a, x, y) => {
            let a_tm = check(cx, a, &Val::U)?;
            let a_val = eval(&cx.env, &a_tm);
            let x_tm = check_dispatch(cx, x, &a_val)?;
            let y_tm = check_dispatch(cx, y, &a_val)?;
            Ok((Tm::Eq(Rc::new(a_tm), Rc::new(x_tm), Rc::new(y_tm)), Val::U))
        }
        Raw::Refl => Err(err(cx, "cannot infer type of refl; use an annotation")),
        Raw::Coe(a, b, p, t) => {
            let a_tm = check(cx, a, &Val::U)?;
            let b_tm = check(cx, b, &Val::U)?;
            let a_val = eval(&cx.env, &a_tm);
            let b_val = eval(&cx.env, &b_tm);
            let eq_u = eq_val(Val::U, a_val.clone(), b_val.clone());
            let p_tm = check(cx, p, &eq_u)?;
            let t_tm = check(cx, t, &a_val)?;
            Ok((
                Tm::Coe(Rc::new(a_tm), Rc::new(b_tm), Rc::new(p_tm), Rc::new(t_tm)),
                b_val,
            ))
        }
        Raw::Subst(a, p, x, y, pr, px) => {
            let a_tm = check(cx, a, &Val::U)?;
            let a_val = eval(&cx.env, &a_tm);
            let p_ty = Val::Pi(
                name("_"),
                Rc::new(a_val.clone()),
                Closure::Body(cx.env.clone(), Rc::new(Tm::U)),
            );
            let p_tm = check(cx, p, &p_ty)?;
            let p_val = eval(&cx.env, &p_tm);
            let x_tm = check(cx, x, &a_val)?;
            let x_val = eval(&cx.env, &x_tm);
            let y_tm = check(cx, y, &a_val)?;
            let y_val = eval(&cx.env, &y_tm);
            let eq_xy = eq_val(a_val, x_val.clone(), y_val.clone());
            let pr_tm = check(cx, pr, &eq_xy)?;
            let p_x = apply(p_val.clone(), x_val);
            let p_y = apply(p_val, y_val);
            let px_tm = check(cx, px, &p_x)?;
            Ok((
                Tm::Subst(
                    Rc::new(a_tm),
                    Rc::new(p_tm),
                    Rc::new(x_tm),
                    Rc::new(y_tm),
                    Rc::new(pr_tm),
                    Rc::new(px_tm),
                ),
                p_y,
            ))
        }
        Raw::Coh(a, b, p, t) => {
            let a_tm = check(cx, a, &Val::U)?;
            let b_tm = check(cx, b, &Val::U)?;
            let a_val = eval(&cx.env, &a_tm);
            let b_val = eval(&cx.env, &b_tm);
            let eq_u = eq_val(Val::U, a_val.clone(), b_val.clone());
            let p_tm = check(cx, p, &eq_u)?;
            let p_val = eval(&cx.env, &p_tm);
            let t_tm = check(cx, t, &a_val)?;
            let t_val = eval(&cx.env, &t_tm);
            let coe_v = crate::eval::coe_val(a_val, b_val.clone(), p_val, t_val.clone());
            let res_ty = Val::Eq(Rc::new(b_val), Rc::new(t_val), Rc::new(coe_v));
            Ok((
                Tm::Coh(Rc::new(a_tm), Rc::new(b_tm), Rc::new(p_tm), Rc::new(t_tm)),
                res_ty,
            ))
        }
        Raw::NatRec(p, z, s, n) => {
            let p_ty = Val::Pi(
                name("_"),
                Rc::new(Val::Nat),
                Closure::Body(cx.env.clone(), Rc::new(Tm::U)),
            );
            let p_tm = check(cx, p, &p_ty)?;
            let p_val = eval(&cx.env, &p_tm);
            let z_ty = apply(p_val.clone(), Val::Zero);
            let z_tm = check(cx, z, &z_ty)?;
            let s_ty = nat_step_ty(cx, &p_val);
            let s_tm = check(cx, s, &s_ty)?;
            let n_tm = check(cx, n, &Val::Nat)?;
            let n_val = eval(&cx.env, &n_tm);
            let res_ty = apply(p_val, n_val);
            Ok((
                Tm::NatRec(Rc::new(p_tm), Rc::new(z_tm), Rc::new(s_tm), Rc::new(n_tm)),
                res_ty,
            ))
        }
        Raw::TagRec(p, cases, t) => {
            let (t_tm, t_ty) = infer(cx, t)?;
            let ls = match &t_ty {
                Val::TagType(ls) => ls.clone(),
                other => {
                    return Err(err(
                        cx,
                        &format!(
                            "tagrec scrutinee has non-TagType type: {}",
                            show_val(cx, other)
                        ),
                    ));
                }
            };
            let case_set: Vec<&Name> = cases.iter().map(|(l, _)| l).collect();
            for l in &ls {
                if !case_set.contains(&l) {
                    return Err(err(cx, &format!("tagrec: missing case `{l}")));
                }
            }
            for l in &case_set {
                if !ls.contains(l) {
                    return Err(err(cx, &format!("tagrec: extra case `{l}")));
                }
            }
            let p_ty = Val::Pi(
                name("_"),
                Rc::new(Val::TagType(ls.clone())),
                Closure::Body(cx.env.clone(), Rc::new(Tm::U)),
            );
            let p_tm = check(cx, p, &p_ty)?;
            let p_val = eval(&cx.env, &p_tm);
            let mut cases_tm: Vec<(Name, Rc<Tm>)> = Vec::with_capacity(ls.len());
            for l in &ls {
                let body = cases
                    .iter()
                    .find_map(|(cl, b)| if cl == l { Some(b) } else { None })
                    .unwrap();
                let case_ty = apply(p_val.clone(), Val::Tag(l.clone()));
                let body_tm = check(cx, body, &case_ty)?;
                cases_tm.push((l.clone(), Rc::new(body_tm)));
            }
            let t_val = eval(&cx.env, &t_tm);
            let res_ty = apply(p_val, t_val);
            Ok((Tm::TagRec(Rc::new(p_tm), cases_tm, Rc::new(t_tm)), res_ty))
        }
        Raw::Let(n, ty, val, body) => {
            let ty_tm = check(cx, ty, &Val::U)?;
            let ty_val = eval(&cx.env, &ty_tm);
            let val_tm = check(cx, val, &ty_val)?;
            let val_v = eval(&cx.env, &val_tm);
            let cx2 = cx.define(n.clone(), val_v, ty_val);
            let (body_tm, body_ty) = infer(&cx2, body)?;
            Ok((
                Tm::Let(n.clone(), Rc::new(ty_tm), Rc::new(val_tm), Rc::new(body_tm)),
                body_ty,
            ))
        }
        Raw::Ann(t, ty) => {
            let ty_tm = check(cx, ty, &Val::U)?;
            let ty_val = eval(&cx.env, &ty_tm);
            let t_tm = check(cx, t, &ty_val)?;
            Ok((t_tm, ty_val))
        }
        Raw::Lam(..) => Err(err(cx, "cannot infer lambda; give a type annotation")),
    }
}

fn elab_telescope_pi(cx: &Cxt, ns: &[Name], dom_val: &Val, body: &Raw) -> Result<Tm, String> {
    if ns.is_empty() {
        return check(cx, body, &Val::U);
    }
    let mut cur_cx = cx.clone();
    for n in ns {
        cur_cx = cur_cx.bind(n.clone(), dom_val.clone());
    }
    let mut body_tm = check(&cur_cx, body, &Val::U)?;
    for n in ns.iter().rev() {
        let lvl_here = cur_cx.level() - 1;
        let dom_tm_here = quote(lvl_here, dom_val);
        body_tm = Tm::Pi(n.clone(), Rc::new(dom_tm_here), Rc::new(body_tm));
        cur_cx.env.0.pop();
        cur_cx.types.pop();
        cur_cx.names.pop();
    }
    Ok(body_tm)
}

fn elab_telescope_sigma(cx: &Cxt, ns: &[Name], dom_val: &Val, body: &Raw) -> Result<Tm, String> {
    if ns.is_empty() {
        return check(cx, body, &Val::U);
    }
    let mut cur_cx = cx.clone();
    for n in ns {
        cur_cx = cur_cx.bind(n.clone(), dom_val.clone());
    }
    let mut body_tm = check(&cur_cx, body, &Val::U)?;
    for n in ns.iter().rev() {
        let lvl_here = cur_cx.level() - 1;
        let dom_tm_here = quote(lvl_here, dom_val);
        body_tm = Tm::Sigma(n.clone(), Rc::new(dom_tm_here), Rc::new(body_tm));
        cur_cx.env.0.pop();
        cur_cx.types.pop();
        cur_cx.names.pop();
    }
    Ok(body_tm)
}

fn nat_step_ty(cx: &Cxt, p_val: &Val) -> Val {
    let p_ty = Val::Pi(
        name("_"),
        Rc::new(Val::Nat),
        Closure::Body(cx.env.clone(), Rc::new(Tm::U)),
    );
    let cx2 = cx.define(name("_P"), p_val.clone(), p_ty);
    let inner_dom = Tm::App(Rc::new(Tm::Var(1)), Rc::new(Tm::Var(0)));
    let inner_cod = Tm::App(Rc::new(Tm::Var(2)), Rc::new(Tm::Suc(Rc::new(Tm::Var(1)))));
    let inner_pi = Tm::Pi(name("_"), Rc::new(inner_dom), Rc::new(inner_cod));
    let outer_pi = Tm::Pi(name("k"), Rc::new(Tm::Nat), Rc::new(inner_pi));
    eval(&cx2.env, &outer_pi)
}
