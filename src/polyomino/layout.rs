use std::f64;
use super::vec2::Vec2i;
use super::shape::{Shape, OFFS};
use rand::{Rng};

const COFFS: [[i32; 2]; 8] = [[1, 0], [0, 1], [-1, 0], [0, -1], [1, -1], [1, 1], [-1, 1], [-1, -1]];

#[derive(PartialEq, Debug)]
pub enum Overlap {
    Overlap, // have a common square
    Border, // have a common edge
    Disjoint, // neither a common square nor edge
}

// A bundle(set) of polyomino shapes
pub type Bundle = Vec<Vec<Shape>>;


pub struct Position {
    pub x : i32,
    pub y : i32,
    pub shape : u16,    //  shape index
    pub var : u16       //  shape variant index
}

pub struct Layout<'a> {
    bundle : &'a Bundle,
    pub pos : Vec<Position>,
}

pub fn parse_bundle(input: &str, mirrored: bool, rotated: bool) -> Bundle {
    input.split("\n\n")
    .map(|s| Shape::parse(s).variants(mirrored, rotated))
    .collect()
}

impl<'a> Layout<'a> {
    //  returns manhattan distance between two shapes' squares
    // -1 if they overlap, 0 if border
    fn distance(shape1 : &Shape, shape2 : &Shape, 
        pos1: &Vec2i, pos2: &Vec2i) -> i32 
    {
        let overlap = Layout::overlap_status(shape1, shape2, pos1, pos2);
        if overlap == Overlap::Border { return 0; }
        if overlap == Overlap::Overlap { return -1; }
           
        let dx = pos1.x - pos2.x;
        let dy = pos1.y - pos2.y;
        
        let min_dist = shape1.squares.iter().flat_map(|sq1| {
            let x = sq1.x + dx;
            let y = sq1.y + dy;
            shape2.squares.iter().map(move |sq2| {
                (x - sq2.x).abs() + (y - sq2.y).abs()
            })
        }).min().unwrap();
        min_dist - 1
    }
    
    //  returns overlap status between two shapes
    fn overlap_status(shape1 : &Shape, shape2 : &Shape, 
        pos1: &Vec2i, pos2: &Vec2i) -> Overlap 
    {
        if pos1.x > pos2.x + shape2.width  ||
           pos2.x > pos1.x + shape1.width  ||
           pos1.y > pos2.y + shape2.height ||
           pos2.y > pos1.y + shape1.height {
            return Overlap::Disjoint;
        }
        let dx = pos1.x - pos2.x;
        let dy = pos1.y - pos2.y;
        
        //  test for overlapping
        for sq in &shape1.squares {
            let x = sq.x + dx;
            let y = sq.y + dy;
            if shape2.is_set(x, y) { return Overlap::Overlap };
        }
        
        //  no overlapping, test for bordering
        for sq in &shape1.squares {
            for offs in OFFS.iter() {
                let x = sq.x + dx + offs[0];
                let y = sq.y + dy + offs[1];
                if shape2.is_set(x, y) { return Overlap::Border };
            }
        }
        Overlap::Disjoint
    }
    
    //  finds a "best fit" variation/position of a shape 
    //  (according to a fit function, minimizing its output), 
    //  so that it is bordered with the anchor shape
    fn best_fit<F>(anchor_shape: &Shape, anchor_pos: &Vec2i, 
        variants: &Vec<Shape>, fit: F) -> (u16, Vec2i) 
        where F : Fn(&Vec2i, &Shape) -> f64
    {
        let mut min_d = f64::MAX;
        let mut res = (0, Vec2i{x: 0, y: 0});
        for (i, ref shape) in variants.iter().enumerate() {
            for bpos in anchor_shape.boundary.iter() {
                for cpos in shape.squares.iter() {
                    let p = Vec2i{
                        x: anchor_pos.x + bpos.x - cpos.x,
                        y: anchor_pos.y + bpos.y - cpos.y,
                    };
                    let d = fit(&p, &shape);
                    if d < min_d {
                        min_d = d;
                        res = (i as u16, p)
                    }
                }
            }
        }
        res
    }
    
