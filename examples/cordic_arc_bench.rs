// Adapted from /private/tmp/ioverlay-arc-bench.JE1wuF/src/main.rs.
// Run: cargo run --release --example cordic_arc_bench [-- reverse]

use i_float::float::number::FloatNumber;
use i_float::int::number::{int::IntNumber, wide_int::WideIntNumber};
use i_float::int::unit_vector::UnitIntVector;
use i_float::int::vector::IntVector;
#[path = "support/bisection.rs"]
mod bisection;
use bisection::{ArcBuilder, ArcDirection, ArcStep};
use i_float::int::angle::{Angle, Rotation};
use std::{
    hint::black_box,
    time::{Duration, Instant},
};
const COUNT: usize = 2048;
#[derive(Clone, Copy)]
struct Case {
    a: [f64; 2],
    b: [f64; 2],
    cw: bool,
}
fn cases(short: bool) -> Vec<Case> {
    let mut rng = 0xd718af03_u64;
    let mut next = || {
        rng ^= rng << 13;
        rng ^= rng >> 7;
        rng ^= rng << 17;
        (rng >> 11) as f64 / ((1u64 << 53) as f64)
    };
    (0..COUNT)
        .map(|_| {
            let start = next() * std::f64::consts::TAU;
            let sweep = (0.1 + next() * if short { 9.9 } else { 179.8 }).to_radians();
            let cw = next() < 0.5;
            let end = start + if cw { -sweep } else { sweep };
            Case {
                a: [start.cos(), start.sin()],
                b: [end.cos(), end.sin()],
                cw,
            }
        })
        .collect()
}
fn measure(mut f: impl FnMut() -> usize) -> (f64, f64, f64, f64) {
    let points = f();
    let t = Instant::now();
    let mut loops = 0usize;
    while t.elapsed() < Duration::from_millis(60) {
        black_box(f());
        loops += 1;
    }
    let repeats = ((70_000_000.0 / (t.elapsed().as_nanos() as f64 / loops as f64)) as usize).max(1);
    let mut times = Vec::new();
    for _ in 0..7 {
        let t = Instant::now();
        for _ in 0..repeats {
            black_box(f());
        }
        times.push(t.elapsed().as_nanos() as f64 / (repeats * COUNT) as f64);
    }
    times.sort_by(f64::total_cmp);
    (times[3], times[0], times[6], points as f64 / COUNT as f64)
}
fn report(name: &str, group: &str, deg: f64, result: (f64, f64, f64, f64)) {
    println!(
        "{name},{group},{deg:.8},{:.3},{:.3},{:.3},{:.3}",
        result.0, result.1, result.2, result.3
    );
}
fn bench_int<I: IntNumber>(data: &[Case], group: &str, deg: f64, name: &str) {
    let normalize = |p: [f64; 2]| {
        IntVector::<I>::new(
            I::Wide::from_rounded_float(p[0] * 1_000_000.0),
            I::Wide::from_rounded_float(p[1] * 1_000_000.0),
        )
        .fast_normalize()
        .unwrap()
    };
    let input: Vec<_> = data
        .iter()
        .map(|c| {
            (
                normalize(c.a),
                normalize(c.b),
                if c.cw {
                    ArcDirection::Clockwise
                } else {
                    ArcDirection::Counterclockwise
                },
            )
        })
        .collect();
    let s = UnitIntVector::<I>::DENOMINATOR.to_f64();
    let chord = 4.0 * (deg.to_radians() / 2.0).sin().powi(2);
    let step = ArcStep::from_squared_chord(I::Wide::from_rounded_float(chord * s * s).to_uint());
    let mut builder = ArcBuilder::<I>::new();
    report(
        name,
        group,
        deg,
        measure(|| {
            let mut count = 0;
            for &(a, b, d) in black_box(&input) {
                let points = builder.build(a, b, d, black_box(step));
                count += points.len();
                black_box(points);
            }
            count
        }),
    );
}
// RoundJoinBuilder's angle/count/rotation kernel, with the same FloatNumber
// libm calls. Both variants emit intermediate unit directions into a reused
// buffer, matching ArcBuilder's output contract. ceil is the comparison with
// a true maximum step; floor reproduces the existing float implementation.
fn bench_float<F: FloatNumber>(data: &[Case], group: &str, deg: f64, ceil: bool, name: &str) {
    let input: Vec<_> = data
        .iter()
        .map(|c| {
            (
                [F::from_float(c.a[0]), F::from_float(c.a[1])],
                [F::from_float(c.b[0]), F::from_float(c.b[1])],
                c.cw,
            )
        })
        .collect();
    let step = F::from_float(deg.to_radians());
    let inv = F::ONE / step;
    let limit = step.cos();
    let mut output = Vec::<[F; 2]>::with_capacity(1024);
    report(
        name,
        group,
        deg,
        measure(|| {
            let mut count = 0;
            for &(a, b, cw) in black_box(&input) {
                output.clear();
                let dot = a[0] * b[0] + a[1] * b[1];
                if black_box(limit) < dot {
                    black_box(&output);
                    continue;
                }
                let angle = dot.max(-F::ONE).min(F::ONE).acos();
                let ratio = angle * black_box(inv);
                let mut n = ratio.to_usize();
                if ceil && F::from_usize(n) < ratio {
                    n += 1;
                }
                n = n.max(1);
                let delta = angle / F::from_usize(n);
                let (sn, cs) = (if cw { -delta } else { delta }).sin_cos();
                let mut v = a;
                for _ in 1..n {
                    v = [cs * v[0] - sn * v[1], sn * v[0] + cs * v[1]];
                    output.push(v);
                }
                count += output.len();
                black_box(&output);
            }
            count
        }),
    );
}
fn bench_cordic<I: IntNumber>(data: &[Case], group: &str, deg: f64, name: &str) {
    let normalize = |p: [f64; 2]| {
        IntVector::<I>::new(
            I::Wide::from_rounded_float(p[0] * 1_000_000.0),
            I::Wide::from_rounded_float(p[1] * 1_000_000.0),
        )
        .fast_normalize()
        .unwrap()
    };
    let input: Vec<_> = data
        .iter()
        .map(|c| (normalize(c.a), normalize(c.b), c.cw))
        .collect();
    let max_step = ((deg / 360.0 * 4294967296.0) as u32).clamp(1 << 22, 1 << 29);
    let mut output = Vec::with_capacity(1024);
    report(
        name,
        group,
        deg,
        measure(|| {
            let mut count = 0;
            for &(a, b, cw) in black_box(&input) {
                output.clear();
                let sweep = if cw {
                    Angle::between(b, a)
                } else {
                    Angle::between(a, b)
                };
                let raw = sweep.bits() as u64;
                if raw == 0 {
                    black_box(&output);
                    continue;
                }
                // Conservative vectoring error margin; integer ceiling, never floor.
                let upper = raw + Angle::MAX_ERROR as u64;
                let max_step = black_box(max_step) as u64;
                let mut n = upper.div_ceil(max_step);
                // i32/i64 only: reserve accumulated error in the final gap.
                while upper + 8 * n * n > n * max_step {
                    n += 1;
                }
                if n > 1 {
                    let step = (raw / n) as u32;
                    let matrix = Rotation::new(Angle::from_bits(if cw { step.wrapping_neg() } else { step }));
                    let mut v = a;
                    for _ in 1..n {
                        v = matrix.apply(v);
                        output.push(v);
                    }
                }
                count += output.len();
                black_box(&output);
            }
            count
        }),
    );
}

