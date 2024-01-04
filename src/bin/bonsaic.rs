// SPDX-License-Identifier: Unlicense
use std::path::Path;
use bonsai::driver;
fn main() {

    let args = std::env::args().collect::<Vec<_>>();
    if args.len() < 2 {
        eprintln!("please specify input file");
        std::process::exit(1);
    }
    let source = Path::new(&args[1]);
    match driver::compile(source) {
        Ok(v) => println!("successfully compiled to {}", v.to_str().unwrap_or("<unknown>")),
        Err(v) => eprintln!("failed to compile:\n{}", v)
    }
}
