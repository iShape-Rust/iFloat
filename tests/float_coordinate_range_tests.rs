#![cfg(feature = "core")]

use i_float::adapter::{FloatPointAdapter, FloatPointAdapterScaleError};
use i_float::float::compatible::FloatPointCompatible;
use i_float::float::number::FloatNumber;
use i_float::float::point::FloatPoint;
use i_float::float::rect::{FloatRect, FloatRectError};
use i_float::float::vector::FloatPointMath;

fn check_coordinate_limits<F: FloatNumber>(limit: F, outside: F, nan: F, infinity: F) {
    assert!(F::MAX_COORDINATE == limit);
    for value in [-F::MIN_POSITIVE, -F::ZERO, F::MIN_POSITIVE] {
        assert!(value.is_in_safe_range());
    }
    for x in [-limit, F::ZERO, limit] {
        assert!(x.is_in_safe_range());
        for y in [-limit, F::ZERO, limit] {
            assert!([x, y].is_in_safe_range());
            assert!(FloatPoint::new(x, y).is_in_safe_range());
            assert!(FloatRect::with_point([x, y]).unwrap().is_in_safe_range());
        }
    }
    assert!(
        FloatRect::new(-limit, limit, -limit, limit)
            .unwrap()
            .is_in_safe_range()
    );
    for invalid in [outside, -outside, nan, infinity, -infinity] {
        assert!(!invalid.is_in_safe_range());
        assert!(![invalid, F::ZERO].is_in_safe_range());
        assert!(![F::ZERO, invalid].is_in_safe_range());
        for bounds in [
            [invalid, limit, -limit, limit],
            [-limit, invalid, -limit, limit],
            [-limit, limit, invalid, limit],
            [-limit, limit, -limit, invalid],
        ] {
            let [min_x, max_x, min_y, max_y] = bounds;
            assert_eq!(
                FloatRect::new(min_x, max_x, min_y, max_y).err(),
                Some(FloatRectError::CoordinatesOutOfRange)
            );
            let raw = FloatRect {
                min_x,
                max_x,
                min_y,
                max_y,
            };
            assert!(!raw.is_in_safe_range());
            assert!(
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    FloatPointAdapter::<[F; 2], i32>::new(raw)
                }))
                .is_err()
            );
            for result in [
                FloatPointAdapter::<[F; 2], i32>::try_with_scale(raw, F::ONE),
                FloatPointAdapter::try_with_scale_and_coordinate_bits(raw, F::ONE, 29),
            ] {
                assert_eq!(
                    result.err(),
                    Some(FloatPointAdapterScaleError::InvalidRect(
                        FloatRectError::CoordinatesOutOfRange
                    ))
                );
            }
        }
        for points in [
            [[invalid, F::ZERO], [F::ZERO, F::ZERO]],
            [[F::ZERO, F::ZERO], [F::ZERO, invalid]],
        ] {
            assert_eq!(
                FloatRect::with_points(&points).err(),
                Some(FloatRectError::CoordinatesOutOfRange)
            );
            assert!(
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    FloatPointAdapter::<[F; 2], i32>::with_iter(points.iter())
                }))
                .is_err()
            );
            assert_eq!(
                FloatPointAdapter::<[F; 2], i32>::with_iter_and_scale_checked(points.iter(), F::ONE).err(),
                Some(FloatPointAdapterScaleError::InvalidRect(
                    FloatRectError::CoordinatesOutOfRange
                ))
            );
        }
        #[cfg(debug_assertions)]
        {
            assert!(
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    FloatPoint::new(invalid, F::ZERO)
                }))
                .is_err()
            );
            assert!(
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    FloatPoint::from_point([F::ZERO, invalid])
                }))
                .is_err()
            );
        }
    }
    // Differences can be larger than input coordinates while their products fit.
    let a = FloatPoint::new(-limit, -limit);
    let b = FloatPoint::new(limit, limit);
    let difference = b - a;
    assert!(difference.sqr_length().is_finite());
    assert!(difference.dot_product(difference).is_finite());
    assert!(difference.cross_product(-difference).is_finite());
    assert!(a.midpoint(b).x == F::ZERO);
    assert!(b.midpoint(b).x == limit);
}

