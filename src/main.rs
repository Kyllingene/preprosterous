use std::collections::HashMap;
use std::process::exit;

use prep::prelude::*;

fn main() {
    let mut vars = HashMap::new();

    let mut args = std::env::args();
    if let Some(file) = args.nth(1) {
        let data = match process_file(file, &mut vars) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Error: {e}");
                exit(1);
            }
        };

        println!("{}", data.output());
    } else {
        eprintln!("No file supplied");
        exit(1);
    }
}
