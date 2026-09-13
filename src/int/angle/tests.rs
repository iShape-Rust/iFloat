extern crate std;

use super::*;
use crate::int::number::{int::IntNumber, uint::UIntNumber, wide_int::WideIntNumber};
use crate::int::unit_vector::UnitIntVector;
use crate::int::vector::IntVector;
use core::f64::consts::TAU;

fn unit<I: IntNumber>(x: i64, y: i64) -> UnitIntVector<I> {
    let component = |v: i64| {
        let n = I::Wide::from_uint(I::WideUInt::from_u64(v.unsigned_abs()));
        if v < 0 { -n } else { n }
    };
    IntVector::<I>::new(component(x), component(y))
        .fast_normalize()
        .unwrap()
}

fn check_angles<I: IntNumber>() {
    let axes = [
        unit::<I>(1, 0),
        unit::<I>(0, 1),
        unit::<I>(-1, 0),
        unit::<I>(0, -1),
    ];
    for i in 0..4 {
        for j in 0..4 {
            assert_eq!(
                Angle::between(axes[i], axes[j]).bits(),
                ((j as u32).wrapping_sub(i as u32)).wrapping_mul(1 << 30)
            );
        }
    }
    let a = unit::<I>(3, 4);
    assert_eq!(Angle::between(a, unit::<I>(6, 8)).bits(), 0);
    assert_eq!(Angle::between(a, unit::<I>(-3, -4)).bits(), 1 << 31);
    let denominator = UnitIntVector::<I>::DENOMINATOR;
    let near = UnitIntVector::<I>::from_components(I::from_wide(denominator - I::Wide::ONE), I::ONE);
    assert!(Angle::between(axes[0], near).bits() > 0);
    assert!(Angle::between(near, axes[0]).bits() > 1 << 31);
    let near_opposite = UnitIntVector::<I>::from_components(-near.x(), near.y());
    assert!(Angle::between(axes[0], near_opposite).bits() < 1 << 31);
    assert!(Angle::between(near_opposite, axes[0]).bits() > 1 << 31);

    let mut random = 0x876abcd987u64;
    let mut next = || {
        random ^= random << 13;
        random ^= random >> 7;
        random ^= random << 17;
        random
    };
    let mut max_error = 0.0f64;
    for _ in 0..20_000 {
        // Includes wide i128 products around 2^124 for i64, with independent
        // input lengths. Components chosen inside the unit disk without sqrt.
        let make = |raw: u64| {
            let x = I::Wide::from_u32(raw as u32 & 0xffff);
            let y = I::Wide::from_u32((raw >> 32) as u32 & 0xffff);
            let shift = I::BITS.saturating_sub(19);
            let (x, y) = (x << shift, y << shift);
            // i16 needs small values too; construction always nonzero.
            let x = if I::BITS == 16 {
                (x >> 3) + I::Wide::ONE
            } else {
                x + I::Wide::ONE
            };
            let y = if I::BITS == 16 { y >> 3 } else { y };
            UnitIntVector::<I>::from_components(
                I::from_wide(if raw & 1 == 0 { x } else { -x }),
                I::from_wide(if raw & 2 == 0 { y } else { -y }),
            )
        };
        let (a, b) = (make(next()), make(next()));
        let expected =
            libm::atan2(b.y().to_f64(), b.x().to_f64()) - libm::atan2(a.y().to_f64(), a.x().to_f64());
        let measured = Angle::between(a, b).bits() as f64 * (TAU / 4294967296.0);
        let error = libm::atan2(libm::sin(measured - expected), libm::cos(measured - expected)).abs()
            * 4294967296.0
            / TAU;
        max_error = max_error.max(error);
        assert!(error <= Angle::MAX_ERROR as f64, "{}: {error}", I::BITS);
        let forward = Angle::between(a, b).bits();
        let backward = Angle::between(b, a).bits();
        assert_eq!(forward.wrapping_add(backward), 0);
    }
    std::println!("i{} vectoring max error: {max_error:.3} binary units", I::BITS);
}

#[test]
fn signs_quadrants_and_wide_products() {
    check_angles::<i16>();
    check_angles::<i32>();
    check_angles::<i64>();
}

