use super::vec2::Vec2i;

const OFFS: [[i32; 2]; 4] = [[1, 0], [0, 1], [-1, 0], [0, -1]];
const COFFS: [[i32; 2]; 8] = [[1, 0], [0, 1], [-1, 0], [0, -1], [1, -1], [1, 1], [-1, 1], [-1, -1]];

enum Rotation {
    None = 0, // no rotation
    CW90 = 1, // 90 degrees clockwise
    CW180 = 2, // 180 degrees clockwise
    CW270 = 3, // 270 degrees clockwise
}

enum Overlap {
    Overlap, // have a common square
    Border, // have a common edge
    Disjoint, // neither a common square nor edge
}

pub struct Shape {
    pub squares: Vec<Vec2i>,
    pub width: u32,
    pub height: u32,
}

impl Shape {
    fn parse(input: &str) -> Shape {
        let squares = input.lines().enumerate();/*.flat_map(|(row, line)| {
            [[1, row]]
        }).collect();*/
        println!("squares: {:?}", squares.collect());
        Shape {
            width: 4,
            height: 4,
            squares: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shape_parse1() {
        let shape = "   \n*\n*** \n*\n";

        let shape = Shape::parse(shape);
        assert_eq!(shape.squares, vec![[1, 1], [1, 2], [2, 2], [3, 2], [1, 3]]);
        assert_eq!((shape.width, shape.height), (4, 4));
    }

    fn test_shape_parse2() {
        let shape = "   *\n*** \n*\n";
    }

    #[test]
    fn test_shape_mirrored() {
        let shape = "* **\n*\n";
    }

    #[test]
    fn test_shape_rotated() {
        let shape = "****\n*\n";
    }
}
