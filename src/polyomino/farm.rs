use std::f64::consts::{PI};
use std::fs::File;
use std::io::prelude::*;
use std::cmp;
use rand::{Rng, SeedableRng, StdRng};


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
    
    fn estimate_radius(bundle: &Bundle) -> f64 {
        let len = bundle.iter().map(|v| v[0].estimate_len())
            .fold(0.0, |sum, i| sum + i);
        len/(2.0*PI)
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
        let mut file = File::create(&self.out_file).unwrap();
        writeln!(file, "<div>");
        
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
        writeln!(file, "</div>");
    }
    
    fn dump_svg(&self, file : &mut File, layout: &Layout) {
        let (lt, rb) = layout.bounds();
        let w = (rb.x - lt.x + 1) as u32;
        let h = (rb.y - lt.y + 1) as u32;

        let cs = self.cell_side;

        //  svg header
        writeln!(file, r###"
        <svg xmlns="http://www.w3.org/2000/svg"
            xmlns:xlink="http://www.w3.org/1999/xlink"
            shape-rendering="crispEdges"
            width="{}" height="{}">
        "###, w*cs, h*cs);

        //  defs
        writeln!(file, r###"
            <defs>
              <pattern id="squares" patternUnits="userSpaceOnUse" x="0" y="0" width="{}" height="{}">
                <g style="fill:none; stroke:#dde; stroke-width:1">
                  <path d="M0,0 l{},0 L{},{} L0,{} Z"/>
                </g>
              </pattern>
            </defs>
            "###, cs, cs, cs, cs, cs, cs);

        //  styles
        let styles = r###"
            <style>
              /* <![CDATA[ */
                .core { fill: url(#squares) #fff; }
                .caption { fill: #aae; font-family:Arial; font-size:25px; font-weight:bold;
                  dominant-baseline:central; text-anchor:middle; }
                .shape { stroke:#224a22; stroke-width:1; opacity:1; }
              /* ]]> */
            </style>"###;

        writeln!(file, "{}", styles);

        for pos in &layout.pos {
            let shape = layout.shape_by_pos(pos);
            let x = (pos.x - lt.x) as f64;
            let y = (pos.y - lt.y) as f64;
            let dx = x*(cs as f64);
            let dy = y*(cs as f64);


        }

        match layout.extract_core() {
            Some((pos, shape)) => (),
            None => ()
        }

        writeln!(file, r###"
        </svg>"### );
    }
}