#[test]
fn coefficient_norm_and_accuracy() {
    let mut max_error = 0.0f64;
    let mut min_norm = 1.0f64;
    for bits in (0..=u32::MAX).step_by(16_381) {
        let (sin, cos) = Angle::from_bits(bits).sin_cos();
        let norm2 = sin as i64 * sin as i64 + cos as i64 * cos as i64;
        assert!(norm2 <= 1 << 60);
        let angle = bits as f64 * TAU / 4294967296.0;
        let (s, c) = (sin as f64 / 1073741824.0, cos as f64 / 1073741824.0);
        let error = libm::hypot(s - libm::sin(angle), c - libm::cos(angle));
        max_error = max_error.max(error);
        min_norm = min_norm.min(libm::hypot(s, c));
        let bound = 1.0 / (1u64 << 28) as f64 + core::f64::consts::SQRT_2 / (1u64 << 30) as f64 + 4e-13;
        assert!(error <= bound, "{bits}: {error}");
    }
    std::println!(
        "rotation coefficient error {max_error:e}, max norm loss {:e}",
        1.0 - min_norm
    );
}

fn drift<I: IntNumber>() {
    let s = UnitIntVector::<I>::DENOMINATOR.to_f64();
    let mut angular = 0.0f64;
    let mut radial = 0.0f64;
    let mut displacement = 0.0f64;
    let mut total = 0.0f64;
    let mut scaled = 0.0f64;
    // All points, including the computed endpoint: no endpoint replacement.
    for sample in 0..257u32 {
        let start_angle = sample as f64 * TAU / 257.0;
        let magnitude = if I::BITS == 16 { 10_000.0 } else { 1_000_000.0 };
        let initial = unit::<I>(
            libm::round(libm::cos(start_angle) * magnitude) as i64,
            libm::round(libm::sin(start_angle) * magnitude) as i64,
        );
        let ix = initial.x().to_f64() / s;
        let iy = initial.y().to_f64() / s;
        let initial_length = libm::hypot(ix, iy);
        // 1/1024 turn through 1/8 turn, both orientations. Not just table angles.
        let step = (1 << 22) + ((sample as u64 * ((1 << 29) - (1 << 22))) / 256) as u32;
        for cw in [false, true] {
            let bits = if cw { step.wrapping_neg() } else { step };
            let r = Rotation::new(Angle::from_bits(bits));
            let mut v = initial;
            for index in 1..=1024 {
                let previous = v;
                v = r.apply(v);
                let length2 = |v: UnitIntVector<I>| {
                    let x = v.x().to_wide();
                    let y = v.y().to_wide();
                    x * x + y * y
                };
                assert!(length2(v) <= length2(previous));
                let angle =
                    (if cw { -(step as f64) } else { step as f64 }) * index as f64 * TAU / 4294967296.0;
                let (sn, cs) = (libm::sin(angle), libm::cos(angle));
                let (ex, ey) = (cs * ix - sn * iy, sn * ix + cs * iy);
                let (x, y) = (v.x().to_f64() / s, v.y().to_f64() / s);
                let length = libm::hypot(x, y);
                displacement = displacement.max(libm::hypot(x - ex, y - ey));
                radial = radial.max(initial_length - length);
                angular = angular.max(libm::atan2(x * ey - y * ex, x * ex + y * ey).abs());
                total = total.max(libm::hypot(x - ex / initial_length, y - ey / initial_length));
                if I::BITS != 16 {
                    let p = v.scale(I::from_u32(65536));
                    scaled = scaled.max(libm::hypot(
                        p.x.to_f64() - 65536.0 * ex,
                        p.y.to_f64() - 65536.0 * ey,
                    ));
                }
            }
        }
    }
    std::println!(
        "i{} /1024 steps: angular={angular:e} rad; at R=1024 / 65536: added={:.6} / {:.6}, radial={:.6} / {:.6}, incl input={:.6} / {:.6}; rounded R65536={scaled:.6}",
        I::BITS,
        displacement * 1024.0,
        displacement * 65536.0,
        radial * 1024.0,
        radial * 65536.0,
        total * 1024.0,
        total * 65536.0
    );
    if I::BITS >= 32 {
        assert!(displacement * 65536.0 < 1.0);
        assert!(radial * 65536.0 < 0.3);
        assert!(scaled < 1.7);
    } else {
        // Q14 loses bits in both the coefficients and every stored result.
        // Matrix norm <=1 makes the sum of the per-step errors a valid bound.
        let per_step = 1.0 / (1u64 << 28) as f64 + 2.0 * libm::sqrt(2.0) / s + 4e-13;
        assert!(displacement <= 1024.0 * per_step);
    }
}

