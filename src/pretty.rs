use pretty::RcDoc;

use crate::syntax::{Name, Tm};

const WIDTH: usize = 100;

const ATOM: u8 = 3;
const APP: u8 = 2;
const ARROW: u8 = 1;
const TOP: u8 = 0;

pub fn pretty_tm(names: &[Name], tm: &Tm) -> String {
    let mut env: Vec<Name> = names.to_vec();
    doc(&mut env, tm, TOP).pretty(WIDTH).to_string()
}

fn lookup(env: &[Name], ix: usize) -> RcDoc<'static> {
    if ix >= env.len() {
        RcDoc::text(format!("?{ix}"))
    } else {
        let i = env.len() - 1 - ix;
        RcDoc::text(env[i].as_ref().to_string())
    }
}

fn paren(level: u8, my: u8, d: RcDoc<'static>) -> RcDoc<'static> {
    if level > my {
        RcDoc::text("(").append(d).append(RcDoc::text(")"))
    } else {
        d
    }
}

fn doc(env: &mut Vec<Name>, tm: &Tm, lvl: u8) -> RcDoc<'static> {
    match tm {
        Tm::Var(ix) => lookup(env, *ix),
        Tm::U => RcDoc::text("Type"),
        Tm::Nat => RcDoc::text("Nat"),
        Tm::Zero => RcDoc::text("zero"),
        Tm::Bool => RcDoc::text("Bool"),
        Tm::BTrue => RcDoc::text("true"),
        Tm::BFalse => RcDoc::text("false"),
        Tm::Unit => RcDoc::text("Unit"),
        Tm::TT => RcDoc::text("tt"),
        Tm::Empty => RcDoc::text("Empty"),
        Tm::Refl => RcDoc::text("refl"),
        Tm::Suc(t) => paren(lvl, APP, RcDoc::text("suc ").append(doc(env, t, ATOM))),
        Tm::App(f, x) => {
            let d = doc(env, f, APP)
                .append(RcDoc::text(" "))
                .append(doc(env, x, ATOM));
            paren(lvl, APP, d)
        }
        Tm::Lam(n, body) => {
            let head = RcDoc::text("\\")
                .append(RcDoc::text(n.as_ref().to_string()))
                .append(RcDoc::text(" => "));
            env.push(n.clone());
            let body_doc = doc(env, body, TOP);
            env.pop();
            paren(lvl, TOP, head.append(body_doc))
        }
        Tm::Pi(n, dom, cod) => {
            let dom_doc = doc(env, dom, APP);
            let head = if n.as_ref() == "_" {
                dom_doc.append(RcDoc::text(" -> "))
            } else {
                RcDoc::text("(")
                    .append(RcDoc::text(n.as_ref().to_string()))
                    .append(RcDoc::text(" : "))
                    .append(doc(env, dom, TOP))
                    .append(RcDoc::text(") -> "))
            };
            env.push(n.clone());
            let cod_doc = doc(env, cod, ARROW);
            env.pop();
            paren(lvl, ARROW, head.append(cod_doc))
        }
        Tm::Sigma(n, dom, cod) => {
            let dom_doc = doc(env, dom, APP);
            let head = if n.as_ref() == "_" {
                dom_doc.append(RcDoc::text(" * "))
            } else {
                RcDoc::text("(")
                    .append(RcDoc::text(n.as_ref().to_string()))
                    .append(RcDoc::text(" : "))
                    .append(doc(env, dom, TOP))
                    .append(RcDoc::text(") * "))
            };
            env.push(n.clone());
            let cod_doc = doc(env, cod, ARROW);
            env.pop();
            paren(lvl, ARROW, head.append(cod_doc))
        }
        Tm::Pair(a, b) => RcDoc::text("(")
            .append(doc(env, a, TOP))
            .append(RcDoc::text(", "))
            .append(doc(env, b, TOP))
            .append(RcDoc::text(")")),
        Tm::Fst(t) => paren(lvl, APP, RcDoc::text("fst ").append(doc(env, t, ATOM))),
        Tm::Snd(t) => paren(lvl, APP, RcDoc::text("snd ").append(doc(env, t, ATOM))),
        Tm::Eq(a, x, y) => paren(
            lvl,
            APP,
            spaced(
                ["Eq", "_", "_", "_"],
                [doc(env, a, ATOM), doc(env, x, ATOM), doc(env, y, ATOM)],
            ),
        ),
        Tm::Coe(a, b, p, t) => paren(
            lvl,
            APP,
            spaced(
                ["coe", "_", "_", "_", "_"],
                [
                    doc(env, a, ATOM),
                    doc(env, b, ATOM),
                    doc(env, p, ATOM),
                    doc(env, t, ATOM),
                ],
            ),
        ),
        Tm::NatRec(p, z, s, n) => paren(
            lvl,
            APP,
            spaced(
                ["natrec", "_", "_", "_", "_"],
                [
                    doc(env, p, ATOM),
                    doc(env, z, ATOM),
                    doc(env, s, ATOM),
                    doc(env, n, ATOM),
                ],
            ),
        ),
        Tm::BoolRec(p, t, f, b) => paren(
            lvl,
            APP,
            spaced(
                ["boolrec", "_", "_", "_", "_"],
                [
                    doc(env, p, ATOM),
                    doc(env, t, ATOM),
                    doc(env, f, ATOM),
                    doc(env, b, ATOM),
                ],
            ),
        ),
        Tm::EmptyRec(p, e) => paren(
            lvl,
            APP,
            spaced(
                ["empty-rec", "_", "_"],
                [doc(env, p, ATOM), doc(env, e, ATOM)],
            ),
        ),
        Tm::Let(n, ty, val, body) => {
            let head = RcDoc::text("let ")
                .append(RcDoc::text(n.as_ref().to_string()))
                .append(RcDoc::text(" : "))
                .append(doc(env, ty, TOP))
                .append(RcDoc::text(" := "))
                .append(doc(env, val, TOP))
                .append(RcDoc::text(" in "));
            env.push(n.clone());
            let body_doc = doc(env, body, TOP);
            env.pop();
            paren(lvl, TOP, head.append(body_doc))
        }
    }
}

fn spaced<const N: usize, const M: usize>(
    head_words: [&'static str; N],
    args: [RcDoc<'static>; M],
) -> RcDoc<'static> {
    let head = head_words[0];
    let mut d = RcDoc::text(head);
    for a in args {
        d = d.append(RcDoc::text(" ")).append(a);
    }
    d
}
