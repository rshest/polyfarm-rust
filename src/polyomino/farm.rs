// ------------------------------------------------------------------------------------------------
// farm.rs
// ------------------------------------------------------------------------------------------------
use std::f64;
use std::f64::consts::{PI};
use std::fs::File;
use std::io::prelude::*;
use std::cmp;
use rand::{Rng, SeedableRng, StdRng};
use time::{PreciseTime};

use polyomino::shape::{Shape};
use polyomino::layout::{Layout, Bundle, COFFS};

const DISPLAY_ENTRIES : usize = 100;
const COLORS : [&'static str; 12] = [
    "8dd3c7", "ffffb3", "bebada", "fb8072", "80b1d3", "fdb462",
    "b3de69", "fccde5", "d9d9d9", "bc80bd", "ccebc5", "ffed6f"
];

const MIN_FLIPS : usize = 2;
const MAX_FLIPS : usize = 4;


pub struct Farm<'a> {
     bundle : &'a Bundle,
     rng: StdRng,
     gen_size: usize,
     max_iter: u32,
     elites: usize,
     cell_side: u32,
     mut_ratio: f64,
     mut_attempts: u32,
     out_file: String
}

struct Score {
    score: f64,
    layout: u32
}

impl<'a> Farm<'a> {
    //  constructor
    pub fn new(bundle: &'a Bundle, out_file: &str, 
        seed: u32, gen_size: u32, max_iter: u32, 
        elites: u32, mut_percentage: u32, mut_attempts: u32, 
        cell_side: u32) -> Farm<'a> 
    {
        let seed = seed as usize;
        let seed: &[_] = &[seed, seed + 1, seed + 2, seed + 3];
        Farm {
            bundle: &bundle,
            rng: SeedableRng::from_seed(seed),
            gen_size: gen_size as usize,
            max_iter: max_iter,
            elites: elites as usize,
            mut_ratio: (mut_percentage as f64)/100.0,
            mut_attempts: mut_attempts,
            cell_side: cell_side,
            out_file: String::from(out_file)
        }
    }
    
    //  finds approximate radius of a circle to lay out the shapes along
    fn estimate_radius(bundle: &Bundle) -> f64 {
        let len = bundle.iter().map(|v| v[0].estimate_len())
            .fold(0.0, |sum, i| sum + i);
        len/(2.0*PI)
    }
        
    //  main grinding procedure    
    pub fn grind(&mut self) {        
        let radius = Farm::estimate_radius(&self.bundle);
        
        let mut gen0 = vec![Layout::new(&self.bundle); self.gen_size]; 
        let mut gen1 = vec![Layout::new(&self.bundle); self.gen_size]; 
        let mut scores = vec![];
        
        let mut start_time = PreciseTime::now();
        
        //  seed the first generation
        for k in 0..self.gen_size {
            let mut layout = &mut gen0[k];
            layout.shuffle(&mut self.rng);
            layout.arrange_circle(radius);
            layout.center();
            scores.push(Score{layout: 0, score: 0.0});
        }
        
        //  iterate on generations
        let mut gen_idx = 0;
        let mut it = 0;
        loop {
            let (prev_gen, cur_gen) = 
                if gen_idx == 0 {(&gen0, &mut gen1)}
                else {(&gen1, &mut gen0)};
            gen_idx = 1 - gen_idx;
            
            for k in 0..self.gen_size {
                scores[k] = Score{layout: k as u32, score: prev_gen[k].score()};
            }
            
            scores.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
            let cur_time = PreciseTime::now();
            println!("Iteration: {}, max score: {}, time: {}ms", 
                it, scores[0].score, start_time.to(cur_time).num_milliseconds());
            start_time = cur_time;
            self.dump_layouts(&scores, prev_gen);
            if it >= self.max_iter { break; }
              
            let mut ii = 0;
            //  transfer the "elite" ones (making sure there is no duplicates)
            for i in 0..self.gen_size {
                let layout = &prev_gen[scores[i].layout as usize];
                let is_dupe = cur_gen.iter().take(ii).any(|cl| cl == layout);
                if !is_dupe {
                    cur_gen[ii] = layout.clone();
                    ii += 1;
                    if ii == self.elites { break; }
                }
            }  
            
            //  apply mutations
            let num_mut = cmp::min(self.gen_size - ii, 
                ((self.gen_size as f64)*self.mut_ratio) as usize);
            for _ in 0..num_mut {
                //  pick the source gene, favoring the ones with higher scores
                let pick_size = self.gen_size;
                let idx = self.rng.gen_range(0, pick_size*pick_size + 1);
                let idx = pick_size - (idx as f64).sqrt() as usize;
                cur_gen[ii] = self.mutate_gene(&prev_gen[scores[idx].layout as usize]);
                cur_gen[ii].center();
                ii += 1;
            }

            //  pad the rest with the fresh ones
            while ii < self.gen_size {
                let mut layout = &mut cur_gen[ii];
                layout.shuffle(&mut self.rng);
                layout.arrange_circle(radius);
                layout.center();
                ii += 1;
            }
            it += 1;
        }
        println!("Done.");
    }
    