#[test]
fn repeated_rotations_report_accumulation() {
    drift::<i16>();
    drift::<i32>();
    drift::<i64>();
}

#[test]
fn exact_cardinal_rotations_all_types() {
    fn check<I: IntNumber>() {
        let v = unit::<I>(3, 4);
        let quarter = Rotation::new(Angle::from_bits(1 << 30));
        let rotated = quarter.apply(v);
        assert!(rotated.x() == -v.y() && rotated.y() == v.x());
        let mut p = v;
        for _ in 0..1024 {
            p = quarter.apply(p);
        }
        assert!(p.x() == v.x() && p.y() == v.y());
        let p = Rotation::new(Angle::from_bits(0)).apply(v);
        assert!(p.x() == v.x() && p.y() == v.y());
        let p = Rotation::new(Angle::from_bits(1 << 31)).apply(v);
        assert!(p.x() == -v.x() && p.y() == -v.y());
    }
    check::<i16>();
    check::<i32>();
    check::<i64>();
}

#[test]
fn directed_arc_subdivision_and_endpoint_gap() {
    let mut worst_gap_excess = 0.0f64;
    let mut worst_error = 0.0f64;
    let mut max_points = 0u64;
    for start in 0..64 {
        let a = start as f64 * TAU / 64.0;
        for sweep in [
            0.0,
            1e-7,
            0.003,
            TAU / 8.0,
            TAU / 2.0 - 1e-7,
            TAU / 2.0,
            TAU / 2.0 + 1e-7,
            TAU * 0.75,
            TAU - 1e-7,
        ] {
            for clockwise in [false, true] {
                let sign = if clockwise { -1.0 } else { 1.0 };
                let b = a + sign * sweep;
                let make = |v: f64| {
                    unit::<i32>(
                        libm::round(libm::cos(v) * 1e9) as i64,
                        libm::round(libm::sin(v) * 1e9) as i64,
                    )
                };
                let from = make(a);
                let to = make(b);
                let angle = if clockwise {
                    Angle::between(to, from)
                } else {
                    Angle::between(from, to)
                };
                if sweep == 0.0 {
                    assert_eq!(angle.bits(), 0);
                    continue;
                }
                let get_angle = |v: UnitIntVector<i32>| libm::atan2(v.y() as f64, v.x() as f64);
                let mut exact = sign * (get_angle(to) - get_angle(from));
                if exact < 0.0 {
                    exact += TAU;
                }
                for max_step in [1u32 << 22, (4294967296u64 / 360) as u32, 1 << 29] {
                    let upper = angle.bits() as u64 + Angle::MAX_ERROR as u64;
                    let mut n = upper.div_ceil(max_step as u64);
                    while upper + 8 * n * n > n * max_step as u64 {
                        n += 1;
                    }
                    assert!(n >= libm::ceil(exact / (max_step as f64 * TAU / 4294967296.0)) as u64);
                    assert!(n <= 1030);
                    max_points = max_points.max(n - 1);
                    let step = (angle.bits() as u64 / n) as u32;
                    let rotation = Rotation::new(Angle::from_bits(if clockwise {
                        step.wrapping_neg()
                    } else {
                        step
                    }));
                    let mut v = from;
                    let mut previous = 0.0;
                    let norm = libm::hypot(from.x() as f64, from.y() as f64) / 1073741824.0;
                    for k in 1..n {
                        v = rotation.apply(v);
                        let mut travelled = sign * (get_angle(v) - get_angle(from));
                        if travelled < 0.0 {
                            travelled += TAU;
                        }
                        assert!(travelled > previous && travelled < exact);
                        worst_gap_excess =
                            worst_gap_excess.max(travelled - previous - max_step as f64 * TAU / 4294967296.0);
                        previous = travelled;
                        let expected = get_angle(from) + sign * exact * k as f64 / n as f64;
                        let error = libm::hypot(
                            v.x() as f64 / 1073741824.0 - norm * libm::cos(expected),
                            v.y() as f64 / 1073741824.0 - norm * libm::sin(expected),
                        );
                        worst_error = worst_error.max(error * 65536.0);
                    }
                    // Includes the gap to the exact stored endpoint, not an
                    // artificially replaced computed point.
                    worst_gap_excess =
                        worst_gap_excess.max(exact - previous - max_step as f64 * TAU / 4294967296.0);
                }
            }
        }
    }
    std::println!(
        "directed arcs: max {max_points} intermediate points; added error R65536={worst_error:.6}; max step excess from accumulated arithmetic={worst_gap_excess:e} rad"
    );
    assert!(worst_error < 0.5);
    assert!(worst_gap_excess < 1e-12);
}

