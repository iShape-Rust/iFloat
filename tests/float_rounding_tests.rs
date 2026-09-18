#![cfg(feature = "core")]

use i_float::float::number::FloatNumber;

fn assert_small_round<F: FloatNumber>(value: F, expected: i16) {
    assert_eq!(value.to_round_i16(), expected);
    assert_eq!(value.to_round_i32(), i32::from(expected));
    assert_eq!(value.to_round_i64(), i64::from(expected));
    assert_eq!(value.to_round_i128(), i128::from(expected));
    assert_eq!(value.to_round_usize(), expected.max(0) as usize);
}

macro_rules! rounding_tests {
    ($module:ident, $float:ty, $boundary:expr) => {
        mod $module {
            use super::*;

            #[test]
            fn rounds_half_and_neighbors_away_from_zero() {
                for (half, below, rounded) in [(0.5, 0, 1), (1.5, 1, 2), (2.5, 2, 3)] {
                    let half = half as $float;
                    for (value, expected) in [
                        (<$float>::from_bits(half.to_bits() - 1), below),
                        (half, rounded),
                        (<$float>::from_bits(half.to_bits() + 1), rounded),
                    ] {
                        assert_small_round(value, expected);
                        assert_small_round(-value, -expected);
                    }
                }
                assert_small_round(0.0 as $float, 0);
                assert_small_round(-0.0 as $float, 0);
            }

            #[test]
            fn preserves_integers_at_precision_boundary() {
                for expected in ($boundary - 2)..=($boundary + 3) {
                    for signed in [expected, -expected] {
                        let value = signed as $float;
                        assert_eq!(value.to_round_i64(), signed);
                        assert_eq!(value.to_round_i128(), i128::from(signed));
                        if let Ok(expected_i32) = i32::try_from(signed) {
                            assert_eq!(value.to_round_i32(), expected_i32);
                        }
                        if let Ok(expected_usize) = usize::try_from(signed) {
                            assert_eq!(value.to_round_usize(), expected_usize);
                        }
                    }
                }
            }

            #[test]
            fn preserves_saturating_cast_behavior() {
                for value in [<$float>::INFINITY, <$float>::MAX] {
                    assert_eq!(value.to_round_i16(), i16::MAX);
                    assert_eq!(value.to_round_i32(), i32::MAX);
                    assert_eq!(value.to_round_i64(), i64::MAX);
                    assert_eq!(value.to_round_i128(), i128::MAX);
                    assert_eq!(value.to_round_usize(), usize::MAX);

                    assert_eq!((-value).to_round_i16(), i16::MIN);
                    assert_eq!((-value).to_round_i32(), i32::MIN);
                    assert_eq!((-value).to_round_i64(), i64::MIN);
                    assert_eq!((-value).to_round_i128(), i128::MIN);
                    assert_eq!((-value).to_round_usize(), 0);
                }
                assert_small_round(<$float>::NAN, 0);
            }
        }
    };
}

rounding_tests!(f32_rounding, f32, 1_i64 << 23);
rounding_tests!(f64_rounding, f64, 1_i64 << 52);
