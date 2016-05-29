use std::cmp;
use std::f64;
use super::vec2::Vec2i;
use std::f64::consts::{PI};

//  neighbor offsets, horizontal/vertical
pub const OFFS: [[i32; 2]; 4] = [[1, 0], [0, 1], [-1, 0], [0, -1]];

pub enum Rotation {
    None = 0, // no rotation
    CW90 = 1, // 90 degrees clockwise
    CW180 = 2, // 180 degrees clockwise
    CW270 = 3, // 270 degrees clockwise
}

// Polyomino shape
pub struct Shape {
    pub squares: Vec<Vec2i>,
    pub width: i32,
    pub height: i32,

    pub boundary: Vec<Vec2i>,
    mask: Vec<bool>,
}

// compare with [x, y]
impl PartialEq for Shape {
    fn eq(&self, rhs: &Shape) -> bool {
        self.width == rhs.width && self.mask == rhs.mask
    }
}

impl Shape {
    //  constructor
    fn new(squares: Vec<Vec2i>) -> Shape {
        let (w, h) = Shape::extents(&squares);
        let mask = Shape::build_mask(&squares);
        let boundary = Shape::build_boundary(&squares, &mask);
        let mut squares = squares.clone();
        squares.sort();
        Shape { width: w, height: h, squares: squares, mask: mask, boundary: boundary, }
    }

    // finds (width, height) of the square coordinate list
    fn extents(squares: &Vec<Vec2i>) -> (i32, i32) {
        let w = squares.iter().map(|v| v.x).max().unwrap();
        let h = squares.iter().map(|v| v.y).max().unwrap();
        (w + 1, h + 1)
    }

    // builds a boolean mask (bitmap) for a square list
    fn build_mask(squares: &Vec<Vec2i>) -> Vec<bool> {
        let (w, h) = Shape::extents(squares);
        let mut res = vec![false; (w*h) as usize];
        for p in squares {
            let idx = p.x + (p.y * (w as i32));
            res[idx as usize] = true;
        }
        res
    }

    // creates a list of boundary square coordinates
    fn build_boundary(squares: &Vec<Vec2i>, mask: &Vec<bool>) -> Vec<Vec2i> {
        let (w, h) = Shape::extents(squares);
        let mut res = vec![];
        for sq in squares {
            for offs in OFFS.iter() {
                let x = sq.x + offs[0];
                let y = sq.y + offs[1];
                let in_area = x >= 0 && y >= 0 && x < w && y < h;
                let is_set = in_area && mask[(x + y * w) as usize];
                if !is_set {
                    res.push(Vec2i { x: x, y: y });
                }
            }
        }
        res.sort();
        res.dedup();
        res
    }

    // parses a shape from string representation (newline separated)
    pub fn parse(input: &str) -> Shape {
        let squares = input.lines()
        .enumerate()
        .flat_map(|(j, line)| {
            line.chars()
            .enumerate()
            .filter(|&(_, c)| c != ' ')
            .map(move |(i, _)| {
                Vec2i {x: i as i32, y: j as i32,}
            })
        })
        .collect();
        Shape::new(squares)
    }

    //  returns mirrored shape
    fn mirrored(&self) -> Shape {
        let squares = self.squares.iter().map(|s| {
            Vec2i {x: self.width - s.x - 1, y: s.y,}
        });
        Shape::new(squares.collect())
    }

    //  returns shape, rotated by given amount
    fn rotated(&self, rot: Rotation) -> Shape {
        let (w, h) = (self.width, self.height);
        let squares = self.squares.iter().map(|s| {
            let (x, y) = match rot {
                Rotation::CW90 => (h - s.y - 1, s.x),
                Rotation::CW180 => (w - s.x - 1, h - s.y - 1),
                Rotation::CW270 => (s.y, w - s.x - 1),
                Rotation::None => (s.x, s.y),
            };
            Vec2i { x: x, y: y }
        });
        Shape::new(squares.collect())
    }
    
