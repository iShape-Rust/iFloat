# Runtime precision experiment

## Relative error of the rotation step

```sh
cargo run --release --example cordic_precision -- --relative 16 --step 20
cargo run --release --example cordic_precision -- --relative 8 --step 20
```

`--relative 16` targets angular error no greater than the requested rotation
angle divided by 16 (6.25%); `--relative 8` allows 12.5%. Iterations are chosen
for **each step angle**, rather than setting a single count for every arc.
This mode keeps vectoring/atan2 at the public 29-iteration precision and matrix
coefficients at Q30. Only rotation CORDIC is shortened. It cannot be combined
with explicit `--iterations` or `--q`.

The selector uses the CORDIC residual bound `2^-(n-1)` radians, converted to
binary angle units, and reserves two binary units for coefficient/table
rounding. It finds the smallest sufficient iteration count with shifts,
leading-bit inspection, and one integer comparison; no runtime logarithm,
float arithmetic, or division is needed. Clockwise angles use their unsigned
magnitude. For extremely tiny angles whose budget is below the fixed-precision
floor, it falls back to 29 iterations without claiming the relative bound.
That floor is outside this experiment's arc-step range.

| Requested step | Iterations, step/8 | Iterations, step/16 |
|---|---:|---:|
| 45 degrees | 5 | 6 |
| 20 degrees | 6 | 7 |
| 5 degrees | 8 | 9 |
| 1 degree | 10 | 11 |
| 360/1024 degrees | 12 | 13 |

This is a tolerance on **one matrix's angle**, not a bound on the phase error
of all generated arc points. Repeating a matrix repeats its angular error with
the same sign. With fixed reference counts, after `n` steps the phase error
is approximately `n * step_error`, even though radial drift stays small in Q30.
The CSV deliberately reports this effect instead of replacing the last computed
point. The 1024-step stress test can cover many full turns for coarse steps;
its displacement is not a typical error for a short 20-degree-step arc.

For this i32 experiment, the public API is `Rotation::<i32>::with_precision(angle, precision)`, where the
parameter is the exponent: 3 means angle/8 and 4 means angle/16. It shares this
experiment's iteration selector. All `u32` exponents are accepted; >=32 uses
full precision. The conservative matrix-angle error bound in binary units is
`max(floor(|angle| / 2^precision), 5)`.

The consumer obtains the **achieved angle** with `rotation.angle()` and uses
it for the number of rotations and the final remainder. CORDIC already tracks
the residual, so another atan2 is unnecessary. The returned angle differs from
the Q30 matrix angle by at most `Rotation::<i32>::ANGLE_MAX_ERROR` (2 binary units).
Tiny requests can report zero. A strict maximum-step policy also needs to
budget for rounding and bias the requested step inward; a consumer regression
test demonstrates this for i32 storage and exponents 3/4. Public
`Rotation::new` retains all 29 iterations; iOverlay is unchanged.

## Explicit iteration and coefficient controls

From the iFloat checkout:

```sh
cargo run --release --example cordic_precision -- --iterations 24 --q 30 --step 1 --radius 65536
cargo run --release --example cordic_precision -- --sweep > precision.csv
cargo run --release --example cordic_precision -- --sweep --reverse > precision-reverse.csv
cargo run --release --example cordic_precision -- --help
```

`--iterations` is a runtime integer in 1..29; it changes both vectoring and
rotation. `--q` is a runtime coefficient precision in 14..30. Neither requires
recompilation; after the initial build the executable in `target/release/examples/`
can be invoked directly with different values. The default is 29 iterations,
Q30, step 1 degree, radius 65536. Step accepts 0.3515625..45 degrees, radius
accepts integers 1..2^30. `--sweep` compares twelve profiles, varying iterations and
coefficient bits independently and together, plus two relative-error profiles; it cannot be combined with explicit
`--iterations`, `--q`, or `--relative`. Invalid options exit with status 2. `--reverse` reverses
the measurement order, including the fixed-library reference.

The example includes `src/int/angle/cordic.rs` by source path. The measured
CORDIC loops and lookup tables are the same ones used by the library, with a
separate precomputed gain for each iteration count. Public `Angle` operations
and `Rotation::<i32>::new` retain constant 29/Q30 settings. `Rotation::<i32>::with_precision`
selects iterations while retaining Q30 coefficients. There is no user-selectable
normalization algorithm. `Angle::sin`, `cos`, `sin_cos`, and `atan2` expose the
fixed-precision operations themselves.

