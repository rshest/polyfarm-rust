// ------------------------------------------------------------------------------------------------
// layout.rs
// ------------------------------------------------------------------------------------------------
use std::f64;
use std::i32;
use std::cmp;
use rand::{Rng};
use super::math::*;
use super::shape::{Shape, OFFS};

pub const COFFS: [[i32; 2]; 8] = [[1, 0], [0, 1], [-1, 0], [0, -1], [1, -1], [1, 1], [-1, 1], [-1, -1]];
const MAX_DIST : f64 = 1000.0;

#[derive(PartialEq, Debug)]
pub enum Overlap {
    Overlap, // have a common square
    Border, // have a common edge
    Disjoint, // neither a common square nor edge
}

// A bundle(set) of polyomino shapes
pub type Bundle = Vec<Vec<Shape>>;

//  Polyomino "intance" (both geometrical and variation)
#[derive(Clone, PartialEq)]
pub struct Position {
    pub x : i32,
    pub y : i32,
    pub shape : u16,    //  shape index
    pub var : u16       //  shape variant index
}

#[derive(Clone)]
pub struct Layout<'a> {
    pub bundle : &'a Bundle,
    pub pos : Vec<Position>,
}

impl<'a> PartialEq<Layout<'a>> for Layout<'a> {
    #[inline]
    fn eq(&self, rhs: &Layout<'a>) -> bool {
        self.pos.iter().zip(&rhs.pos).all(|(p1, p2)| p1 == p2)
    }
}

pub fn parse_bundle(input: &str, mirrored: bool, rotated: bool) -> Bundle {
    input.split("\n\n")
    .map(|s| Shape::parse(s).variants(mirrored, rotated))
    .collect()
}

impl Position {
    fn p(&self) -> Vec2i {
        Vec2i{x: self.x, y: self.y}
    }
    
    fn zero() -> Position {
        Position{x: 0, y: 0, shape: 0, var: 0}
    }
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
                shape: i as u16, ..Position::zero()
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
    
    //  get shape by position
    pub fn shape_by_pos(&self, pos: &Position) -> &Shape {
        &self.bundle[pos.shape as usize][pos.var as usize]
    }
       
    //  lays out the chain of shapes along a circle with given radius,
    //  picking positions/variants such that neighbor shapes bound each other    
    pub fn arrange_circle(&mut self, radius : f64) {
        let nshapes = self.pos.len();
        for i in 0..nshapes {
            let shape_idx = self.pos[i].shape;
            let mut var_idx = 0;
            let mut pos = Vec2i{x: 0, y: 0};
            if i == 0 {
                //  place the first shape at the right side of the circle
                let sh = &self.bundle[shape_idx as usize][0];
                pos.x = (radius - (sh.width as f64)*0.5).round() as i32;
                pos.y = (-(sh.height as f64)*0.5).round() as i32;
            } else {
                let prev_pos = &self.pos[i - 1];
                let prev_shape = self.shape_by_pos(prev_pos);
                let (_, pang2) = prev_shape.angle_range(&prev_pos.p());
                let k = i%nshapes;
                let variants = &self.bundle[self.pos[k].shape as usize];
                let res = Layout::best_fit(prev_shape, &prev_pos.p(), 
                    variants, |pos, shape| {
                    if i == nshapes {
                        //  last shape should border with both neighbors
                        let i1 = (i + 1)%nshapes;
                        let next_pos = &self.pos[i1];
                        let next_shape = self.shape_by_pos(next_pos);
                        let d1 = Layout::distance(shape, next_shape, pos, &next_pos.p());
                        if d1 != 0 { return MAX_DIST + (d1.abs() as f64) }
                    }
                    
                    //  check for bordering with the previous
                    let d = Layout::distance(shape, prev_shape, pos, &prev_pos.p());
                    if d != 0 { return MAX_DIST }
                    
                    //  check that we are laying out in right direction
                    let (_, ang2) = shape.angle_range(pos);
                    if angle_greater(ang2, pang2) { return MAX_DIST }
                    
                    //  pick the one with minimum distance to the target circle
                    let dist = shape.dist_to_circle(radius, pos);
                    dist/ang2
                });
                var_idx = res.0;
                pos = res.1;
            }
            self.pos[i] = Position{x: pos.x, y: pos.y, var: var_idx, shape: shape_idx};
        }
    }
    
