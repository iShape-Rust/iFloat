#![cfg(feature = "core")]

use i_float::adapter::{FloatPointAdapter, FloatPointAdapterScaleError};
use i_float::float::number::FloatNumber;
use i_float::float::rect::FloatRect;
use i_float::int::number::int::IntNumber;
use i_float::int::point::IntPoint;

#[test]
fn automatic_constructors_reject_invalid_bounds() {
    // Public fields can bypass FloatRect construction. The adapter must still
    // reject these bounds and invalid points supplied through an iterator.
    let rect = FloatRect {
        min_x: 0.0_f64,
        max_x: f64::INFINITY,
        min_y: 0.0,
        max_y: 0.0,
    };
    assert!(std::panic::catch_unwind(|| FloatPointAdapter::<[f64; 2], i32>::new(rect)).is_err());
    let points = [[0.0_f64, 0.0], [f64::INFINITY, 0.0]];
    assert!(
        std::panic::catch_unwind(|| { FloatPointAdapter::<[f64; 2], i32>::with_iter(points.iter()) })
            .is_err()
    );
}

#[test]
fn empty_iterator_uses_zero_bounds_and_unit_scale() {
    let points: [[f64; 2]; 0] = [];
    let adapter = FloatPointAdapter::<[f64; 2], i32>::with_iter(points.iter());
    assert_eq!(adapter.offset(), [0.0, 0.0]);
    assert_eq!(adapter.dir_scale(), 1.0);
    assert_eq!(adapter.inv_scale(), 1.0);
    assert_eq!(adapter.try_float_to_int(&[0.0, 0.0]), Ok(IntPoint::ZERO));
    assert!(adapter.try_float_to_int(&[1.0, 0.0]).is_err());
}

#[test]
fn tiny_finite_bounds_keep_both_scales_finite() {
    let rect = FloatRect::new(-1e-35_f32, 1e-35, -1e-35, 1e-35).unwrap();
    for (name, adapter) in [
        ("new", FloatPointAdapter::<[f32; 2], i32>::new(rect)),
        (
            "with_coordinate_bits",
            FloatPointAdapter::with_coordinate_bits(rect, 29),
        ),
    ] {
        let scale = adapter.dir_scale();
        let inverse = adapter.inv_scale();
        assert!(scale.is_finite() && scale > 0.0, "{name}: dir_scale={scale}");
        assert!(
            inverse.is_finite() && inverse > 0.0,
            "{name}: inv_scale={inverse}"
        );

        let source = [rect.max_x, 0.0];
        let point = adapter.try_float_to_int(&source).unwrap();
        assert!(point.is_in_safe_range(), "{name}: point={point}");
        let restored = adapter.try_int_to_float(&point).unwrap();
        assert!(
            (restored[0] - source[0]).abs() <= inverse,
            "{name}: restored={restored:?}"
        );
        assert_eq!(restored[1], 0.0);
    }
}

#[test]
fn checked_constructors_reject_scale_with_infinite_reciprocal() {
    let rect = FloatRect::new(-1.0_f32, 1.0, -1.0, 1.0).unwrap();
    let scale = 1e-40_f32;
    assert!(scale.is_finite() && scale > 0.0);
    assert!((1.0 / scale).is_infinite());

    for (name, result) in [
        (
            "try_with_scale",
            FloatPointAdapter::<[f32; 2], i32>::try_with_scale(rect, scale),
        ),
        (
            "try_with_scale_and_coordinate_bits",
            FloatPointAdapter::try_with_scale_and_coordinate_bits(rect, scale, 29),
        ),
    ] {
        assert_eq!(
            result.err(),
            Some(FloatPointAdapterScaleError::ScaleTooSmall),
            "{name}"
        );
    }
}

