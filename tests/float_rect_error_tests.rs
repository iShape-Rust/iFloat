#![cfg(feature = "core")]

use core::error::Error;
use i_float::float::rect::{FloatRect, FloatRectError};

#[test]
fn rectangle_errors_support_display_and_error_from_public_api() {
    let cases = [
        (
            FloatRect::new(f64::NAN, 1.0, 0.0, 1.0).unwrap_err(),
            FloatRectError::CoordinatesOutOfRange,
            "A coordinate is non-finite or exceeds the supported absolute limit",
        ),
        (
            FloatRect::new(1.0, 0.0, 0.0, 1.0).unwrap_err(),
            FloatRectError::InvalidBounds,
            "A minimum bound is greater than its maximum",
        ),
    ];

    for (error, expected_variant, expected_message) in cases {
        assert_eq!(error, expected_variant);
        assert_eq!(error.to_string(), expected_message);

        let source: &(dyn Error + 'static) = &error;
        assert_eq!(source.to_string(), expected_message);
        assert!(source.source().is_none());
        assert_eq!(source.downcast_ref::<FloatRectError>(), Some(&error));
    }
}
