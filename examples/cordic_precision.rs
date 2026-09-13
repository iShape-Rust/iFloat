//! Controlled runtime precision experiment; not an arc builder or public policy.
//! The CORDIC kernel is shared by source with the library, never copied.
#[path = "../src/int/angle/cordic.rs"]
mod cordic;

use i_float::int::{
    angle::{Angle, Rotation},
    vector::IntVector,
};
use std::{
    f64::consts::TAU,
    hint::black_box,
    time::{Duration, Instant},
};

const COUNT: usize = 2048;
const SCALE: f64 = (1u64 << 30) as f64;
const BINARY: f64 = (1u64 << 32) as f64;

#[derive(Clone, Copy, Debug)]
struct Precision {
    iterations: usize,
    q: u32,
    relative: u32, // 0: explicit iterations; 8/16: rotation error divisor
}
impl Precision {
    fn rotation_iterations(self, bits: u32) -> usize {
        if self.relative == 0 {
            return self.iterations;
        }
        cordic::rotation_iterations(
            bits,
            self.relative.trailing_zeros(),
            Rotation::<i32>::ANGLE_MAX_ERROR,
        )
    }
}

#[derive(Debug)]
struct Options {
    precision: Precision,
    step: f64,
    radius: u32,
    sweep: bool,
    reverse: bool,
}

fn options(args: impl IntoIterator<Item = String>) -> Result<Option<Options>, String> {
    let mut result = Options {
        precision: Precision {
            iterations: cordic::ITERATIONS,
            q: 30,
            relative: 0,
        },
        step: 1.0,
        radius: 65536,
        sweep: false,
        reverse: false,
    };
    let mut args = args.into_iter();
    let mut explicit = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => return Ok(None),
            "--sweep" => result.sweep = true,
            "--reverse" => result.reverse = true,
            "--iterations" | "--q" | "--relative" | "--step" | "--radius" => {
                let value = args.next().ok_or_else(|| format!("missing value for {arg}"))?;
                let invalid = || format!("invalid value for {arg}: {value}");
                match arg.as_str() {
                    "--iterations" => {
                        result.precision.iterations = value.parse().map_err(|_| invalid())?;
                        explicit = true;
                    }
                    "--q" => {
                        result.precision.q = value.parse().map_err(|_| invalid())?;
                        explicit = true;
                    }
                    "--relative" => {
                        result.precision.relative = value.parse().map_err(|_| invalid())?;
                        if ![8, 16].contains(&result.precision.relative) {
                            return Err("--relative must be 8 or 16".into());
                        }
                    }
                    "--step" => result.step = value.parse().map_err(|_| invalid())?,
                    _ => result.radius = value.parse().map_err(|_| invalid())?,
                }
            }
            _ => return Err(format!("unknown argument: {arg}")),
        }
    }
    if !(1..=cordic::ITERATIONS).contains(&result.precision.iterations) {
        return Err(format!("--iterations must be 1..{}", cordic::ITERATIONS));
    }
    if !(14..=30).contains(&result.precision.q) {
        return Err("--q must be 14..30".into());
    }
    if !result.step.is_finite() || !(360.0 / 1024.0..=45.0).contains(&result.step) {
        return Err("--step must be 0.3515625..45 degrees".into());
    }
    if result.radius == 0 || result.radius > 1 << 30 {
        return Err("--radius must be 1..1073741824".into());
    }
    if result.precision.relative != 0 && ![8, 16].contains(&result.precision.relative) {
        return Err("--relative must be 8 or 16".into());
    }
    if result.precision.relative != 0 && explicit {
        return Err(format!(
            "--relative keeps atan2 at {} iterations and coefficients Q30; do not combine with --iterations/--q",
            cordic::ITERATIONS
        ));
    }
    if result.sweep && (explicit || result.precision.relative != 0) {
        return Err("use --sweep or individual --iterations/--q/--relative values".into());
    }
    Ok(Some(result))
}

#[derive(Clone, Copy)]
struct Case {
    a: [i32; 2],
    b: [i32; 2],
    cw: bool,
    sweep: f64,
    segments: usize,
}

