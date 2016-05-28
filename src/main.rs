extern crate getopts;
extern crate num;

mod polyomino;

use std::env;
use std::io::{BufReader};
use std::io::prelude::*;
use std::fs::File;
use getopts::Options;

const DEFAULT_SHAPES_FILE: &'static str = "data/pentomino.txt";

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} SHAPES_FILE [options]", program);
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

    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }

    let shapes_file = if !matches.free.is_empty() {
        matches.free[0].clone()
    } else {
        println!("Shapes file not specified, using default: {}", DEFAULT_SHAPES_FILE);
        String::from(DEFAULT_SHAPES_FILE)
    };

    let f = File::open(shapes_file).unwrap();
    for line in BufReader::new(f).lines() {
        println!("{}", line.unwrap());
    }


}
