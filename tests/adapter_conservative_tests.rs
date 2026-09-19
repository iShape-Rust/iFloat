#![cfg(feature = "core")]

use i_float::adapter::{FloatPointAdapter, FloatPointAdapterScaleError};
use i_float::float::{number::FloatNumber, rect::FloatRect};
use i_float::int::number::int::IntNumber;

fn check_rounding_margin<F: FloatNumber, I: IntNumber>() {
    let limit = I::ONE << (I::BITS - 3);
    for radius in [1.0, 1.99999, 3.0] {
        let r = F::from_float(radius);
        let points = [[-r, -r], [r, r]];
        let rect = FloatRect::new(-r, r, -r, r).unwrap();
        let automatic = FloatPointAdapter::<[F; 2], I>::new_conservative(rect);
        let scale = automatic.dir_scale();
        let adapters = [
            automatic,
            FloatPointAdapter::with_iter_conservative(points.iter()),
            FloatPointAdapter::try_with_scale_conservative(rect, scale).unwrap(),
            FloatPointAdapter::try_with_iter_and_scale_conservative(points.iter(), scale).unwrap(),
        ];
        for adapter in adapters {
            for point in &points {
                let integer = adapter.try_float_to_int(point).unwrap();
                assert!(integer.is_in_safe_range());
                assert!(-limit <= integer.x && integer.x <= limit);
                assert!(-limit <= integer.y && integer.y <= limit);
            }
        }
        let too_large = scale * F::from_float(2.0);
        assert_eq!(
            FloatPointAdapter::<[F; 2], I>::try_with_scale_conservative(rect, too_large).err(),
            Some(FloatPointAdapterScaleError::ScaleTooLarge),
        );
        assert_eq!(
            FloatPointAdapter::<[F; 2], I>::try_with_iter_and_scale_conservative(points.iter(), too_large)
                .err(),
            Some(FloatPointAdapterScaleError::ScaleTooLarge),
        );
    }
}

#[test]
fn conservative_constructors_reserve_rounding_margin() {
    check_rounding_margin::<f32, i16>();
    check_rounding_margin::<f32, i32>();
    check_rounding_margin::<f32, i64>();
    check_rounding_margin::<f64, i16>();
    check_rounding_margin::<f64, i32>();
    check_rounding_margin::<f64, i64>();
}
