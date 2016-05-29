use rand::{Rng, SeedableRng, StdRng};

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
    
        for b in layout.pos {
            println!("{}", b.shape)
        }    
        
        Farm {
            bundle: &bundle,
            rng: rng
        }
    }
    
    pub fn grind(&mut self) {
    
    }
}