use rand::{Rng, SeedableRng, StdRng};
use std::f64::consts::{PI};
use std::fs::File;
use std::io::prelude::*;
use std::cmp;

use polyomino::layout::{Layout, Bundle, Position};

const DISPLAY_ENTRIES : usize = 100;

pub struct Farm<'a> {
     bundle : &'a Bundle,
     rng: StdRng,
     gen_size: usize,
     max_iter: u32,
     cell_side: u32,
     out_file: String
}

struct Score {
    score: f64,
    layout: u32
}

impl<'a> Farm<'a> {
    pub fn new(bundle: &'a Bundle, out_file: &str, 
        seed: u32, gen_size: u32, max_iter: u32, cell_side: u32) -> Farm<'a> {
        let seed = seed as usize;
        let seed: &[_] = &[seed, seed + 1, seed + 2, seed + 3];
        Farm {
            bundle: &bundle,
            rng: SeedableRng::from_seed(seed),
            gen_size: gen_size as usize,
            max_iter: max_iter,
            cell_side: cell_side,
            out_file: String::from(out_file)
        }
    }
    
    pub fn grind(&mut self) {        
        let radius = Farm::estimate_radius(&self.bundle);
        
        let mut gen = [
            vec![Layout::new(&self.bundle);self.gen_size], 
            vec![Layout::new(&self.bundle);self.gen_size],];
            
        let mut scores = vec![];
        
        let cur_gen = 0;
        for k in 0..self.gen_size {
            let mut layout = &mut gen[cur_gen][k];
            layout.shuffle(&mut self.rng);
            layout.arrange_circle(radius);
            layout.center();
            scores.push(Score{
                layout: k as u32,
                score: layout.score()
            });
        }
        
        let mut it = 0;
        loop {
            scores.sort_by(|a, b| a.score.partial_cmp(&b.score).unwrap());
            
            self.dump_layouts(&scores, &gen[cur_gen]);
            
            it += 1;
            if it >= self.max_iter { break; }
        }
    }
    
    fn dump_layouts(&self, scores: &Vec<Score>, gen : &Vec<Layout>) {
        let mut buffer = File::create(&self.out_file).unwrap();
        writeln!(buffer, "<div>");
        
        let ndisp = cmp::min(DISPLAY_ENTRIES, self.gen_size);
        let mut k = 0;
        let mut cur_pos = 0;
        while cur_pos < ndisp && k < self.gen_size {
            let layout = &gen[scores[k].layout as usize];
            let is_dupe = scores.iter().take(k).any(|s| {
                layout == &gen[s.layout as usize]
            });
            k += 1;
            if is_dupe { continue; }
            cur_pos += 1;
        }
        
        writeln!(buffer, "</div>");
    }
    
    fn estimate_radius(bundle: &Bundle) -> f64 {
        let len = bundle.iter().map(|v| v[0].estimate_len())
            .fold(0.0, |sum, i| sum + i);
        len/(2.0*PI)
    }
}