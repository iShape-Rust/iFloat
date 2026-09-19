#![cfg(feature = "core")]

use i_float::float::number::FloatNumber;
use i_float::int::angle::{Angle, AngleDelta};
use i_float::int::number::wide_int::WideIntNumber;
use std::f64::consts::TAU;

#[test]
fn angle_radians_support_both_float_types() {
    for (angle, unsigned, signed) in [
        (Angle::ZERO, 0.0, 0.0),
        (Angle::QUARTER_TURN, TAU / 4.0, TAU / 4.0),
        (Angle::HALF_TURN, TAU / 2.0, -TAU / 2.0),
        (Angle::THREE_QUARTER_TURN, 3.0 * TAU / 4.0, -TAU / 4.0),
    ] {
        assert_eq!(angle.to_radians::<f64>(), unsigned);
        assert_eq!(angle.to_signed_radians::<f64>(), signed);
        assert_eq!(angle.to_radians::<f32>(), unsigned as f32);
        assert_eq!(angle.to_signed_radians::<f32>(), signed as f32);
        let delta = Angle::ZERO.delta_to(angle);
        assert_eq!(delta.to_radians::<f64>(), signed);
        assert_eq!(delta.to_radians::<f32>(), signed as f32);
    }
    let last = Angle::from_bits(u32::MAX);
    assert!(last.to_radians::<f64>() < TAU);
    assert!(last.to_signed_radians::<f64>() < 0.0);
    assert_eq!(last.to_radians::<f32>(), TAU as f32);
    let before_half = AngleDelta::from_raw(i32::MAX);
    assert!(before_half.to_radians::<f64>() < TAU / 2.0);
    assert_eq!(before_half.to_radians::<f32>(), (TAU / 2.0) as f32);
}

#[test]
fn angle_arithmetic_wraps_and_chooses_shortest_delta() {
    const BACKWARD: AngleDelta = Angle::ZERO.delta_to(Angle::THREE_QUARTER_TURN);
    const WRAPPED: Angle = Angle::ZERO.wrapping_add(BACKWARD);
    assert_eq!(BACKWARD.raw(), -(1 << 30));
    assert_eq!(WRAPPED, Angle::THREE_QUARTER_TURN);
    assert_eq!(Angle::default(), Angle::ZERO);
    assert_eq!(AngleDelta::default(), AngleDelta::ZERO);
    assert_eq!(Angle::HALF_TURN + Angle::HALF_TURN, Angle::ZERO);
    assert_eq!(Angle::THREE_QUARTER_TURN + Angle::HALF_TURN, Angle::QUARTER_TURN);
    assert_eq!(Angle::ZERO.delta_to(Angle::HALF_TURN).raw(), i32::MIN);
    assert_eq!(Angle::HALF_TURN.delta_to(Angle::ZERO).raw(), i32::MIN);
    assert_eq!(Angle::from_bits(u32::MAX).delta_to(Angle::ZERO).raw(), 1);
    assert_eq!(Angle::ZERO.delta_to(Angle::from_bits(u32::MAX)).raw(), -1);
    let boundaries = [0, 1, (1 << 31) - 1, 1 << 31, (1 << 31) + 1, u32::MAX];
    for from in boundaries.map(Angle::from_bits) {
        for target in boundaries.map(Angle::from_bits) {
            assert_eq!(from.wrapping_add(from.delta_to(target)), target);
        }
    }
}

#[test]
fn radians_reject_non_finite_inputs() {
    for radians in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(Angle::from_radians(radians), None);
        assert_eq!(Angle::from_radians(radians as f32), None);
    }
    assert_eq!(Angle::from_radians(0.0f32).unwrap().bits(), 0);
    assert_eq!(Angle::from_radians(1.0f32), Angle::from_radians(1.0f64));
}

#[test]
fn radians_wrap_turns_and_preserve_axes() {
    for (radians, bits) in [
        (0.0, 0),
        (-0.0, 0),
        (TAU, 0),
        (-TAU, 0),
        (TAU / 4.0, 1 << 30),
        (-TAU / 4.0, 3 << 30),
        (TAU / 2.0, 1 << 31),
        (-TAU / 2.0, 1 << 31),
        (3.0 * TAU / 4.0, 3 << 30),
        (5.0 * TAU / 4.0, 1 << 30),
        (-5.0 * TAU / 4.0, 3 << 30),
    ] {
        assert_eq!(Angle::from_radians(radians).unwrap().bits(), bits);
    }
}

