use crate::elab::{check, infer, Cxt};
use crate::eval::{eval, quote};
use crate::parser::parse_str;
use crate::pretty::pretty_tm;
use crate::syntax::Decl;
use crate::value::Val;
use std::fmt::Write;

pub fn check_str(src: &str) -> Result<String, String> {
    let decls = parse_str(src);
    let mut cx = Cxt::default();
    let mut out = String::new();
    for d in decls {
        match d {
            Decl::Def(n, ty_raw, body_raw) => {
                let ty_tm = check(&cx, &ty_raw, &Val::U)?;
                let ty_val = eval(&cx.env, &ty_tm);
                let body_tm = check(&cx, &body_raw, &ty_val)?;
                let body_val = eval(&cx.env, &body_tm);
                writeln!(
                    out,
                    "def {n}\n  : {}\n  := {}",
                    pretty_tm(&cx.names, &ty_tm),
                    pretty_tm(&cx.names, &body_tm)
                )
                .unwrap();
                cx = cx.define(n, body_val, ty_val);
            }
            Decl::Eval(raw) => {
                let (tm, ty) = infer(&cx, &raw)?;
                let v = eval(&cx.env, &tm);
                let nf_tm = quote(cx.level(), &v);
                let ty_tm = quote(cx.level(), &ty);
                writeln!(
                    out,
                    "eval\n  = {}\n  : {}",
                    pretty_tm(&cx.names, &nf_tm),
                    pretty_tm(&cx.names, &ty_tm)
                )
                .unwrap();
            }
            Decl::Check(raw_tm, raw_ty) => {
                let ty_tm = check(&cx, &raw_ty, &Val::U)?;
                let ty_val = eval(&cx.env, &ty_tm);
                let tm = check(&cx, &raw_tm, &ty_val)?;
                writeln!(
                    out,
                    "check ok\n  : {}\n  ~ {}",
                    pretty_tm(&cx.names, &ty_tm),
                    pretty_tm(&cx.names, &tm)
                )
                .unwrap();
            }
        }
    }
    Ok(out)
}
