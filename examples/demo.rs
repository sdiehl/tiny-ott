use std::process;

use tiny_ott::check_str;

const PRELUDE: &str = include_str!("prelude.ott");
const DEMO: &str = include_str!("demo.ott");

fn main() {
    let src = format!("{PRELUDE}\n{DEMO}");
    match check_str(&src) {
        Ok(out) => print!("{out}"),
        Err(e) => {
            eprintln!("error: {e}");
            process::exit(1);
        }
    }
}
