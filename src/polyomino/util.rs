
pub fn angle_greater(a: f64, b: f64) -> bool {
    use std::f64::consts::{PI};
    if a < b && b - a > PI { true }
    else { a > b && a - b < PI }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_angle_greater() {
        assert!(angle_greater(1.0, 0.0));
        assert!(!angle_greater(0.0, 0.0));
        assert!(!angle_greater(0.0, 1.0));

        assert!(angle_greater(2.0, 1.0));
        assert!(angle_greater(1.0, 6.0));
        assert!(!angle_greater(6.0, 1.0));
    }
}