fn main() {
    println!("implementation,group,step_deg,median_ns_arc,min_ns_arc,max_ns_arc,points_arc");
    let mixed = cases(false);
    let short = cases(true);
    let reverse = std::env::args().any(|a| a == "reverse");
    for (group, data, steps) in [
        ("mixed", &mixed[..], &[45.0, 15.0, 5.0, 1.0, 360.0 / 1024.0][..]),
        ("short", &short[..], &[45.0, 5.0][..]),
    ] {
        for &deg in steps {
            if reverse {
                bench_cordic::<i64>(data, group, deg, "cordic_i64");
                bench_cordic::<i32>(data, group, deg, "cordic_i32");
                bench_int::<i64>(data, group, deg, "bisection_i64");
                bench_int::<i32>(data, group, deg, "bisection_i32");
            }

            bench_float::<f64>(data, group, deg, true, "f64_ceil");

            bench_float::<f32>(data, group, deg, true, "f32_ceil");
            if !reverse {
                bench_int::<i32>(data, group, deg, "bisection_i32");
                bench_int::<i64>(data, group, deg, "bisection_i64");
                bench_cordic::<i32>(data, group, deg, "cordic_i32");
                bench_cordic::<i64>(data, group, deg, "cordic_i64");
            }
        }
    }
}
