
const COFFS: [[i32; 2]; 8] = [[1, 0], [0, 1], [-1, 0], [0, -1], [1, -1], [1, 1], [-1, 1], [-1, -1]];

enum Overlap {
    Overlap, // have a common square
    Border, // have a common edge
    Disjoint, // neither a common square nor edge
}

const MAX_DIST: f64 = 1e5;


#[cfg(test)]
mod tests {
    use super::*;

    const SHAPE1: &'static str = "   \n*\n*** \n*\n";

    const SHAPE2: &'static str = "   *\n*** \n*\n";

    const SHAPE3: &'static str = "****\n*\n";

    #[test]
    fn test_overlap_status() {}

    #[test]
    fn test_shape_dist() {}
}
