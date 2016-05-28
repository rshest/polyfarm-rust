use super::shape::Shape;

// A bundle(set) of polyomino shapes
pub struct Bundle {
    pub variants: Vec<Vec<Shape>>
}

impl Bundle {
    pub fn parse(input: &str, mirrored: bool, rotated: bool) -> Bundle {
        Bundle {
            variants: input.split("\n\n").map(|s| 
                Shape::parse(s).variants(mirrored, rotated)).collect()
        }
    }
}