fn cases(step: f64) -> Vec<Case> {
    let mut state = 0xd718af03u64;
    let mut next = || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        (state >> 11) as f64 / (1u64 << 53) as f64
    };
    let normalize = |angle: f64| {
        let v = IntVector::<i32>::new(
            (angle.cos() * 1e6).round() as i64,
            (angle.sin() * 1e6).round() as i64,
        )
        .fast_normalize()
        .unwrap();
        [v.x(), v.y()]
    };
    (0..COUNT)
        .map(|_| {
            let start = next() * TAU;
            let sweep = (0.1 + next() * 179.8).to_radians();
            let cw = next() < 0.5;
            let a = normalize(start);
            let b = normalize(start + if cw { -sweep } else { sweep });
            let mut sweep = (b[1] as f64).atan2(b[0] as f64) - (a[1] as f64).atan2(a[0] as f64);
            if cw {
                sweep = -sweep;
            }
            if sweep < 0.0 {
                sweep += TAU;
            }
            // Fixed reference work across configurations. No low-precision profile
            // can appear faster merely by underestimating its segment count.
            let segments = (sweep / step.to_radians()).ceil().max(1.0) as usize;
            Case {
                a,
                b,
                cw,
                sweep,
                segments,
            }
        })
        .collect()
}

fn atan2(y: i64, x: i64, iterations: usize) -> u32 {
    if y == 0 {
        return if x < 0 { 1 << 31 } else { 0 };
    }
    if x == 0 {
        return if y > 0 { 1 << 30 } else { 3 << 30 };
    }
    let (ax, ay) = (x.unsigned_abs(), y.unsigned_abs());
    let top = ax.max(ay).ilog2();
    let input_top = cordic::VECTOR_INPUT_BITS - 1;
    let reduce = |v: u64| -> i64 {
        (if top > input_top {
            v >> (top - input_top)
        } else {
            v << (input_top - top)
        }) as i64
            * (1i64 << cordic::VECTOR_GUARD_BITS)
    };
    let mut z = cordic::vectoring(reduce(ax), reduce(ay), iterations);
    if x < 0 {
        z = (1 << 31) - z;
    }
    let magnitude = z.clamp(1, (1 << 31) - 1) as u32;
    if y < 0 {
        magnitude.wrapping_neg()
    } else {
        magnitude
    }
}

#[derive(Clone, Copy)]
struct Matrix {
    sin: i64,
    cos: i64,
}
impl Matrix {
    fn from_coefficients((sin, cos): (i32, i32), q: u32) -> Self {
        // Q14..Q30 quantization, all stored in Q30 for the same i64 multiply
        // and constant division in every profile. This isolates precision
        // from the cost of runtime division or changing vector storage.
        Self {
            sin: (sin as i64) << (30 - q),
            cos: (cos as i64) << (30 - q),
        }
    }
    #[inline]
    fn apply(self, v: [i32; 2]) -> [i32; 2] {
        let (x, y) = (v[0] as i64, v[1] as i64);
        [
            ((self.cos * x - self.sin * y) / (1 << 30)) as i32,
            ((self.sin * x + self.cos * y) / (1 << 30)) as i32,
        ]
    }
}

#[inline]
fn setup<const FIXED: bool>(case: Case, p: Precision) -> (u32, Matrix) {
    let (ax, ay, bx, by) = (
        case.a[0] as i64,
        case.a[1] as i64,
        case.b[0] as i64,
        case.b[1] as i64,
    );
    let cross = ax * by - ay * bx;
    let cross = if case.cw { -cross } else { cross };
    let dot = ax * bx + ay * by;
    let sweep = if FIXED || p.relative != 0 {
        Angle::atan2(cross, dot).unwrap().bits()
    } else {
        atan2(cross, dot, p.iterations)
    };
    let step = sweep / case.segments as u32;
    let step = if case.cw { step.wrapping_neg() } else { step };
    let coefficients = if FIXED {
        Angle::from_bits(step).sin_cos()
    } else {
        cordic::sin_cos(step, p.rotation_iterations(step), p.q)
    };
    (
        sweep,
        Matrix::from_coefficients(coefficients, if FIXED { 30 } else { p.q }),
    )
}

