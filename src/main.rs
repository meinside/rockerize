mod docker;
mod shell;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    shell::process_args(&args);
}