The `library_fixed` reference calls the public `Angle::atan2` and `sin_cos`.
The `runtime` profiles pass settings through `black_box`, allowing the CSV to
show runtime-loop overhead separately from precision changes. The 29/Q30 runtime
profile is tested to match the public library bit for bit. The experiment's
matrix application is also checked against `Rotation::apply`.

All vectors retain i32 Q30 storage. Coefficients quantized to Q14..Q30 are lifted
back to Q30 **without recovering lost bits** before multiplication. This gives
every profile the same i64 multiplication and constant division in the point
loop, isolating coefficient accuracy from runtime division and storage changes.
The library also has `Rotation<i16>` with Q14 coefficients and `Rotation<i64>`
with coefficients lifted exactly from Q30 to Q62. This experiment does not
benchmark i16/Q14 vector storage, SIMD, or
narrow multiply implementation. The computational kernel remains integer-only;
float references and reporting run outside the measured regions.

## What the CSV measures

- `setup_median_ns`: exact cross/dot, vectoring, step division, rotation CORDIC,
  coefficient conversion. One setup for every input arc, including short arcs.
- `arc_median_ns`, `arc_min_ns`, `arc_max_ns`: setup plus matrix applications
  and writing intermediate points into a warmed reusable buffer. Arcs with no
  intermediate points skip setup using the precomputed reference segment count.
- `points_arc`: identical work across all configurations; `amortized_ns_point`
  includes setup and is not a pure matrix-operation latency.
- `rotation_median_ns`: matrix coefficient construction alone, including adaptive
  iteration selection when enabled, with step angles prepared outside timing.
- `api_rotation_median_ns`: public `Rotation::new` or `with_precision` construction,
  including the achieved-angle result. Empty for explicit runtime profiles that
  do not correspond to a public constructor. Older CSV snapshots lack this column.
- `rotation_iterations_min/max/mean`: iteration counts actually chosen for
  rotation across the inputs, including short arcs in the setup measurement.
- `rotation_max_relative_error`: largest absolute matrix angle error divided
  by its requested step angle, not accumulated arc error.
- `atan_max_error_bits`: maximum vectoring error, in 32-bit binary angle units.
- `arc_max_error_units`: maximum displacement over all computed arc points,
  **including a computed endpoint**, relative to the ideal rotation of the
  stored initial vector. It includes vectoring and matrix errors.
- `drift_1024_max_error_units` and `radial_1024_max_loss_units`: accumulated
  displacement and radial loss over 1024 rotations, independently of vectoring.
- `max_step_excess_deg`, `nonmonotone_gaps`: observed step-limit excess and
  wrong-order/zero angular gaps, including the gap to the exact input endpoint.

Timing uses 2048 deterministic mixed arcs (0.1..179.9 degrees, both directions).
Input normalization and reference segment counts are outside timing. Counts
are computed by ceiling the accurate reference sweep divided by the requested
maximum step, and are held fixed for every profile. This prevents a low-precision
profile from appearing faster just because it underestimated the sweep and
emitted fewer points. It is a **controlled comparison**, not an end-to-end
measurement of adaptive subdivision. The existing `cordic_arc_bench` example
continues to measure subdivision as part of the build.

Experimental profiles have **no maximum-step guarantee**: the usual final-gap
reserve is intentionally omitted to keep point counts identical. Violations
are reported, not hidden. The production example and its strict-gap tests still
include their reserve. Accuracy is relative to stored normalized inputs, before
integer grid rounding; initial normalization error is additional. The 1024-step
stress test covers 257 start/step combinations over the supported step range,
both orientations, every intermediate result, with no endpoint replacement.

Timing: 40 ms warmup, seven batches of approximately 50 ms, median and range.
Errors are deterministic sample maxima, not exhaustive bounds. `--radius` scales
reported errors; it does not change the timed direction-generation workload.

## Sample results

The following explicit-profile measurements precede the relative-mode addition.
Mac ARM64, rustc 1.98.1, release defaults, step 1 degree, radius 65536. Each arc
has 88.342 intermediate points on average, for every profile. Ranges below span
the normal-order and reversed-order medians; see the raw CSVs for each run's
minimum and maximum. Values are local measurements, not portable speed promises.

