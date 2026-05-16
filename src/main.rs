use std::fs;
use std::mem;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser as ClapParser, Subcommand};
use rustyline::error::ReadlineError;
use rustyline::{Config, DefaultEditor};
use tiny_ott::diagnostics::render;
use tiny_ott::driver::check_str_pretty;
use tiny_ott::elab::{check, infer, Cxt};
use tiny_ott::errors::{TinyOttError, TypeError};
use tiny_ott::eval::{eval, quote, quote_typed};
use tiny_ott::parse::Parser;
use tiny_ott::pretty::pretty_tm;
use tiny_ott::syntax::{Decl, ReplInput};
use tiny_ott::value::Val;

const REPL_HELP: &str = "\
:t <expr>    infer the type of <expr>
:l <file>    load definitions from <file>
:?           this help
:q           quit
<decl>       def/eval/check declaration
<expr>       evaluate to normal form
";

#[derive(ClapParser)]
#[command(name = "tiny-ott", about = "a small observational type theory checker")]
struct Cli {
    #[command(subcommand)]
    cmd: Option<Cmd>,
}

#[derive(Subcommand)]
enum Cmd {
    /// Start an interactive REPL
    Repl,
    /// Type-check a .ott file
    Check {
        /// Path to a .ott source file
        file: PathBuf,
    },
}

fn main() -> ExitCode {
    match Cli::parse().cmd {
        None | Some(Cmd::Repl) => repl(),
        Some(Cmd::Check { file }) => check_file(&file),
    }
}

fn check_file(path: &Path) -> ExitCode {
    let Ok(src) = fs::read_to_string(path) else {
        eprintln!("cannot read {}", path.display());
        return ExitCode::FAILURE;
    };
    match check_str_pretty(&src) {
        Ok(out) => {
            print!("{out}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprint!("{}", render(&e, &path.display().to_string(), &src));
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
            print!("{REPL_HELP}");
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
    match parser.parse_repl(input) {
        Ok(ReplInput::Decl(d)) => run_decl(cx, &d, input),
        Ok(ReplInput::Term(raw)) => match infer(cx, &raw).map_err(TypeError::new) {
            Ok((tm, ty)) => {
                let v = eval(&cx.env, &tm);
                let nf = quote_typed(cx.level(), &ty, &v);
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
        },
        Err(e) => eprint!("{}", render(&e, "<repl>", input)),
    }
}

fn run_decl(cx: &mut Cxt, decl: &Decl, input: &str) {
    let mut src = input.to_string();
    if !src.ends_with('\n') {
        src.push('\n');
    }
    match check_str_pretty(&src) {
        Ok(out) => {
            print!("{out}");
            if matches!(decl, Decl::Def(..)) {
                if let Ok(mut new_cx) = rebuild_cxt(&src) {
                    mem::swap(cx, &mut new_cx);
                }
            }
        }
        Err(e) => eprint!("{}", render(&e, "<repl>", &src)),
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
                mem::swap(cx, &mut new_cx);
            }
        }
        Err(e) => eprint!("{}", render(&e, path, &src)),
    }
}

fn rebuild_cxt(src: &str) -> Result<Cxt, TinyOttError> {
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