    //  constructor
    pub fn new(bundle : &Bundle) -> Layout {
        Layout {
            bundle: bundle,
            pos: (0..bundle.len()).map(|i| Position {
                x: 0, y: 0, shape: i as u16, var: 0
            }).collect()
        }
    }
    
    //  shuffles shape order
    pub fn shuffle<T: Rng>(&mut self, rng : &mut T) {
        for i in 0..self.bundle.len() {
            let j = rng.gen_range(0, i + 1);
            let tmp = self.pos[i].shape;
            self.pos[i].shape = self.pos[j].shape;
            self.pos[j].shape = tmp;
        }
    }
    
    pub fn arrange_circle(&mut self) {
        
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::vec2::Vec2i;
    use super::super::shape::Shape;

    #[test]
    fn test_overlap_status() {
        let shape1 = "   \n O\n OOO \n O\n";
        let shape1 = Shape::parse(shape1);

        let shape2 = "   O\n OOO \n O\n";
        let shape2 = Shape::parse(shape2);
        
        assert_eq!(Overlap::Overlap, Layout::overlap_status(&shape1, &shape2, 
            &Vec2i{x: 0, y: 0}, &Vec2i{x: 0, y: 0}));
        
        assert_eq!(Overlap::Border, Layout::overlap_status(&shape1, &shape2, 
            &Vec2i{x: 0, y: 0}, &Vec2i{x: 0, y: 3}));
        
        assert_eq!(Overlap::Border, Layout::overlap_status(&shape1, &shape2, 
            &Vec2i{x: 1, y: 1}, &Vec2i{x: 1, y: 4}));
        
        assert_eq!(Overlap::Disjoint, Layout::overlap_status(&shape1, &shape2, 
            &Vec2i{x: 0, y: 0}, &Vec2i{x: 0, y: 4}));
        
        assert_eq!(Overlap::Border, Layout::overlap_status(&shape1, &shape2, 
            &Vec2i{x: -2, y: 0}, &Vec2i{x: 0, y: 2}));
    }
    
    #[test]
    fn test_shape_dist() {
        let shape2 = "   O\n OOO \n O\n";
        let shape2 = Shape::parse(shape2);
        
        let shape3 = "OOOO\n   O\n";
        let shape3 = Shape::parse(shape3);  
        
        assert_eq!(-1, Layout::distance(&shape2, &shape3,
            &Vec2i{x: 0, y: 0}, &Vec2i{x: 0, y: 0}));     

        assert_eq!(0, Layout::distance(&shape2, &shape3,
            &Vec2i{x: 0, y: -3}, &Vec2i{x: 0, y: 0}));     

        assert_eq!(1, Layout::distance(&shape2, &shape3,
            &Vec2i{x: 0, y: -4}, &Vec2i{x: 0, y: 0}));     

        assert_eq!(2, Layout::distance(&shape2, &shape3,
            &Vec2i{x: 0, y: -4}, &Vec2i{x: 1, y: 1}));     
    }
    
    #[test]
    fn test_shape_dist2() {
        let shape1 = "OOOO\nO\n";
        let shape1 = Shape::parse(shape1);
        
        let shape2 = "O\nO\nO\nO\nO\n";
        let shape2 = Shape::parse(shape2);
        
        assert_eq!(0, Layout::distance(&shape1, &shape2,
            &Vec2i{x: 0, y: 1}, &Vec2i{x: 3, y: 2}));     
        
        assert_eq!(-1, Layout::distance(&shape1, &shape2,
            &Vec2i{x: 0, y: 1}, &Vec2i{x: 0, y: 2}));    
    }
    
    #[test]
    fn test_best_fit() {
        let shape1 = "OOOO\nO\n";
        let shape1 = Shape::parse(shape1);
        
        let shape2 = "OOOOO\n";
        let shape2 = Shape::parse(shape2);
        
        let variants = shape2.variants(true, true);
        
        let pos1 = Vec2i{ x: 0, y: 1 };
        let res = Layout::best_fit(&shape1, &pos1, 
            &variants, |pos, shape| {
                let d = Layout::distance(&shape1, &shape, &pos1, &pos);
                if d != 0 {return 1000.0}
                (-(pos.x + shape.width)) as f64
            }); 
        
        assert_eq!(2, variants.len());
        assert_eq!((1, Vec2i{ x: 4, y: 1 }), res);
    }
    
}
