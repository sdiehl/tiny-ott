use crate::syntax::{Name, Tm};

pub fn pretty_tm(names: &[Name], tm: &Tm) -> String {
    let mut out = String::new();
    let mut env: Vec<Name> = names.to_vec();
    go(&mut out, &mut env, tm, 0);
    out
}

fn lookup_name(env: &[Name], ix: usize) -> String {
    if ix >= env.len() {
        format!("?{ix}")
    } else {
        let i = env.len() - 1 - ix;
        env[i].as_ref().to_string()
    }
}

fn paren(out: &mut String, level: u8, my: u8, f: impl FnOnce(&mut String)) {
    if level > my {
        out.push('(');
        f(out);
        out.push(')');
    } else {
        f(out);
    }
}

fn go(out: &mut String, env: &mut Vec<Name>, tm: &Tm, lvl: u8) {
    match tm {
        Tm::Var(ix) => out.push_str(&lookup_name(env, *ix)),
        Tm::U => out.push_str("Type"),
        Tm::Nat => out.push_str("Nat"),
        Tm::Zero => out.push_str("zero"),
        Tm::Bool => out.push_str("Bool"),
        Tm::BTrue => out.push_str("true"),
        Tm::BFalse => out.push_str("false"),
        Tm::Unit => out.push_str("Unit"),
        Tm::TT => out.push_str("tt"),
        Tm::Empty => out.push_str("Empty"),
        Tm::Refl => out.push_str("refl"),
        Tm::Suc(t) => paren(out, lvl, 2, |o| {
            o.push_str("suc ");
            go(o, env, t, 3);
        }),
        Tm::App(f, x) => paren(out, lvl, 2, |o| {
            go(o, env, f, 2);
            o.push(' ');
            go(o, env, x, 3);
        }),
        Tm::Lam(n, body) => paren(out, lvl, 0, |o| {
            o.push('\\');
            o.push_str(n);
            o.push_str(" => ");
            env.push(n.clone());
            go(o, env, body, 0);
            env.pop();
        }),
        Tm::Pi(n, dom, cod) => paren(out, lvl, 1, |o| {
            if n.as_ref() == "_" {
                go(o, env, dom, 2);
                o.push_str(" -> ");
            } else {
                o.push('(');
                o.push_str(n);
                o.push_str(" : ");
                go(o, env, dom, 0);
                o.push_str(") -> ");
            }
            env.push(n.clone());
            go(o, env, cod, 1);
            env.pop();
        }),
        Tm::Sigma(n, dom, cod) => paren(out, lvl, 1, |o| {
            if n.as_ref() == "_" {
                go(o, env, dom, 2);
                o.push_str(" * ");
            } else {
                o.push('(');
                o.push_str(n);
                o.push_str(" : ");
                go(o, env, dom, 0);
                o.push_str(") * ");
            }
            env.push(n.clone());
            go(o, env, cod, 1);
            env.pop();
        }),
        Tm::Pair(a, b) => {
            out.push('(');
            go(out, env, a, 0);
            out.push_str(", ");
            go(out, env, b, 0);
            out.push(')');
        }
        Tm::Fst(t) => paren(out, lvl, 2, |o| {
            o.push_str("fst ");
            go(o, env, t, 3);
        }),
        Tm::Snd(t) => paren(out, lvl, 2, |o| {
            o.push_str("snd ");
            go(o, env, t, 3);
        }),
        Tm::Eq(a, x, y) => paren(out, lvl, 2, |o| {
            o.push_str("Eq ");
            go(o, env, a, 3);
            o.push(' ');
            go(o, env, x, 3);
            o.push(' ');
            go(o, env, y, 3);
        }),
        Tm::Coe(a, b, p, t) => paren(out, lvl, 2, |o| {
            o.push_str("coe ");
            go(o, env, a, 3);
            o.push(' ');
            go(o, env, b, 3);
            o.push(' ');
            go(o, env, p, 3);
            o.push(' ');
            go(o, env, t, 3);
        }),
        Tm::NatRec(p, z, s, n) => paren(out, lvl, 2, |o| {
            o.push_str("natrec ");
            go(o, env, p, 3);
            o.push(' ');
            go(o, env, z, 3);
            o.push(' ');
            go(o, env, s, 3);
            o.push(' ');
            go(o, env, n, 3);
        }),
        Tm::BoolRec(p, t, f, b) => paren(out, lvl, 2, |o| {
            o.push_str("boolrec ");
            go(o, env, p, 3);
            o.push(' ');
            go(o, env, t, 3);
            o.push(' ');
            go(o, env, f, 3);
            o.push(' ');
            go(o, env, b, 3);
        }),
        Tm::EmptyRec(p, e) => paren(out, lvl, 2, |o| {
            o.push_str("empty-rec ");
            go(o, env, p, 3);
            o.push(' ');
            go(o, env, e, 3);
        }),
        Tm::Let(n, ty, val, body) => paren(out, lvl, 0, |o| {
            o.push_str("let ");
            o.push_str(n);
            o.push_str(" : ");
            go(o, env, ty, 0);
            o.push_str(" := ");
            go(o, env, val, 0);
            o.push_str(" in ");
            env.push(n.clone());
            go(o, env, body, 0);
            env.pop();
        }),
    }
}