#[test]
fn radians_round_half_units_away_from_zero() {
    let half_unit = TAU / (1_u64 << 33) as f64;
    let below_half = f64::from_bits(half_unit.to_bits() - 1);
    let above_half = f64::from_bits(half_unit.to_bits() + 1);
    for (radians, bits) in [
        (below_half, 0),
        (-below_half, 0),
        (half_unit, 1),
        (-half_unit, u32::MAX),
        (above_half, 1),
        (-above_half, u32::MAX),
        (TAU - half_unit / 2.0, 0),
        (-TAU + half_unit / 2.0, 0),
    ] {
        assert_eq!(Angle::from_radians(radians).unwrap().bits(), bits);
    }
}

#[test]
fn float_coefficients_preserve_axes_norm_and_accuracy() {
    let scale = Angle::SIN_COS_SCALE;
    for (bits, expected) in [
        (0, (0, scale)),
        (1 << 30, (scale, 0)),
        (1 << 31, (0, -scale)),
        (3 << 30, (-scale, 0)),
    ] {
        assert_eq!(Angle::from_bits(bits).sin_cos_with_float(), expected);
    }
    let near_axes = [0u32, 1 << 30, 1 << 31, 3 << 30]
        .into_iter()
        .flat_map(|axis| [0u32, 1, 2, 3, u32::MAX, u32::MAX - 1].map(|delta| axis.wrapping_add(delta)));
    for bits in (0..=u32::MAX).step_by(16_381).chain(near_axes) {
        let (sin, cos) = Angle::from_bits(bits).sin_cos_with_float();
        let norm2 = sin as i64 * sin as i64 + cos as i64 * cos as i64;
        assert!(norm2 <= scale as i64 * scale as i64, "bits={bits}");
        let radians = bits as f64 * TAU / 4294967296.0;
        let (expected_sin, expected_cos) = radians.sin_cos();
        // Independent native-float reference; allow rounding and contraction.
        let error =
            (sin as f64 - expected_sin * scale as f64).hypot(cos as f64 - expected_cos * scale as f64);
        assert!(error < 2.0, "bits={bits}, error={error}");
    }
}

fn check_atan2<W: WideIntNumber>() {
    assert_eq!(Angle::atan2_with_float(W::ZERO, W::ZERO), None);
    for (y, x, bits) in [
        (W::ZERO, W::ONE, 0),
        (W::ONE, W::ZERO, 1 << 30),
        (W::ZERO, W::MIN, 1 << 31),
        (W::MIN, W::ZERO, 3 << 30),
    ] {
        assert_eq!(Angle::atan2_with_float(y, x).unwrap().bits(), bits);
    }
    let check = |y: W, x: W| {
        let Some(angle) = Angle::atan2_with_float(y, x) else {
            assert!(y == W::ZERO && x == W::ZERO);
            return;
        };
        let bits = angle.bits();
        if y > W::ZERO {
            assert!(bits > 0 && bits < 1 << 31);
        } else if y < W::ZERO {
            assert!(bits > 1 << 31);
        }
        let expected = y.to_f64().atan2(x.to_f64());
        let actual = bits as f64 * TAU / 4294967296.0;
        let difference = (actual - expected).sin().atan2((actual - expected).cos()).abs();
        // Rounding is within half a binary unit, except for the one-unit
        // adjustment that preserves the side of an almost parallel ray.
        assert!(difference * 4294967296.0 / TAU < 1.00001, "y={y}, x={x}");
    };
    let values = [W::MIN, W::MIN + W::ONE, -W::ONE, W::ZERO, W::ONE, W::MAX];
    for y in values {
        for x in values {
            check(y, x);
        }
    }
    // Unequal magnitudes at every power of two, in all quadrants.
    let mut magnitude = W::MAX;
    while magnitude > W::ZERO {
        for x in [magnitude, -magnitude] {
            for y in [W::ONE, -W::ONE, magnitude, -magnitude] {
                check(y, x);
                check(x, y);
                assert_eq!(
                    Angle::atan2_with_float(y, x)
                        .unwrap()
                        .bits()
                        .wrapping_add(Angle::atan2_with_float(-y, x).unwrap().bits()),
                    0
                );
            }
        }
        magnitude = magnitude >> 1;
    }
}

#[test]
fn float_atan2_handles_full_wide_range_and_preserves_turn_side() {
    check_atan2::<i32>();
    check_atan2::<i64>();
    check_atan2::<i128>();
}

#[test]
fn float_number_atan2_returns_signed_radians() {
    for (y, x) in [(1.0f64, 1.0), (-1.0, 1.0), (1.0, -1.0), (-1.0, -1.0)] {
        assert!((FloatNumber::atan2(y, x) - y.atan2(x)).abs() < 1e-15);
        let (y, x) = (y as f32, x as f32);
        assert!((FloatNumber::atan2(y, x) - y.atan2(x)).abs() < 1e-6);
    }
    assert!(FloatNumber::atan2(-0.0f64, 1.0).is_sign_negative());
    assert!(FloatNumber::atan2(-0.0f32, 1.0).is_sign_negative());
}
