use std::path::Path;

mod cli;
mod index;


fn main() {
    let p = Path::new(".");
    cli::run_cli();
    //hash(&p);
    //watch(&p);
}