fn measure(mut f: impl FnMut()) -> (f64, f64, f64) {
    f();
    let start = Instant::now();
    let mut loops = 0usize;
    while start.elapsed() < Duration::from_millis(40) {
        f();
        loops += 1;
    }
    let repeats = (50_000_000.0 / (start.elapsed().as_nanos() as f64 / loops as f64)).max(1.0) as usize;
    let mut times = [0.0; 7];
    for time in &mut times {
        let start = Instant::now();
        for _ in 0..repeats {
            f();
        }
        *time = start.elapsed().as_nanos() as f64 / (repeats * COUNT) as f64;
    }
    times.sort_by(f64::total_cmp);
    (times[3], times[0], times[6])
}

#[derive(Default, Debug)]
struct Error {
    arc: f64,
    atan_bits: f64,
    drift: f64,
    radial: f64,
    step_excess: f64,
    bad_order: usize,
    rotation_iterations_min: usize,
    rotation_iterations_max: usize,
    rotation_iterations_sum: usize,
    rotation_relative_error: f64,
}

fn accuracy<const FIXED: bool>(data: &[Case], p: Precision, step: f64) -> Error {
    let mut error = Error {
        rotation_iterations_min: cordic::ITERATIONS,
        ..Error::default()
    };
    for &case in data {
        let (sweep, matrix) = setup::<FIXED>(case, p);
        let step_bits = sweep / case.segments as u32;
        let n = if FIXED {
            cordic::ITERATIONS
        } else {
            p.rotation_iterations(step_bits)
        };
        error.rotation_iterations_min = error.rotation_iterations_min.min(n);
        error.rotation_iterations_max = error.rotation_iterations_max.max(n);
        error.rotation_iterations_sum += n;
        let requested = step_bits as f64 * TAU / BINARY;
        if requested > 0.0 {
            let actual = (matrix.sin as f64).atan2(matrix.cos as f64) * if case.cw { -1.0 } else { 1.0 };
            error.rotation_relative_error = error
                .rotation_relative_error
                .max((actual - requested).abs() / requested);
        }
        error.atan_bits = error
            .atan_bits
            .max((sweep as f64 - case.sweep * BINARY / TAU).abs());
        let mut v = case.a;
        let direction = if case.cw { -1.0 } else { 1.0 };
        let mut travelled = 0.0;
        for k in 1..=case.segments {
            let previous = v;
            v = matrix.apply(v);
            let angle = direction * case.sweep * k as f64 / case.segments as f64;
            let (sin, cos) = angle.sin_cos();
            let expected = [
                (cos * case.a[0] as f64 - sin * case.a[1] as f64) / SCALE,
                (sin * case.a[0] as f64 + cos * case.a[1] as f64) / SCALE,
            ];
            error.arc = error
                .arc
                .max((v[0] as f64 / SCALE - expected[0]).hypot(v[1] as f64 / SCALE - expected[1]));
            if k < case.segments {
                let (x, y, px, py) = (v[0] as f64, v[1] as f64, previous[0] as f64, previous[1] as f64);
                let delta = direction * (px * y - py * x).atan2(px * x + py * y);
                if delta <= 0.0 {
                    error.bad_order += 1;
                }
                travelled += delta;
                error.step_excess = error.step_excess.max(delta - step.to_radians());
            }
        }
        // Gap from the last emitted interior point to the exact input end.
        let final_gap = case.sweep - travelled;
        if final_gap < 0.0 {
            error.bad_order += 1;
        }
        error.step_excess = error.step_excess.max(final_gap - step.to_radians());
    }
    for sample in 0..257u32 {
        let start = sample as f64 * TAU / 257.0;
        let initial = IntVector::<i32>::new(
            (start.cos() * 1e6).round() as i64,
            (start.sin() * 1e6).round() as i64,
        )
        .fast_normalize()
        .unwrap();
        let initial = [initial.x(), initial.y()];
        let initial_norm = (initial[0] as f64).hypot(initial[1] as f64) / SCALE;
        let step = (1 << 22) + ((sample as u64 * ((1 << 29) - (1 << 22))) / 256) as u32;
        for cw in [false, true] {
            let bits = if cw { step.wrapping_neg() } else { step };
            let coeff = if FIXED {
                Angle::from_bits(bits).sin_cos()
            } else {
                cordic::sin_cos(bits, p.rotation_iterations(bits), p.q)
            };
            let matrix = Matrix::from_coefficients(coeff, if FIXED { 30 } else { p.q });
            let mut v = initial;
            for k in 1..=1024 {
                v = matrix.apply(v);
                let angle = (if cw { -(step as f64) } else { step as f64 }) * k as f64 * TAU / BINARY;
                let (sin, cos) = angle.sin_cos();
                let expected = [
                    (cos * initial[0] as f64 - sin * initial[1] as f64) / SCALE,
                    (sin * initial[0] as f64 + cos * initial[1] as f64) / SCALE,
                ];
                error.drift = error
                    .drift
                    .max((v[0] as f64 / SCALE - expected[0]).hypot(v[1] as f64 / SCALE - expected[1]));
                error.radial = error
                    .radial
                    .max(initial_norm - (v[0] as f64).hypot(v[1] as f64) / SCALE);
            }
        }
    }
    error
}