#[test]
fn public_trigonometry_and_atan2_cover_full_integer_range() {
    fn check<W: WideIntNumber>() {
        assert_eq!(Angle::atan2(W::ZERO, W::ZERO), None);
        for y in [W::MIN, W::MIN + W::ONE, -W::ONE, W::ZERO, W::ONE, W::MAX] {
            for x in [W::MIN, W::MIN + W::ONE, -W::ONE, W::ZERO, W::ONE, W::MAX] {
                let Some(angle) = Angle::atan2(y, x) else {
                    continue;
                };
                let expected = libm::atan2(y.to_f64(), x.to_f64());
                let actual = angle.bits() as f64 * TAU / 4294967296.0;
                let difference =
                    libm::atan2(libm::sin(actual - expected), libm::cos(actual - expected)).abs();
                assert!(difference * 4294967296.0 / TAU <= Angle::MAX_ERROR as f64);
            }
        }
    }
    check::<i32>();
    check::<i64>();
    check::<i128>();
    for (bits, expected) in [
        (0, (0, 1 << 30)),
        (1 << 30, (1 << 30, 0)),
        (1 << 31, (0, -(1 << 30))),
        (3 << 30, (-(1 << 30), 0)),
    ] {
        let angle = Angle::from_bits(bits);
        assert_eq!(angle.sin_cos(), expected);
        assert_eq!(angle.sin(), expected.0);
        assert_eq!(angle.cos(), expected.1);
    }
    for bits in (0..=u32::MAX).step_by(65537) {
        let angle = Angle::from_bits(bits);
        assert_eq!(angle.sin_cos(), (angle.sin(), angle.cos()));
    }
}

#[test]
fn rotation_uses_coordinate_scale() {
    macro_rules! check {
        ($t:ty, $q:expr) => {
            for bits in (0..=u32::MAX).step_by(1_048_573) {
                let angle = Angle::from_bits(bits);
                let rotation = Rotation::<$t>::new(angle);
                let (sin, cos) = angle.sin_cos();
                // Independent rescaling of the public Q30 coefficients.
                let convert = |v: i32| (v as i128 * (1i128 << $q)) / (1i128 << 30);
                let (sin, cos) = (convert(sin), convert(cos));
                for (x, y) in [(1, 0), (0, -1), (3, 4), (-5, 2), (-1, -7)] {
                    let from = unit::<$t>(x, y);
                    let to = rotation.apply(from);
                    let (x, y) = (from.x() as i128, from.y() as i128);
                    assert_eq!(to.x() as i128, (cos * x - sin * y) / (1i128 << $q));
                    assert_eq!(to.y() as i128, (sin * x + cos * y) / (1i128 << $q));
                }
            }
        };
    }
    check!(i16, 14);
    check!(i32, 30);
    check!(i64, 62);
}

