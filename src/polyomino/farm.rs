use rand::{Rng, SeedableRng, StdRng};
use std::f64::consts::{PI};

use polyomino::layout::{Layout, Bundle};

pub struct Farm<'a> {
     bundle : &'a Bundle,
     rng: StdRng
}

impl<'a> Farm<'a> {
    pub fn new(bundle: &'a Bundle, seed: u64, out_file: &str) -> Farm<'a> {
        let seed = seed as usize;
        let seed: &[_] = &[seed, seed + 1, seed + 2, seed + 3];
        let mut rng: StdRng = SeedableRng::from_seed(seed);
        
        let mut layout = Layout::new(&bundle); 
        layout.shuffle(&mut rng);
        
        let radius = Farm::estimate_radius(&bundle);
        layout.arrange_circle(radius);
    
        for b in layout.pos {
            println!("{}, pos: {}, {}, var: {}", 
                b.shape, b.x, b.y, b.var);
        }    
        
        Farm {
            bundle: &bundle,
            rng: rng
        }
    }
    
    
    pub fn grind(&mut self) {
    
    }
    
    fn estimate_radius(bundle: &Bundle) -> f64 {
        let len = bundle.iter().map(|v| v[0].estimate_len())
            .fold(0.0, |sum, i| sum + i);
        len/(2.0*PI)
    }
}