#[test]
fn rectangle_builders_and_mutators_preserve_valid_bounds() {
    fn check<F: FloatNumber>() {
        let limit = F::MAX_COORDINATE;
        assert_eq!(
            FloatRect::new(F::ONE, F::ZERO, F::ZERO, F::ONE).err(),
            Some(FloatRectError::InvalidBounds)
        );
        assert_eq!(
            FloatRect::new(F::ZERO, F::ONE, F::ONE, F::ZERO).err(),
            Some(FloatRectError::InvalidBounds)
        );
        let invalid = FloatRect {
            min_x: F::ONE,
            max_x: F::ZERO,
            min_y: F::ZERO,
            max_y: F::ONE,
        };
        assert!(!invalid.is_in_safe_range());
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                FloatPointAdapter::<[F; 2], i32>::new(invalid)
            }))
            .is_err()
        );
        assert_eq!(
            FloatRect::with_rects(invalid, FloatRect::zero()).err(),
            Some(FloatRectError::InvalidBounds)
        );
        assert_eq!(
            FloatRect::with_optional_rects(None, Some(invalid)).err(),
            Some(FloatRectError::InvalidBounds)
        );

        let empty: [[F; 2]; 0] = [];
        assert!(FloatRect::with_points(&empty).unwrap().is_none());
        let mut optional = None;
        FloatRect::optional_add_point(&mut optional, &[F::ZERO, F::ONE]).unwrap();
        FloatRect::optional_add_point(&mut optional, &[-limit, limit]).unwrap();
        let mut rect = optional.unwrap();
        assert!(rect.min_x == -limit && rect.max_y == limit);
        let before = rect;
        assert_eq!(rect.add_offset(limit), Err(FloatRectError::CoordinatesOutOfRange));
        assert_eq!(rect.add_offset(-limit), Err(FloatRectError::InvalidBounds));
        assert_eq!(
            rect.add_point(&[limit * F::TWO, F::ZERO]),
            Err(FloatRectError::CoordinatesOutOfRange)
        );
        assert!(
            rect.min_x == before.min_x
                && rect.max_x == before.max_x
                && rect.min_y == before.min_y
                && rect.max_y == before.max_y
        );
        rect.add_point(&[limit, -limit]).unwrap();
        assert!(rect.min_x == -limit && rect.max_x == limit && rect.min_y == -limit && rect.max_y == limit);
        rect.add_offset(-limit).unwrap();
        assert!(
            rect.min_x == F::ZERO && rect.max_x == F::ZERO && rect.min_y == F::ZERO && rect.max_y == F::ZERO
        );
    }
    check::<f32>();
    check::<f64>();
}

#[test]
fn f32_coordinate_limits_are_inclusive() {
    let limit = libm::exp2f(60.0);
    check_coordinate_limits(
        limit,
        f32::from_bits(limit.to_bits() + 1),
        f32::NAN,
        f32::INFINITY,
    );
}

#[test]
fn f64_coordinate_limits_are_inclusive() {
    let limit = libm::exp2(500.0);
    check_coordinate_limits(
        limit,
        f64::from_bits(limit.to_bits() + 1),
        f64::NAN,
        f64::INFINITY,
    );
}

fn check_normalization<F: FloatNumber>() {
    for x in [F::MIN_POSITIVE.sqrt(), F::MAX_COORDINATE] {
        let point = FloatPoint::new(x, F::ZERO);
        let direction = point.normalize();
        assert!(direction.x == F::ONE && direction.y == F::ZERO);
        let direction = FloatPointMath::normalize(&[x, F::ZERO]);
        assert!(direction[0] == F::ONE && direction[1] == F::ZERO);
    }
}

#[test]
fn normalization_supports_the_documented_boundaries() {
    check_normalization::<f32>();
    check_normalization::<f64>();
}

#[cfg(debug_assertions)]
#[test]
fn normalization_debug_checks_reject_unsupported_squared_lengths() {
    fn check<F: FloatNumber>() {
        for x in [F::ZERO, F::MIN_POSITIVE, F::MIN_POSITIVE.sqrt() * F::HALF, F::MAX] {
            // Public fields allow constructing a vector to test normalization's
            // own check independently of the point constructor.
            let point = FloatPoint { x, y: F::ZERO };
            assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| point.normalize())).is_err());
            assert!(
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| FloatPointMath::normalize(&[
                    x,
                    F::ZERO
                ])))
                .is_err()
            );
        }
    }
    check::<f32>();
    check::<f64>();
}
