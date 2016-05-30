extern crate getopts;
extern crate num;
extern crate rand;

mod polyomino;

use std::env;
use std::io::prelude::*;
use std::fs::File;
use getopts::Options;

use polyomino::layout::{parse_bundle};
use polyomino::farm::{Farm};

const DEFAULT_SHAPES_FILE: &'static str = "data/pentomino.txt";

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} SHAPES_FILE [options]", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help text");
    opts.optflag("m", "no-mirror", "don't mirror the shapes");
    opts.optflag("r", "no-rotation", "don't rotate the shapes");
    opts.optopt("o", "output", "output HTML file path", "FILE");
    opts.optopt("s", "seed", "random seed", "NUMBER");
    opts.optopt("g", "gen-size", "generation size", "NUMBER");
    opts.optopt("n", "max-iter", "maximum iteration", "NUMBER");
    opts.optopt("c", "cell-side", "SVG cell side, pixels", "NUMBER");
    
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

    let mut f = File::open(shapes_file).unwrap();
    let mut contents = String::new();
    f.read_to_string(&mut contents).unwrap();
    
    let mirrored = !matches.opt_present("m");
    let rotated = !matches.opt_present("r");
        
    let out_file = matches.opt_str("o")
        .unwrap_or_else(|| String::from("output.html"));
    
    let get_num = |name, default| {
        matches.opt_str(name)
        .unwrap_or_else(|| String::from(default))
        .parse::<u32>().unwrap()
    };

    let seed = get_num("s", "42");
    let gen_size = get_num("g", "1000");
    let max_iter = get_num("n", "1");
    let cell_side = get_num("c", "10");    
    
    println!("gen_size: {}, seed: {}, max_iter: {}, cell_side: {}, output file: {}", 
        gen_size, seed, max_iter, cell_side, out_file);   
    
    let bundle = parse_bundle(&contents, mirrored, rotated);
    let mut farm = Farm::new(&bundle, &out_file, seed, gen_size, max_iter, cell_side);
    farm.grind();
}