| Mode | Iterations | Q | Setup ns | Arc ns | Arc error | Error after 1024 |
|---|---:|---:|---:|---:|---:|---:|
| library_fixed | 29 | 30 | 63.5–65.8 | 250.1–252.9 | 0.065 | 0.308 |
| runtime | 29 | 30 | 68.1–71.3 | 255.2–255.3 | 0.065 | 0.308 |
| runtime | 24 | 30 | 50.9–52.8 | 239.7–241.4 | 1.359 | 7.967 |
| runtime | 20 | 30 | 38.7–41.9 | 231.8–232.8 | 20.994 | 127.902 |
| runtime | 15 | 30 | 26.6–26.8 | 224.0–224.3 | 699.105 | 4068.959 |
| runtime | 29 | 24 | 69.1–71.1 | 253.7–256.4 | 0.918 | 5.573 |
| runtime | 29 | 20 | 67.1–71.0 | 255.4–256.7 | 14.737 | 87.067 |
| runtime | 29 | 14 | 68.2–70.7 | 257.3–257.5 | 797.476 | 5208.792 |
| runtime | 24 | 24 | 49.5–52.8 | 238.6–240.7 | 2.054 | 11.169 |
| runtime | 20 | 20 | 38.2–38.3 | 232.7–233.8 | 25.246 | 177.744 |
| runtime | 15 | 14 | 27.0–27.3 | 222.1–224.2 | 1394.052 | 7545.789 |

Reducing iteration count speeds up setup, but point generation dominates long
arcs. Lowering only coefficient precision barely changes speed in this storage
model and severely increases radial/angular drift. A dynamic 29/Q30 configuration
also costs more setup time than the constant public implementation; that overhead
should not be mistaken for an effect of numerical accuracy.

Raw runs: [normal order](cordic/precision.csv),
[reverse order](cordic/precision-reverse.csv).

Tests check public atan2 across full signed integer ranges (including i128::MIN),
public sin/cos consistency, runtime/default equivalence, invalid CLI arguments,
and coefficient non-expansion for every supported iteration/Q combination over
4097 full-turn samples. Existing accuracy and strict consumer-gap tests remain.

## Relative-mode measurements

Same Mac ARM64, release, 2048 mixed arcs, radius 65536, maximum step 20 degrees.
Counts are held at 3.934 interior points per arc. Ranges span the two run-order
medians, not different input sets.

| Mode | Matrix setup ns | Full setup ns | Arc ns | Maximum single-step relative error |
|---|---:|---:|---:|---:|
| Fixed 29/Q30 | 21.67–21.68 | 62.25–65.79 | 55.65–58.65 | 0.000031% |
| Rotation step/16 | 6.37–6.39 | 50.05–50.58 | 44.48–45.11 | 5.880032% |
| Rotation step/8 | 5.61–5.63 | 46.43–52.48 | 39.22–43.04 | 11.811171% |

At a nominal 20-degree step, the matrix setup alone is about 3–4x faster.
Full-arc speedup is smaller because atan2 stays accurate and output work remains.
Only the rotation iteration count changes; coefficient norm remains conservative
Q30 and radial drift after 1024 applications is still about 0.11 units at R65536.

The unchanged-reference-count experiment also exposes why subdivision must use
the achieved matrix angle: at max step 20 and divisor 16, the worst final gap
exceeds the requested step by 6.991 degrees. At max step 1 and divisor 16,
1640 of 2048 arcs have a negative final remainder because the repeated rotations
have passed the input endpoint. This is systematic angular drift, not radial
contraction or vectoring error. Those configurations are a speed/accuracy
experiment, not a valid replacement arc builder with the current count policy.

Raw runs: [step 20, divisor 16](cordic/relative16-20.csv),
[reversed](cordic/relative16-20-reverse.csv),
[step 20, divisor 8](cordic/relative8-20.csv),
[reversed](cordic/relative8-20-reverse.csv),
[step 1, divisor 16](cordic/relative16-1.csv).

Additional tests verify the minimal sufficient iteration count, clockwise/CCW
symmetry, and the single-step angular budget on 65,536 angles per divisor and
five arc-step workloads. Atan2 is checked to retain its original accuracy.

## Public constructor measurement

After adding `with_precision` and the achieved-angle field, the same step-20
workload measured public construction at 21.75–21.81 ns for `Rotation::new`,
7.45 ns for exponent 4, and 6.62 ns for exponent 3. These are local medians,
including the achieved-angle result; the old coefficient-only column remains
separate. The full-arc timing still uses fixed reference counts and is not a
validated approximate subdivision implementation.

Raw runs: [exponent 4](cordic/rotation-api16-20.csv),
[exponent 3](cordic/rotation-api8-20.csv).
