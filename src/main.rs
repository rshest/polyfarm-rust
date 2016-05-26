extern crate getopts;
use getopts::Options;

use std::env;
use std::io::{BufReader};
use std::io::prelude::*;
use std::fs::File;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} FILE [options]", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help text");

    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };

    let shapes_file = if !matches.free.is_empty() {
        matches.free[0].clone()
    } else {
        print_usage(&program, opts);
        return;
    };

    let f = File::open(shapes_file).unwrap();
    for line in BufReader::new(f).lines() {
        println!("{}", line.unwrap());
    }

    println!("Hello, world!");
}
