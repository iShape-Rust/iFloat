#![cfg(feature = "core")]
use i_float::int::{unit_vector::UnitIntVector, vector::IntVector};

macro_rules! check_normalize {
    ($name:ident, $int:ty, $wide:ty) => {
        #[test]
        fn $name() {
            let scale = UnitIntVector::<$int>::DENOMINATOR;
            let check = |x: $wide, y: $wide| {
                let result = UnitIntVector::<$int>::normalize_with_float(IntVector::new(x, y));
                if x == 0 && y == 0 {
                    assert!(result.is_none());
                    return;
                }
                let unit = result.expect("every nonzero wide vector can be normalized");
                let (a, b) = (unit.x() as $wide, unit.y() as $wide);
                assert!(a != 0 || b != 0);
                let length = (x as f64).hypot(y as f64);
                let tolerance = 1.0 / scale as f64 + 8.0 * f64::EPSILON;
                let norm = (a as f64 / scale as f64).hypot(b as f64 / scale as f64);
                assert!((norm - 1.0).abs() <= 2.0 * tolerance);
                assert!((a as f64 / scale as f64 - x as f64 / length).abs() <= tolerance);
                assert!((b as f64 / scale as f64 - y as f64 / length).abs() <= tolerance);
                if x != <$wide>::MIN && y != <$wide>::MIN {
                    let opposite =
                        UnitIntVector::<$int>::normalize_with_float(IntVector::new(-x, -y)).unwrap();
                    assert_eq!((opposite.x(), opposite.y()), (-unit.x(), -unit.y()));
                }
            };
            let values = [
                <$wide>::MIN,
                <$wide>::MIN + 1,
                -scale,
                -4,
                -1,
                0,
                1,
                3,
                scale,
                <$wide>::MAX,
            ];
            for x in values {
                for y in values {
                    check(x, y);
                }
            }
            for power in 0..(<$wide>::BITS - 1) {
                let n = (1 as $wide) << power;
                for x in [n - 1, n, n + 1] {
                    for y in [1, n / 3, n - 1, n] {
                        check(x, y);
                        check(-x, y);
                    }
                }
            }
        }
    };
}
check_normalize!(normalize_i16, i16, i32);
check_normalize!(normalize_i32, i32, i64);
check_normalize!(normalize_i64, i64, i128);

#[test]
fn near_axis_roundoff_does_not_reject_nonzero_vector() {
    let unit = UnitIntVector::<i64>::normalize_with_float(IntVector::new(1 << 30, 1)).unwrap();
    let scale = UnitIntVector::<i64>::DENOMINATOR;
    let (x, y) = (unit.x() as i128, unit.y() as i128);
    // f64 loses the small squared component. A slight overshoot is intentional.
    assert!(x * x + y * y > scale * scale);
    assert!((x as f64 / scale as f64 - 1.0).abs() <= f64::EPSILON);
    assert!(y > 0);
}
