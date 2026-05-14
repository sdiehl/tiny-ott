use tiny_ott::check_str;

const SRC: &str = include_str!("demo.ott");

fn main() {
    match check_str(SRC) {
        Ok(out) => print!("{out}"),
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    }
}