#[test]
fn supported_boundary_keeps_center_and_scales_finite() {
    let radius = f64::MAX_COORDINATE;
    let rect = FloatRect::new(-radius, radius, -1.0, 1.0).unwrap();
    assert!(rect.width().is_finite());
    for (name, adapter) in [
        ("new", FloatPointAdapter::<[f64; 2], i32>::new(rect)),
        (
            "with_coordinate_bits",
            FloatPointAdapter::with_coordinate_bits(rect, 29),
        ),
        ("with_scale", FloatPointAdapter::with_scale(rect, 1e-145)),
        (
            "try_with_scale",
            FloatPointAdapter::try_with_scale(rect, 1e-145).unwrap(),
        ),
        (
            "try_with_scale_and_coordinate_bits",
            FloatPointAdapter::try_with_scale_and_coordinate_bits(rect, 1e-145, 29).unwrap(),
        ),
    ] {
        assert_eq!(adapter.offset(), [0.0, 0.0], "{name}");
        let scale = adapter.dir_scale();
        let inverse = adapter.inv_scale();
        assert!(scale.is_finite() && scale > 0.0, "{name}: dir_scale={scale}");
        assert!(
            inverse.is_finite() && inverse > 0.0,
            "{name}: inv_scale={inverse}"
        );
        assert_eq!(
            adapter.try_float_to_int(&[0.0, 0.0]),
            Ok(IntPoint::ZERO),
            "{name}"
        );
        assert_eq!(
            adapter.try_int_to_float(&IntPoint::ZERO),
            Ok([0.0, 0.0]),
            "{name}"
        );
    }
}

fn check_extremes<F: FloatNumber, I: IntNumber>(small: F, large: F) {
    for radius in [small, large] {
        let rect = FloatRect::new(-radius, radius, -radius, radius).unwrap();
        for adapter in [
            FloatPointAdapter::<[F; 2], I>::new(rect),
            FloatPointAdapter::with_coordinate_bits(rect, 0),
            FloatPointAdapter::with_coordinate_bits(rect, I::BITS - 3),
        ] {
            assert!(adapter.dir_scale().is_finite() && adapter.dir_scale() > F::ZERO);
            assert!(adapter.inv_scale().is_finite() && adapter.inv_scale() > F::ZERO);
            assert!(adapter.offset() == [F::ZERO, F::ZERO]);
            for source in [[-radius, radius], [F::ZERO, F::ZERO], [radius, -radius]] {
                let integer = adapter.try_float_to_int(&source).unwrap();
                assert!(integer.is_in_safe_range());
                let restored = adapter.try_int_to_float(&integer).unwrap();
                assert!(restored[0].is_finite() && restored[1].is_finite());
                assert!((restored[0] - source[0]).abs() <= adapter.inv_scale());
                assert!((restored[1] - source[1]).abs() <= adapter.inv_scale());
            }
        }
    }
    // Halving this span underflows to zero, but the bounds are not a point.
    let rect = FloatRect::new(F::ZERO, small, F::ZERO, F::ZERO).unwrap();
    let adapter = FloatPointAdapter::<[F; 2], I>::new(rect);
    assert!(adapter.dir_scale() > F::ONE);
    let explicit = FloatPointAdapter::<[F; 2], I>::with_scale(rect, F::TWO);
    assert!(explicit.dir_scale() == F::TWO);
}

#[test]
fn rounded_center_preserves_coordinate_budget_for_adjacent_floats() {
    fn check<F: FloatNumber>(next: F) {
        let rect = FloatRect::new(F::ONE, next, -next, -F::ONE).unwrap();
        let adapter = FloatPointAdapter::<[F; 2], i32>::with_coordinate_bits(rect, 0);
        // The midpoint rounds to an endpoint even around ordinary values like
        // 1.0. Using half the span as radius would exceed the zero-bit budget.
        assert!(adapter.offset() == [F::ONE, -F::ONE]);
        let expected_scale = F::ONE / (next - F::ONE);
        assert!(adapter.dir_scale() == expected_scale);
        for (source, expected) in [
            ([F::ONE, -next], IntPoint::new(0, -1)),
            ([next, -F::ONE], IntPoint::new(1, 0)),
        ] {
            let integer = adapter.try_float_to_int(&source).unwrap();
            assert_eq!(integer, expected);
            assert!(adapter.try_int_to_float(&integer).unwrap() == source);
        }
        assert_eq!(
            FloatPointAdapter::<[F; 2], i32>::try_with_scale_and_coordinate_bits(
                rect,
                expected_scale * F::TWO,
                0,
            )
            .err(),
            Some(FloatPointAdapterScaleError::ScaleTooLarge),
        );
    }
    check(f32::from_bits(1.0_f32.to_bits() + 1));
    check(f64::from_bits(1.0_f64.to_bits() + 1));
}