#[test]
fn approximate_rotation_reports_achieved_angle_and_respects_budget() {
    fn check_type<I: IntNumber>() {
        let axis = unit::<I>(1, 0);
        let check = |bits: u32, precision: u32| {
            let requested = Angle::from_bits(bits);
            let rotation = Rotation::with_precision(requested, precision);
            // Applying an exact axis exposes the stored coefficients without adding
            // component rounding, independently of the residual-based estimate.
            let v = rotation.apply(axis);
            let actual = libm::atan2(v.y().to_f64(), v.x().to_f64());
            let to_radians = |a: Angle| a.bits() as f64 * TAU / 4294967296.0;
            let difference =
                |a: f64, b: f64| libm::atan2(libm::sin(a - b), libm::cos(a - b)).abs() * 4294967296.0 / TAU;
            assert!(
                difference(actual, to_radians(rotation.angle())) <= Rotation::<I>::ANGLE_MAX_ERROR as f64,
                "achieved angle: bits={bits}, precision={precision}"
            );
            let budget = (bits as i32)
                .unsigned_abs()
                .checked_shr(precision)
                .unwrap_or(0)
                .max(Rotation::<I>::MAX_ERROR);
            assert!(
                difference(actual, to_radians(requested)) <= budget as f64,
                "requested angle: bits={bits}, precision={precision}"
            );
            let (x, y) = (v.x().to_wide(), v.y().to_wide());
            let scale = UnitIntVector::<I>::DENOMINATOR;
            assert!(x * x + y * y <= scale * scale);
            if precision >= 32 {
                let precise = Rotation::new(requested);
                assert_eq!(rotation.angle(), precise.angle());
                assert!(v == precise.apply(axis));
            }
            if bits & ((1 << 30) - 1) == 0 {
                assert_eq!(rotation.angle(), requested);
                assert!(v == Rotation::new(requested).apply(axis));
            }
        };
        for precision in [0, 1, 2, 3, 4, 8, 16, 29, 31, 32, u32::MAX] {
            for bits in (0..=u32::MAX).step_by(65537) {
                check(bits, precision);
            }
            for axis in [0u32, 1 << 30, 1 << 31, 3 << 30] {
                for delta in [0u32, 1, 2, 3, 4, u32::MAX, u32::MAX - 1, u32::MAX - 2] {
                    check(axis.wrapping_add(delta), precision);
                }
            }
        }
    }
    check_type::<i16>();
    check_type::<i32>();
    check_type::<i64>();
}

#[test]
fn achieved_angle_drives_approximate_arc_counts() {
    let mut max_points = 0;
    // Test-only consumer policy: leave margin for one-step error and the final
    // gap, then count by the achieved matrix angle, not the requested one.
    // i32, fresh normalization, relative exponents 3/4, at most 1400 rotations.
    const LIMIT: u64 = 1400;
    const ERROR: u64 = 4; // two metadata units plus component rounding
    for precision in [3u32, 4] {
        for max_step in [
            1u32 << 22,
            (4294967296u64 / 360) as u32,
            (4294967296u64 / 18) as u32,
            1 << 29,
        ] {
            let divisor = 1u64 << precision;
            let reserve = 2 * ERROR * (LIMIT + 1) + 2 * Angle::MAX_ERROR as u64;
            let requested = ((max_step as u64 - reserve) * divisor / (divisor + 1)) as u32;
            for clockwise in [false, true] {
                let rotation = Rotation::with_precision(
                    Angle::from_bits(if clockwise {
                        requested.wrapping_neg()
                    } else {
                        requested
                    }),
                    precision,
                );
                let step = (rotation.angle().bits() as i32).unsigned_abs() as u64;
                assert!(step > ERROR);
                for start in 0..16 {
                    for sweep in [0.001, 0.3, TAU / 2.0, 3.0 * TAU / 4.0, TAU - 0.0001] {
                        let initial = start as f64 * TAU / 16.0;
                        let sign = if clockwise { -1.0 } else { 1.0 };
                        let make =
                            |a: f64| unit::<i32>((libm::cos(a) * 1e6) as i64, (libm::sin(a) * 1e6) as i64);
                        let from = make(initial);
                        let to = make(initial + sign * sweep);
                        let measured = if clockwise {
                            Angle::between(to, from)
                        } else {
                            Angle::between(from, to)
                        };
                        let lower = measured.bits().saturating_sub(Angle::MAX_ERROR) as u64;
                        let count = lower.saturating_sub(1) / (step + ERROR);
                        assert!(count < LIMIT);
                        max_points = max_points.max(count);
                        let angle = |v: UnitIntVector<i32>| libm::atan2(v.y() as f64, v.x() as f64);
                        let mut exact = sign * (angle(to) - angle(from));
                        if exact < 0.0 {
                            exact += TAU;
                        }
                        let mut previous = 0.0;
                        let mut v = from;
                        for _ in 0..count {
                            v = rotation.apply(v);
                            let mut travelled = sign * (angle(v) - angle(from));
                            if travelled < 0.0 {
                                travelled += TAU;
                            }
                            assert!(travelled > previous && travelled < exact);
                            assert!(travelled - previous <= max_step as f64 * TAU / 4294967296.0 + 1e-12);
                            previous = travelled;
                        }
                        assert!(exact - previous <= max_step as f64 * TAU / 4294967296.0 + 1e-12);
                    }
                }
            }
        }
    }
    std::println!(
        "approximate arcs using achieved angle: max {max_points} intermediate points; all points ordered, all gaps within maximum"
    );
}
