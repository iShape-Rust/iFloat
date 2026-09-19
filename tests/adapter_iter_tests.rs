#![cfg(feature = "core")]

use i_float::adapter::{FloatPointAdapter, FloatPointAdapterScaleError};
use i_float::float::number::FloatNumber;
use i_float::float::rect::FloatRectError;
use i_float::int::number::int::IntNumber;
use i_float::int::point::IntPoint;

fn check_budget<F: FloatNumber, I: IntNumber>() {
    let three = F::from_float(3.0);
    let points = [[-three, -F::ONE], [three, F::ONE]];
    for bits in [0, 2, I::BITS - 3] {
        let adapter = FloatPointAdapter::<[F; 2], I>::with_iter_and_coordinate_bits(points.iter(), bits);
        let limit = I::ONE << bits;
        for point in &points {
            let integer = adapter.try_float_to_int(point).unwrap();
            assert!(-limit <= integer.x && integer.x <= limit);
            assert!(-limit <= integer.y && integer.y <= limit);
        }
        let scale = adapter.dir_scale();
        assert!(
            FloatPointAdapter::<[F; 2], I>::try_with_iter_and_scale_and_coordinate_bits(
                points.iter(),
                scale,
                bits,
            )
            .is_ok()
        );
        assert_eq!(
            FloatPointAdapter::<[F; 2], I>::try_with_iter_and_scale_and_coordinate_bits(
                points.iter(),
                scale * F::from_float(2.0),
                bits,
            )
            .err(),
            Some(FloatPointAdapterScaleError::ScaleTooLarge)
        );
    }
}

#[test]
fn iterator_constructors_enforce_selected_budget() {
    check_budget::<f32, i16>();
    check_budget::<f32, i32>();
    check_budget::<f32, i64>();
    check_budget::<f64, i16>();
    check_budget::<f64, i32>();
    check_budget::<f64, i64>();
}

type Adapter = FloatPointAdapter<[f64; 2], i32>;

#[test]
fn empty_and_single_point_inputs_preserve_scale_policy() {
    for points in [&[][..], &[[7.0, -3.0]][..]] {
        let origin = points.first().copied().unwrap_or([0.0, 0.0]);
        let automatic = Adapter::with_iter_and_coordinate_bits(points.iter(), 29);
        let fixed = Adapter::try_with_iter_and_scale_and_coordinate_bits(points.iter(), 100.0, 29).unwrap();
        assert_eq!(automatic.dir_scale(), 1.0);
        assert_eq!(fixed.dir_scale(), 100.0);
        for adapter in [automatic, fixed] {
            assert_eq!(adapter.offset(), origin);
            assert_eq!(adapter.try_float_to_int(&origin), Ok(IntPoint::ZERO));
            assert_eq!(adapter.try_int_to_float(&IntPoint::ZERO), Ok(origin));
        }
        for (scale, error) in [
            (0.0, FloatPointAdapterScaleError::ScaleNonPositive),
            (-1.0, FloatPointAdapterScaleError::ScaleNonPositive),
            (f64::NAN, FloatPointAdapterScaleError::ScaleNotFinite),
            (f64::INFINITY, FloatPointAdapterScaleError::ScaleNotFinite),
            (f64::from_bits(1), FloatPointAdapterScaleError::ScaleTooSmall),
        ] {
            assert_eq!(
                Adapter::try_with_iter_and_scale_and_coordinate_bits(points.iter(), scale, 29).err(),
                Some(error)
            );
        }
    }
}

#[test]
fn invalid_coordinates_are_rejected_anywhere_in_iterator() {
    for invalid in [f64::NAN, f64::INFINITY, 2.0 * f64::MAX_COORDINATE] {
        for points in [[[invalid, 0.0], [0.0, 0.0]], [[0.0, 0.0], [0.0, invalid]]] {
            assert_eq!(
                Adapter::try_with_iter_and_scale_and_coordinate_bits(points.iter(), 1.0, 29).err(),
                Some(FloatPointAdapterScaleError::InvalidRect(
                    FloatRectError::CoordinatesOutOfRange
                ))
            );
            assert!(
                std::panic::catch_unwind(|| { Adapter::with_iter_and_coordinate_bits(points.iter(), 29) })
                    .is_err()
            );
        }
    }
}

#[test]
fn invalid_budget_panics_even_for_empty_input() {
    assert!(
        std::panic::catch_unwind(|| {
            Adapter::with_iter_and_coordinate_bits(core::iter::empty(), i32::BITS - 1)
        })
        .is_err()
    );
    assert!(
        std::panic::catch_unwind(|| {
            Adapter::try_with_iter_and_scale_and_coordinate_bits(core::iter::empty(), 1.0, i32::BITS - 1)
        })
        .is_err()
    );
}
