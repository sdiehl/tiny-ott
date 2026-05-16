use pretty::RcDoc;

use crate::syntax::{Name, Tm, TAG_FALSE, TAG_TRUE, TAG_TT};

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
        Tm::TagType(ls) => tag_type_doc(ls),
        Tm::Tag(l) => tag_doc(l),
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
        Tm::Subst(a, p, x, y, pr, px) => paren(
            lvl,
            APP,
            spaced(
                ["subst", "_", "_", "_", "_", "_", "_"],
                [
                    doc(env, a, ATOM),
                    doc(env, p, ATOM),
                    doc(env, x, ATOM),
                    doc(env, y, ATOM),
                    doc(env, pr, ATOM),
                    doc(env, px, ATOM),
                ],
            ),
        ),
        Tm::Coh(a, b, p, t) => paren(
            lvl,
            APP,
            spaced(
                ["coh", "_", "_", "_", "_"],
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
        Tm::TagRec(p, cases, t) => tagrec_doc(env, p, cases, t, lvl),
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
        Tm::Irrel => RcDoc::text("_"),
    }
}

fn tag_type_doc(ls: &[Name]) -> RcDoc<'static> {
    if ls.is_empty() {
        return RcDoc::text("Empty");
    }
    if ls.len() == 1 && ls[0].as_ref() == TAG_TT {
        return RcDoc::text("Unit");
    }
    if ls.len() == 2 && ls[0].as_ref() == TAG_TRUE && ls[1].as_ref() == TAG_FALSE {
        return RcDoc::text("Bool");
    }
    let mut d = RcDoc::text("{|");
    for (i, l) in ls.iter().enumerate() {
        if i > 0 {
            d = d.append(RcDoc::text(","));
        }
        d = d
            .append(RcDoc::text(" `"))
            .append(RcDoc::text(l.as_ref().to_string()));
    }
    d.append(RcDoc::text(" |}"))
}

fn tag_doc(l: &Name) -> RcDoc<'static> {
    let s = l.as_ref();
    if s == TAG_TT || s == TAG_TRUE || s == TAG_FALSE {
        RcDoc::text(s.to_string())
    } else {
        RcDoc::text("`").append(RcDoc::text(s.to_string()))
    }
}

fn tagrec_doc(
    env: &mut Vec<Name>,
    p: &Tm,
    cases: &[(Name, std::rc::Rc<Tm>)],
    t: &Tm,
    lvl: u8,
) -> RcDoc<'static> {
    if cases.is_empty() {
        let ty_doc = if let Tm::Lam(n, body) = p {
            env.push(n.clone());
            let d = doc(env, body, ATOM);
            env.pop();
            d
        } else {
            doc(env, p, ATOM)
        };
        return paren(
            lvl,
            APP,
            RcDoc::text("empty-rec ")
                .append(ty_doc)
                .append(RcDoc::text(" "))
                .append(doc(env, t, ATOM)),
        );
    }
    if cases.len() == 2 && cases[0].0.as_ref() == TAG_TRUE && cases[1].0.as_ref() == TAG_FALSE {
        return paren(
            lvl,
            APP,
            spaced(
                ["boolrec", "_", "_", "_", "_"],
                [
                    doc(env, p, ATOM),
                    doc(env, &cases[0].1, ATOM),
                    doc(env, &cases[1].1, ATOM),
                    doc(env, t, ATOM),
                ],
            ),
        );
    }
    let mut body = RcDoc::text("{ ");
    for (i, (l, b)) in cases.iter().enumerate() {
        if i > 0 {
            body = body.append(RcDoc::text("; "));
        }
        body = body
            .append(RcDoc::text("`"))
            .append(RcDoc::text(l.as_ref().to_string()))
            .append(RcDoc::text(" => "))
            .append(doc(env, b, TOP));
    }
    body = body.append(RcDoc::text(" }"));
    paren(
        lvl,
        APP,
        RcDoc::text("tagrec ")
            .append(doc(env, p, ATOM))
            .append(RcDoc::text(" "))
            .append(body)
            .append(RcDoc::text(" "))
            .append(doc(env, t, ATOM)),
    )
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