    //  returns layout bounds, (LeftTop, RightBottom)
    pub fn bounds(&self) -> (Vec2i, Vec2i) {
        let start = (Vec2i{x:i32::MAX, y:i32::MAX}, Vec2i{x:i32::MIN, y:i32::MIN});
        self.pos.iter().fold(start, |(lt, rb), pos| {
            let sh = self.shape_by_pos(pos);
            (Vec2i{x: cmp::min(lt.x, pos.x), y: cmp::min(lt.y, pos.y)},
             Vec2i{x: cmp::max(rb.x, pos.x + sh.width), 
                   y: cmp::max(rb.y, pos.y + sh.height)})
        })
    }
    
    //  centers the layout around (0, 0)
    pub fn center(&mut self) {
        let (lt, rb) = self.bounds();
        let cx = (rb.x + lt.x)/2;
        let cy = (rb.y + lt.y)/2;
        self.pos = self.pos.iter().map(|p| 
            Position{x: p.x - cx, y: p.y - cy, ..*p}
        ).collect();
    }
    
    fn flood_fill<F>(&self, mut hit_fn: F) -> Option<usize> 
        where F : FnMut(i32, i32) 
    {
        let (lt, rb) = self.bounds();
        let w = rb.x - lt.x + 1;
        let h = rb.y - lt.y + 1;
        
        // create the mask
        let mut mask = vec![false; (w*h) as usize];
        for p in &self.pos {
            let shape = self.shape_by_pos(&p);
            for sq in &shape.squares {
                let x = p.x + sq.x - lt.x;
                let y = p.y + sq.y - lt.y;
                mask[(x + y*w) as usize] = true;
            }
        }
        
        //  compute the starting point
        let mut sx = w/2;
        let mut sy = h/2;
        if mask[(sx + sy*w) as usize] {
            for offs in COFFS.iter() {
                let cx = sx + offs[0];
                let cy = sy + offs[1];
                if !mask[(cx + cy*w) as usize] {
                    sx = cx; sy = cy;
                    break;
                }
            }
        }
        
        //  do the flood fill
        let mut cellq = vec![];
        cellq.push(Vec2i{x: sx, y: sy});
        mask[(sx + sy*w) as usize] = true;
        let mut nvisited = 0;
            
        loop {
            let c = match cellq.pop() {
                Some(p) => p,
                None => break
            };
            hit_fn(c.x + lt.x, c.y + lt.y);
            nvisited += 1;
            for offs in COFFS.iter() {
                let cx = c.x + offs[0];
                let cy = c.y + offs[1];
                if cx < 0 || cy < 0 || cx >= w || cy >= h { return None; }
                let idx = (cx + cy*w) as usize;
                if !mask[idx] {
                    cellq.push(Vec2i{x: cx, y: cy});
                    mask[idx] = true;
                }
            }
        }    
        Some(nvisited)
    }
    
    //  computes "score" heuristic for the layout
    pub fn score(&self) -> f64 {
        let nshapes = self.pos.len();
        match self.flood_fill(|_, _| {}) {
            Some(n) => n as f64, // closed area (a donut)
            None => {
                //  non-closed area (a brezel)
                let dist = self.pos.iter().enumerate().map(|(i, p)| {
                    let sh = self.shape_by_pos(p);
                    let p1 = &self.pos[(i + 1)%nshapes as usize];
                    let sh1 = self.shape_by_pos(&p1);
                    Layout::distance(sh, sh1, &p.p(), &p1.p()).abs() as f64
                }).fold(0.0, |sum, i| sum + i);
                -dist
            }
        }
    }
    
    pub fn extract_core(&self) -> Option<(Shape, Vec2i)> {
        let mut squares = vec![];
        let (mut cx, mut cy) = (i32::MAX, i32::MAX);
        let num_visited = self.flood_fill(|x, y| {
            cx = cmp::min(cx, x);
            cy = cmp::min(cy, y);
            squares.push(Vec2i{x: x, y: y});
        });
        
        match num_visited {
            Some(_) => {
                squares = squares.iter().map(|p| Vec2i{x: p.x - cx, y: p.y - cy}).collect();
                Some((Shape::new(squares), Vec2i{x: cx, y: cy}))            
            },
            None => None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::math::*;
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