#[test]
fn finite_scales_cover_subnormals_and_all_coordinate_types() {
    check_extremes::<f32, i16>(f32::from_bits(1), f32::MAX_COORDINATE);
    check_extremes::<f32, i32>(f32::from_bits(1), f32::MAX_COORDINATE);
    check_extremes::<f32, i64>(f32::from_bits(1), f32::MAX_COORDINATE);
    check_extremes::<f64, i16>(f64::from_bits(1), f64::MAX_COORDINATE);
    check_extremes::<f64, i32>(f64::from_bits(1), f64::MAX_COORDINATE);
    check_extremes::<f64, i64>(f64::from_bits(1), f64::MAX_COORDINATE);
}

fn check_explicit_scale_limits<F: FloatNumber>(small: F) {
    for rect in [
        FloatRect::zero(),
        FloatRect::new(-F::ONE, F::ONE, -F::ONE, F::ONE).unwrap(),
    ] {
        assert_eq!(
            FloatPointAdapter::<[F; 2], i32>::try_with_scale(rect, small).err(),
            Some(FloatPointAdapterScaleError::ScaleTooSmall)
        );
        assert_eq!(
            FloatPointAdapter::<[F; 2], i32>::try_with_scale_and_coordinate_bits(rect, small, 29).err(),
            Some(FloatPointAdapterScaleError::ScaleTooSmall)
        );
    }
    // An explicit finite scale above the automatic power-of-two cap is valid
    // when the bounds permit it. Do not silently replace it with that cap.
    let rect = FloatRect::new(-small, small, -small, small).unwrap();
    for adapter in [
        FloatPointAdapter::<[F; 2], i32>::with_scale(rect, F::MAX),
        FloatPointAdapter::try_with_scale(rect, F::MAX).unwrap(),
        FloatPointAdapter::try_with_scale_and_coordinate_bits(rect, F::MAX, 0).unwrap(),
    ] {
        assert!(adapter.dir_scale() == F::MAX);
        assert!(adapter.inv_scale().is_finite() && adapter.inv_scale() > F::ZERO);
    }
}

#[test]
fn explicit_scale_limits_apply_to_both_float_types_and_point_bounds() {
    check_explicit_scale_limits(f32::from_bits(1));
    check_explicit_scale_limits(f64::from_bits(1));
}

#[test]
#[should_panic(expected = "Invalid adapter scale: ScaleTooSmall")]
fn infallible_explicit_constructor_rejects_infinite_reciprocal() {
    FloatPointAdapter::<[f32; 2], i32>::with_scale(FloatRect::new(-1.0, 1.0, -1.0, 1.0).unwrap(), 1e-40);
}

#[test]
#[should_panic(expected = "Invalid adapter bounds")]
fn infallible_constructor_rejects_bounds_outside_the_contract() {
    let rect = FloatRect {
        min_x: -1e308,
        max_x: 1e308,
        min_y: -1.0,
        max_y: 1.0,
    };
    FloatPointAdapter::<[f64; 2], i32>::with_coordinate_bits(rect, 0);
}

#[test]
fn checked_constructor_respects_budget_at_coordinate_limit() {
    let radius = f64::MAX_COORDINATE;
    let rect = FloatRect::new(-radius, radius, -1.0, 1.0).unwrap();
    let scale = libm::exp2(-499.0);
    assert!((1.0 / scale).is_finite());
    assert_eq!(
        FloatPointAdapter::<[f64; 2], i32>::try_with_scale_and_coordinate_bits(rect, scale, 0).err(),
        Some(FloatPointAdapterScaleError::ScaleTooLarge)
    );
    let adapter = FloatPointAdapter::<[f64; 2], i32>::with_coordinate_bits(rect, 1);
    assert_eq!(adapter.dir_scale(), scale);
    assert!(adapter.inv_scale().is_finite());
}