    fn mutate_gene<'c>(&mut self, layout: &Layout<'c>) -> Layout<'c> {
        let mut max_score = -f64::MAX;
        let nshapes = layout.pos.len();
        let mut res = layout.clone();
        for _ in 0..self.mut_attempts {
            let num_flips = self.rng.gen_range(MIN_FLIPS, MAX_FLIPS + 1);
            let mut cl = layout.clone();
            for _ in 0..num_flips {
                let mut_type = self.rng.gen_range(0, 3);
                let pidx1 = self.rng.gen_range(0, nshapes);
                let pidx2 = self.rng.gen_range(0, nshapes);
                if mut_type == 0 {
                    //  change variants on two shapes
                    let shape1 = cl.pos[pidx1].shape as usize;
                    let nvar1 = layout.bundle[shape1].len() as u16;
                    cl.pos[pidx1].var = self.rng.gen_range(0, nvar1);
                    
                    let shape2 = cl.pos[pidx2].shape as usize;
                    let nvar2 = layout.bundle[shape2].len() as u16;
                    cl.pos[pidx2].var = self.rng.gen_range(0, nvar2);
                } else if mut_type == 1 {
                    //  randomly offset a section
                    let offs = COFFS[self.rng.gen_range(0, COFFS.len())];
                    for k in pidx1..(pidx2 + 1) {
                        cl.pos[k].x += offs[0];
                        cl.pos[k].y += offs[1];
                    }
                } else {
                    //  swap two shapes
                    let shape = cl.pos[pidx1].shape;
                    cl.pos[pidx1].shape = cl.pos[pidx2].shape;
                    cl.pos[pidx2].shape = shape;
                    
                    let var = cl.pos[pidx1].var;
                    cl.pos[pidx1].var = cl.pos[pidx2].var;
                    cl.pos[pidx2].var = var;
                }
            }
            let score = cl.score();
            if score > max_score {
                max_score = score;
                res = cl;
            }
        }
        res
    }
    
    fn dump_layouts(&self, scores: &Vec<Score>, gen : &Vec<Layout>) {
        let mut file = File::create(&self.out_file).unwrap();
        writeln!(file, "<div>").unwrap();
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
            
            self.dump_svg(&mut file, layout);
        } 
        writeln!(file, "</div>").unwrap();
    }
    
    fn dump_svg(&self, file : &mut File, layout: &Layout) {
        let (lt, rb) = layout.bounds();
        let w = (rb.x - lt.x + 1) as u32;
        let h = (rb.y - lt.y + 1) as u32;

        let cs = self.cell_side as f64;

        //  svg header
        writeln!(file, r###"
        <svg xmlns="http://www.w3.org/2000/svg"
            xmlns:xlink="http://www.w3.org/1999/xlink"
            shape-rendering="crispEdges"
            width="{}" height="{}">
        "###, (w as f64)*cs, (h as f64)*cs).unwrap();

        //  defs
        writeln!(file, r###"
            <defs>
              <pattern id="squares" patternUnits="userSpaceOnUse" x="0" y="0" width="{}" height="{}">
                <g style="fill:none; stroke:#dde; stroke-width:1">
                  <path d="M0,0 l{},0 L{},{} L0,{} Z"/>
                </g>
              </pattern>
            </defs>
            "###, cs, cs, cs, cs, cs, cs).unwrap();

        //  styles
        let styles = r###"
            <style>
              /* <![CDATA[ */
                .core { fill: url(#squares) #fff; }
                .caption { fill: #aae; font-family:Arial; font-size:25px; font-weight:bold;
                  dominant-baseline:central; text-anchor:middle; }
                .shape { stroke:#8888aa; stroke-width:1; opacity:1; }
              /* ]]> */
            </style>"###;

        writeln!(file, "{}", styles).unwrap();

        //  the core (if present)
        match layout.extract_core() {
            Some((shape, pos)) => {
                let x = (pos.x - lt.x) as f64;
                let y = (pos.y - lt.y) as f64;
                let dx = x*cs;
                let dy = y*cs;
                
                //  the core path
                let path = self.gen_shape_path(&shape);
                write!(file, r###"
                <path class="core" transform="translate({},{})" d="{}"></path>"###,
                    dx, dy, path).unwrap();
                
                //  the caption         
                let tx = dx + (shape.width as f64)*cs*0.5;
                let ty = dy + (shape.height as f64)*cs*0.5;
                write!(file, r###"
                <text class="caption" x="{}" y="{}">{}</text>"###,
                    tx, ty, shape.squares.len()).unwrap();              
            },
            None => ()
        }
        
        //  the shapes
        for pos in &layout.pos {
            let shape = layout.shape_by_pos(pos);
            let x = (pos.x - lt.x) as f64;
            let y = (pos.y - lt.y) as f64;
            let dx = x*cs;
            let dy = y*cs;
            
            let path = self.gen_shape_path(shape);
            let color = COLORS[(pos.shape as usize)%COLORS.len()];
            write!(file, r###"
            <path fill="#{}" class="shape" transform="translate({},{})" d="{}"></path>"###, 
                color, dx, dy, path).unwrap();
        }

        writeln!(file, r###"
        </svg>"### ).unwrap();
    }
    
    fn gen_shape_path(&self, shape: &Shape) -> String {
        use std::fmt::Write;
        let cs = self.cell_side as f64;
        let mut res = String::new();
        for sq in &shape.squares {
            let (x, y) = ((sq.x as f64)*cs, (sq.y as f64)*cs);
            let (x1, y1) = (x + cs, y + cs);
            write!(&mut res, "M{},{} L{},{} L{},{} L{},{} Z ", 
                x, y, x1, y, x1, y1, x, y1).unwrap();
        }
        res
    }
}