    //  returns set of possible shape transformed variants
    pub fn variants(&self, mirrored: bool, rotated: bool) -> Vec<Shape> {
        let mut res = vec![];
        {
            let mut add_shape = |shape| {
                if !res.contains(&shape) { res.push(shape); }
            };
            let s = self.rotated(Rotation::None);
            if rotated {
                add_shape(s.rotated(Rotation::CW90));
                add_shape(s.rotated(Rotation::CW180));
                add_shape(s.rotated(Rotation::CW270));
            }
            add_shape(s);
            
            if mirrored {
                let m = self.mirrored();
                if rotated {
                    add_shape(m.rotated(Rotation::CW90));
                    add_shape(m.rotated(Rotation::CW180));
                    add_shape(m.rotated(Rotation::CW270));
                }
                add_shape(m);
            }
        }
        res
    }

    //  returns true if square at given coordinate is present
    pub fn is_set(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 &&
        x < self.width && y < self.height &&
        self.mask[(x + y * self.width) as usize]
    }
    
    //  very approximate "length" of the shape
    fn estimate_len(&self) -> f64 {
        cmp::max(self.width, self.height) as f64
    }
    
    //  measurement of "distance" from the shape to the circle 
    //  with given radius and centered at (0, 0)
    fn dist_to_circle(&self, radius: f64, pos: &Vec2i) -> f64 {
        self.squares.iter().map(|p| {
            let cp = Vec2i {x:pos.x + p.x, y:pos.y + p.y};
            let dr = cp.len() - radius;
            dr*dr
        }).fold(0.0, |sum, i| sum + i)
    }
    
    //  returns the range of angles (from (0,0)) that this shape spans
    fn angle_range(&self, pos: &Vec2i) -> (f64, f64) {
        self.squares.iter()
        .fold((f64::INFINITY, -f64::NEG_INFINITY), |(amin, amax), p| {
            let x = (pos.x + p.x) as f64;
            let y = (pos.y + p.y) as f64;
            let mut ang = y.atan2(x);
            if ang < 0.0 { ang += 2.0*PI; }
            (amin.min(ang), amax.max(ang))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shape_parse1() {
        let shape = "*\n*** \n*\n";
        let shape = Shape::parse(shape);
        assert_eq!(shape.squares, vec![[0, 0], [0, 1], [0, 2], [1, 1], [2, 1]]);
        assert_eq!((shape.width, shape.height), (3, 3));

        assert!(shape.is_set(0, 0));
        assert!(shape.is_set(1, 1));
        assert!(!shape.is_set(1, 0));
        assert!(!shape.is_set(5, 10));
        assert!(!shape.is_set(-1, 0));
        
        assert_eq!(shape.boundary, 
            vec![[-1, 0], [-1, 1], [-1, 2],
                [0, -1], [0, 3], [1, 0], [1, 2],
                [2, 0], [2, 2], [3, 1]]);
        
    }

    #[test]
    fn test_shape_parse2() {
        let shape = "   *\n****\n";
        let shape = Shape::parse(shape);
        assert_eq!(shape.squares, vec![[0, 1], [1, 1], [2, 1], [3, 0], [3, 1]]);
        assert_eq!((shape.width, shape.height), (4, 2));
    }

    #[test]
    fn test_shape_mirrored() {
        let shape = "* **\n   *\n";
        let shape = Shape::parse(shape);
        assert_eq!(shape.mirrored().squares,
                   vec![[0, 0], [0, 1], [1, 0], [3, 0]]);
        assert_eq!((shape.width, shape.height), (4, 2));
    }

    #[test]
    fn test_shape_rotated() {
        let shape = "****\n   *\n";
        let shape = Shape::parse(shape);
        assert_eq!(shape.rotated(Rotation::None).squares,
                   vec![[0, 0], [1, 0], [2, 0], [3, 0], [3, 1]]);
        assert_eq!(shape.rotated(Rotation::CW90).squares,
                   vec![[0, 3], [1, 0], [1, 1], [1, 2], [1, 3]]);
        assert_eq!(shape.rotated(Rotation::CW180).squares,
                   vec![[0, 0], [0, 1], [1, 1], [2, 1], [3, 1]]);
        assert_eq!(shape.rotated(Rotation::CW270).squares,
                   vec![[0, 0], [0, 1], [0, 2], [0, 3], [1, 0]]);
    }
    
    #[test]
    fn test_variants1() {
        let shape = "****";
        let shape = Shape::parse(shape);
        assert_eq!(2, shape.variants(true, true).len());
    }
    
    #[test]
    fn test_variants2() {
        let shape = "*\n*\n***";
        let shape = Shape::parse(shape);
        assert_eq!(4, shape.variants(true, true).len());
    }
}
