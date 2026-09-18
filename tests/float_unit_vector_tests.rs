#![cfg(feature = "core")]
use i_float::int::unit_vector::UnitIntVector;

macro_rules! float_conversion {
    ($name:ident, $int:ty, $wide:ty) => {
        #[test]
        fn $name() {
            let s = UnitIntVector::<$int>::DENOMINATOR;
            for (x, y) in [
                (1.0, 0.0),
                (0.0, -1.0),
                (0.3, 0.4),
                (-0.3, -0.4),
                (0.5, 0.5),
            ] {
                let unit = UnitIntVector::<$int>::try_from_float(x, y).unwrap();
                assert_eq!(unit, UnitIntVector::<$int>::from_float_unchecked(x, y));
                let (a, b) = (unit.x() as $wide, unit.y() as $wide);
                assert!(a * a + b * b <= s * s);
                assert!((a as f64 / s as f64 - x).abs() <= 1.0 / s as f64);
                assert!((b as f64 / s as f64 - y).abs() <= 1.0 / s as f64);
                let opposite = UnitIntVector::<$int>::try_from_float(-x, -y).unwrap();
                assert_eq!((opposite.x(), opposite.y()), (-unit.x(), -unit.y()));
            }
            for step in 0..10000 {
                let angle = step as f64 * core::f64::consts::TAU / 10000.0;
                let (sin, cos) = libm::sincos(angle);
                let inward = 1.0 - 8.0 * f64::EPSILON;
                let unit = UnitIntVector::<$int>::try_from_float(cos * inward, sin * inward).unwrap();
                assert_eq!(
                    unit,
                    UnitIntVector::<$int>::from_float_unchecked(cos * inward, sin * inward)
                );
                let (a, b) = (unit.x() as $wide, unit.y() as $wide);
                assert!(a * a + b * b <= s * s);
            }
            let axis = UnitIntVector::<$int>::try_from_float(1.0_f32, 0.0).unwrap();
            assert_eq!(axis.x() as $wide, s);
            assert_eq!(axis.y(), 0);
            for (x, y) in [
                (0.0, 0.0),
                (1.0, 1.0),
                (1.01, 0.0),
                (f64::NAN, 0.0),
                (0.0, f64::INFINITY),
            ] {
                assert!(UnitIntVector::<$int>::try_from_float(x, y).is_none());
            }
            for non_finite in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
                for finite in [0.0, 0.5, -1.0] {
                    for (x, y) in [(non_finite, finite), (finite, non_finite)] {
                        assert!(UnitIntVector::<$int>::try_from_float(x, y).is_none());
                        assert!(UnitIntVector::<$int>::try_from_float(x as f32, y as f32).is_none());
                    }
                }
            }
        }
    };
}
float_conversion!(float_i16, i16, i32);
float_conversion!(float_i32, i32, i64);
float_conversion!(float_i64, i64, i128);

#[test]
fn i64_checks_length_below_float_resolution() {
    // In f64, 1 + 2^-60 rounds to 1. The exact fixed-scale norm is larger.
    assert!(UnitIntVector::<i64>::try_from_float(1.0, 1.0 / (1_u64 << 30) as f64).is_none());
}