fn bench<const FIXED: bool>(data: &[Case], p: Precision, options: &Options) {
    let step_bits: Vec<_> = data
        .iter()
        .map(|&case| {
            let (sweep, _) = setup::<true>(case, p);
            let bits = sweep / case.segments as u32;
            if case.cw { bits.wrapping_neg() } else { bits }
        })
        .collect();
    let api_rotation_time = if FIXED || p.relative != 0 {
        Some(
            measure(|| {
                let p = black_box(p);
                for &bits in black_box(&step_bits) {
                    let angle = Angle::from_bits(bits);
                    black_box(if FIXED {
                        Rotation::<i32>::new(angle)
                    } else {
                        Rotation::<i32>::with_precision(angle, p.relative.trailing_zeros())
                    });
                }
            })
            .0,
        )
    } else {
        None
    };
    let rotation_time = measure(|| {
        let p = black_box(p);
        for &bits in black_box(&step_bits) {
            black_box(if FIXED {
                Angle::from_bits(bits).sin_cos()
            } else {
                cordic::sin_cos(bits, p.rotation_iterations(bits), p.q)
            });
        }
    });
    let setup_time = measure(|| {
        let p = black_box(p);
        for &case in black_box(data) {
            black_box(setup::<FIXED>(case, p));
        }
    });
    let mut output = Vec::with_capacity(1024);
    let arc_time = measure(|| {
        let p = black_box(p);
        for &case in black_box(data) {
            output.clear();
            if case.segments > 1 {
                let (_, matrix) = setup::<FIXED>(case, p);
                let mut v = case.a;
                for _ in 1..case.segments {
                    v = matrix.apply(v);
                    output.push(v);
                }
            }
            black_box(&output);
        }
    });
    let error = accuracy::<FIXED>(data, p, options.step);
    let points = data.iter().map(|c| c.segments - 1).sum::<usize>() as f64 / data.len() as f64;
    let mode = if FIXED {
        "library_fixed"
    } else if p.relative != 0 {
        "relative"
    } else {
        "runtime"
    };
    println!(
        "{mode},{},{},{:.8},{},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.6},{:.6},{:.6},{:.6},{:.9},{},{},{},{},{:.3},{:.9},{:.3},{}",
        p.iterations,
        p.q,
        options.step,
        options.radius,
        setup_time.0,
        arc_time.0,
        arc_time.1,
        arc_time.2,
        arc_time.0 / points.max(1.0),
        points,
        error.atan_bits,
        error.arc * options.radius as f64,
        error.drift * options.radius as f64,
        error.radial * options.radius as f64,
        error.step_excess.to_degrees(),
        error.bad_order,
        p.relative,
        error.rotation_iterations_min,
        error.rotation_iterations_max,
        error.rotation_iterations_sum as f64 / data.len() as f64,
        error.rotation_relative_error,
        rotation_time.0,
        api_rotation_time.map(|v| format!("{v:.3}")).unwrap_or_default()
    );
}

