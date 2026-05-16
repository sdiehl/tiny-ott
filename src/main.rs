use std::env;
use std::fs;
use std::process::ExitCode;

use rustyline::error::ReadlineError;
use rustyline::{Config, DefaultEditor};
use tiny_ott::diagnostics::render;
use tiny_ott::driver::check_str_pretty;
use tiny_ott::elab::{infer, Cxt};
use tiny_ott::errors::{TinyOttError, TypeError};
use tiny_ott::eval::{eval, quote};
use tiny_ott::parse::Parser;
use tiny_ott::pretty::pretty_tm;

const HELP: &str = "\
tiny-ott: a small observational type theory checker

USAGE:
    tiny-ott [FILE]        check a .ott file and print results
    tiny-ott repl          start an interactive REPL
    tiny-ott --help        show this help

REPL commands:
    :t <expr>              infer the type of <expr>
    :l <file>              load definitions from <file>
    :q                     quit
    :?                     this help
    <decl>                 def/eval/check declaration
    <expr>                 evaluate to normal form
";

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    match args.as_slice() {
        [] => repl(),
        [a] if a == "--help" || a == "-h" => {
            println!("{HELP}");
            ExitCode::SUCCESS
        }
        [a] if a == "repl" => repl(),
        [a] if a == "check" => {
            eprintln!("missing FILE");
            ExitCode::FAILURE
        }
        [cmd, file] if cmd == "check" => check_file(file),
        [file] => check_file(file),
        _ => {
            eprintln!("{HELP}");
            ExitCode::FAILURE
        }
    }
}

fn check_file(path: &str) -> ExitCode {
    let Ok(src) = fs::read_to_string(path) else {
        eprintln!("cannot read {path}");
        return ExitCode::FAILURE;
    };
    match check_str_pretty(&src) {
        Ok(out) => {
            print!("{out}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprint!("{}", render(&e, path, &src));
            ExitCode::FAILURE
        }
    }
}

fn repl() -> ExitCode {
    println!("tiny-ott REPL. Type :? for help, :q to quit.");
    let config = Config::builder().auto_add_history(true).build();
    let Ok(mut rl) = DefaultEditor::with_config(config) else {
        eprintln!("failed to start readline");
        return ExitCode::FAILURE;
    };
    let parser = Parser::new();
    let mut cx = Cxt::default();
    loop {
        match rl.readline("> ") {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                if let Some(rest) = line.strip_prefix(':') {
                    if !handle_cmd(&mut cx, &parser, rest) {
                        return ExitCode::SUCCESS;
                    }
                } else {
                    eval_input(&mut cx, &parser, line);
                }
            }
            Err(ReadlineError::Interrupted | ReadlineError::Eof) => return ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("readline error: {e}");
                return ExitCode::FAILURE;
            }
        }
    }
}

fn handle_cmd(cx: &mut Cxt, parser: &Parser, rest: &str) -> bool {
    let (cmd, arg) = rest.split_once(char::is_whitespace).unwrap_or((rest, ""));
    let arg = arg.trim();
    match cmd {
        "q" | "quit" => false,
        "?" | "h" | "help" => {
            println!("{HELP}");
            true
        }
        "t" | "type" => {
            type_of(cx, parser, arg);
            true
        }
        "l" | "load" => {
            load(cx, arg);
            true
        }
        _ => {
            println!("unknown command :{cmd} (try :?)");
            true
        }
    }
}

fn eval_input(cx: &mut Cxt, parser: &Parser, input: &str) {
    let trimmed = input.trim_start();
    if trimmed.starts_with("def ") || trimmed.starts_with("eval ") || trimmed.starts_with("check ")
    {
        let mut src = input.to_string();
        if !src.ends_with('\n') {
            src.push('\n');
        }
        match check_str_pretty(&src) {
            Ok(out) => {
                print!("{out}");
                if let Ok(mut new_cx) = rebuild_cxt(&src) {
                    std::mem::swap(cx, &mut new_cx);
                }
            }
            Err(e) => eprint!("{}", render(&e, "<repl>", &src)),
        }
        return;
    }
    let raw = match parser.parse_term(input) {
        Ok(r) => r,
        Err(e) => {
            eprint!("{}", render(&e, "<repl>", input));
            return;
        }
    };
    match infer(cx, &raw).map_err(TypeError::new) {
        Ok((tm, ty)) => {
            let v = eval(&cx.env, &tm);
            let nf = quote(cx.level(), &v);
            let ty_tm = quote(cx.level(), &ty);
            println!(
                "{}\n  : {}",
                pretty_tm(&cx.names, &nf),
                pretty_tm(&cx.names, &ty_tm)
            );
        }
        Err(e) => {
            let err: TinyOttError = e.into();
            eprint!("{}", render(&err, "<repl>", input));
        }
    }
}

fn type_of(cx: &Cxt, parser: &Parser, input: &str) {
    if input.is_empty() {
        println!("usage: :t <expr>");
        return;
    }
    let raw = match parser.parse_term(input) {
        Ok(r) => r,
        Err(e) => {
            eprint!("{}", render(&e, "<repl>", input));
            return;
        }
    };
    match infer(cx, &raw).map_err(TypeError::new) {
        Ok((_, ty)) => {
            let ty_tm = quote(cx.level(), &ty);
            println!("{} : {}", input, pretty_tm(&cx.names, &ty_tm));
        }
        Err(e) => {
            let err: TinyOttError = e.into();
            eprint!("{}", render(&err, "<repl>", input));
        }
    }
}

fn load(cx: &mut Cxt, path: &str) {
    if path.is_empty() {
        println!("usage: :l <file>");
        return;
    }
    let Ok(src) = fs::read_to_string(path) else {
        eprintln!("cannot read {path}");
        return;
    };
    match check_str_pretty(&src) {
        Ok(out) => {
            print!("{out}");
            if let Ok(mut new_cx) = rebuild_cxt(&src) {
                std::mem::swap(cx, &mut new_cx);
            }
        }
        Err(e) => eprint!("{}", render(&e, path, &src)),
    }
}

fn rebuild_cxt(src: &str) -> Result<Cxt, TinyOttError> {
    use tiny_ott::elab::check;
    use tiny_ott::syntax::Decl;
    use tiny_ott::value::Val;
    let parser = Parser::new();
    let decls = parser.parse_module(src)?;
    let mut cx = Cxt::default();
    for d in decls {
        if let Decl::Def(n, ty_raw, body_raw) = d {
            let ty_tm = check(&cx, &ty_raw, &Val::U).map_err(TypeError::new)?;
            let ty_val = eval(&cx.env, &ty_tm);
            let body_tm = check(&cx, &body_raw, &ty_val).map_err(TypeError::new)?;
            let body_val = eval(&cx.env, &body_tm);
            cx = cx.define(n, body_val, ty_val);
        }
    }
    Ok(cx)
}