fn main() {
    let options = match options(std::env::args().skip(1)) {
        Ok(Some(v)) => v,
        Ok(None) => {
            println!(
                "Usage: cordic_precision [--iterations 1..{iterations}] [--q 14..30] [--relative 8|16] [--step degrees] [--radius integer] [--sweep] [--reverse]\nDefaults: {iterations} iterations, Q30, step 1 degree, radius 65536.\n--relative 8/16 automatically reduces only rotation iterations for an angular error budget step/8 or step/16; atan2 stays accurate and coefficients stay Q30.\n--sweep compares explicit and relative precision profiles; cannot combine with --iterations/--q/--relative.\nCSV timings use fixed reference segment counts and exclude input normalization. Errors include computed endpoints, before grid rounding. Experimental profiles have no maximum-step guarantee.",
                iterations = cordic::ITERATIONS
            );
            return;
        }
        Err(e) => {
            eprintln!("{e}; use --help");
            std::process::exit(2);
        }
    };
    let data = cases(options.step);
    let mut profiles = if options.sweep {
        vec![
            Precision {
                iterations: cordic::ITERATIONS,
                q: 30,
                relative: 0,
            },
            Precision {
                iterations: 24,
                q: 30,
                relative: 0,
            },
            Precision {
                iterations: 20,
                q: 30,
                relative: 0,
            },
            Precision {
                iterations: 15,
                q: 30,
                relative: 0,
            },
            Precision {
                iterations: cordic::ITERATIONS,
                q: 24,
                relative: 0,
            },
            Precision {
                iterations: cordic::ITERATIONS,
                q: 20,
                relative: 0,
            },
            Precision {
                iterations: cordic::ITERATIONS,
                q: 14,
                relative: 0,
            },
            Precision {
                iterations: 24,
                q: 24,
                relative: 0,
            },
            Precision {
                iterations: 20,
                q: 20,
                relative: 0,
            },
            Precision {
                iterations: 15,
                q: 14,
                relative: 0,
            },
            Precision {
                iterations: cordic::ITERATIONS,
                q: 30,
                relative: 8,
            },
            Precision {
                iterations: cordic::ITERATIONS,
                q: 30,
                relative: 16,
            },
        ]
    } else {
        vec![options.precision]
    };
    println!(
        "mode,iterations,q,step_deg,radius,setup_median_ns,arc_median_ns,arc_min_ns,arc_max_ns,amortized_ns_point,points_arc,atan_max_error_bits,arc_max_error_units,drift_1024_max_error_units,radial_1024_max_loss_units,max_step_excess_deg,nonmonotone_gaps,relative_divisor,rotation_iterations_min,rotation_iterations_max,rotation_iterations_mean,rotation_max_relative_error,rotation_median_ns,api_rotation_median_ns"
    );
    if !options.reverse {
        bench::<true>(
            &data,
            Precision {
                iterations: cordic::ITERATIONS,
                q: 30,
                relative: 0,
            },
            &options,
        );
    }
    if options.reverse {
        profiles.reverse();
    }
    for p in profiles {
        bench::<false>(&data, p, &options);
    }
    if options.reverse {
        bench::<true>(
            &data,
            Precision {
                iterations: cordic::ITERATIONS,
                q: 30,
                relative: 0,
            },
            &options,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_profile_matches_public_kernel_and_rotation() {
        use i_float::int::angle::Rotation;
        let p = Precision {
            iterations: cordic::ITERATIONS,
            q: 30,
            relative: 0,
        };
        for case in cases(1.0) {
            let (a, ma) = setup::<true>(case, p);
            let (b, mb) = setup::<false>(case, p);
            assert_eq!(a, b);
            assert_eq!((ma.sin, ma.cos), (mb.sin, mb.cos));
            let from = IntVector::<i32>::new(case.a[0] as i64, case.a[1] as i64)
                .fast_normalize()
                .unwrap();
            let bits = if case.cw {
                (a / case.segments as u32).wrapping_neg()
            } else {
                a / case.segments as u32
            };
            let rotated = Rotation::new(Angle::from_bits(bits)).apply(from);
            assert_eq!(ma.apply([from.x(), from.y()]), [rotated.x(), rotated.y()]);
        }
    }
    #[test]
    fn every_profile_keeps_coefficients_nonexpanding() {
        for iterations in 1..=cordic::ITERATIONS {
            for q in 14..=30 {
                for bits in (0..=u32::MAX).step_by(1_048_573) {
                    let (sin, cos) = cordic::sin_cos(bits, iterations, q);
                    assert!(sin as i64 * sin as i64 + cos as i64 * cos as i64 <= 1i64 << (2 * q));
                }
            }
        }
    }
    #[test]
    fn relative_budget_selects_minimum_count_and_preserves_direction() {
        for divisor in [8, 16] {
            let p = Precision {
                iterations: cordic::ITERATIONS,
                q: 30,
                relative: divisor,
            };
            for sample in 0..65536u32 {
                let bits = (BINARY * (0.1 + 44.9 * sample as f64 / 65535.0) / 360.0) as u32;
                let n = p.rotation_iterations(bits);
                assert_eq!(n, p.rotation_iterations(bits.wrapping_neg()));
                let budget = bits >> divisor.trailing_zeros();
                let bound = |n: usize| 683_565_276u32.div_ceil(1 << (n - 1)) + 2;
                assert!(bound(n) <= budget);
                if n > 1 {
                    assert!(bound(n - 1) > budget);
                }
                for signed in [bits, bits.wrapping_neg()] {
                    let (s, c) = cordic::sin_cos(signed, n, 30);
                    let expected = (signed as i32 as f64) * TAU / BINARY;
                    let actual = (s as f64).atan2(c as f64);
                    assert!((actual - expected).abs() <= expected.abs() / divisor as f64);
                    assert_eq!(s.signum(), (signed as i32).signum());
                    let public = Rotation::with_precision(Angle::from_bits(signed), divisor.trailing_zeros());
                    let axis = IntVector::<i32>::new(1, 0).fast_normalize().unwrap();
                    let v = public.apply(axis);
                    assert_eq!([v.y(), v.x()], [s, c]);
                }
            }
            for step in [0.3515625, 1.0, 5.0, 20.0, 45.0] {
                let errors = accuracy::<false>(&cases(step), p, step);
                assert!(errors.rotation_relative_error <= 1.0 / divisor as f64);
                assert!(errors.atan_bits <= Angle::MAX_ERROR as f64);
            }
        }
        let angle20 = (BINARY / 18.0) as u32;
        assert_eq!(
            Precision {
                iterations: cordic::ITERATIONS,
                q: 30,
                relative: 8
            }
            .rotation_iterations(angle20),
            6
        );
        assert_eq!(
            Precision {
                iterations: cordic::ITERATIONS,
                q: 30,
                relative: 16
            }
            .rotation_iterations(angle20),
            7
        );
    }

    #[test]
    fn cli_rejects_invalid_ranges_and_conflicting_modes() {
        for args in [
            vec!["--q", "31"],
            vec!["--iterations", "0"],
            vec!["--step", "NaN"],
            vec!["--radius", "0"],
            vec!["--sweep", "--q", "14"],
            vec!["--q"],
            vec!["--unknown"],
            vec!["--relative", "0"],
            vec!["--relative", "10"],
            vec!["--relative", "16", "--iterations", "15"],
            vec!["--relative", "8", "--q", "30"],
            vec!["--relative", "16", "--sweep"],
        ] {
            assert!(options(args.into_iter().map(str::to_owned)).is_err());
        }
        assert!(options(["--iterations", "15", "--q", "14"].into_iter().map(str::to_owned)).is_ok());
    }
}
