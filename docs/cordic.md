# Integer CORDIC for arc consumers

The public implementation is `i_float::int::angle::{Angle, Rotation}`. It has
no allocations, float arithmetic, runtime trigonometry, physics dependencies,
serde support, coordinate/radius types, or arc topology. The crate remains
`no_std`.

```rust
impl Angle {
    pub const MAX_ERROR: u32 = 32;
    pub const SIN_COS_SCALE: i32 = 1 << 30;
    pub const fn from_bits(bits: u32) -> Self;
    pub const fn bits(self) -> u32;
    pub fn between<I: IntNumber>(from: UnitIntVector<I>, to: UnitIntVector<I>) -> Self;
    pub fn atan2<W: WideIntNumber>(y: W, x: W) -> Option<Self>;
    pub fn sin(self) -> i32;
    pub fn cos(self) -> i32;
    pub fn sin_cos(self) -> (i32, i32);
}
impl<I: IntNumber> Rotation<I> {
    pub fn new(angle: Angle) -> Self;
    pub fn with_precision(angle: Angle, precision: u32) -> Self;
    pub const ANGLE_MAX_ERROR: u32; // 131072 for i16; 2 for i32/i64
    pub const MAX_ERROR: u32; // ANGLE_MAX_ERROR + 3
    pub const fn angle(self) -> Angle;
    pub fn apply(self, vector: UnitIntVector<I>) -> UnitIntVector<I>;
}
```

`between(a,b)` measures counterclockwise sweep. For clockwise magnitude, use
`between(b,a)` and negate the eventual step with `wrapping_neg()`. Coincident
rays give zero even if their lengths differ. Opposite rays give exactly half a
turn. Axes are exact. Almost coincident rays retain the sign of the exact cross
product, so a tiny major arc cannot accidentally become an empty arc. There is
no dedicated full-circle API.

`src/int/angle/angle.rs` contains an executable documentation example of the future
iOverlay loop. It clears and reuses a consumer-owned `Vec`, computes the sweep
once, divides it into segments, constructs one matrix, and applies that matrix
only to intermediate directions. It excludes both input endpoints. The
consumer retains the exact original contact points, center, radius, and mesh
topology. Replacing the endpoint does not correct the measured interior errors.

The example uses a binary-angle step clamped to `[2^22, 2^29]`, exactly
`360/1024` through `45` degrees. These limits do **not** quantize the input rays.
The existing iOverlay `ArcStep` currently stores a squared chord. Integrating
this example will require adapting that representation or its conversion on
the iOverlay side; the current squared-chord constructor is not directly
accepted by this new API. No iOverlay files were modified.

The module is split into `angle.rs` (public angle and trigonometry),
`rotation.rs` (matrix storage/application), and private `cordic.rs` (shared
integer kernels and tables), re-exported by `mod.rs`. `atan2(y,x)` accepts the
full built-in wide integer range, including MIN; only `(0,0)` returns `None`.
`sin_cos` returns `(sin,cos)` in Q30, with `SIN_COS_SCALE` representing one.
Separate `sin`/`cos` calls each run the kernel; use `sin_cos` when both are needed.
`Rotation::new` reuses the full 29-iteration kernel. For approximate steps,
`Rotation::<i32>::with_precision(angle, 3)` allows angle/8 error; exponent 4 allows
angle/16. The magnitude is the shortest signed angle, so clockwise steps have
the same budget. `Rotation<I>` stores coefficients in `I`, with the same scale
as `UnitIntVector<I>`: Q14/Q30/Q62. The precision parameter changes iteration
count, including a reserve for coefficient quantization in the selected type.
Every `u32` exponent is accepted, with >=32 selecting full precision. The
matrix-angle error bound in binary units is `max(floor(|angle| / 2^precision), Rotation::<I>::MAX_ERROR)`.
Zero and cardinal rotations are exact.

`rotation.angle()` returns the achieved angle from the CORDIC residual, without
another atan2. Its error relative to the matrix angle is at most
`Rotation::<I>::ANGLE_MAX_ERROR` (131072 binary units for i16, 2 for i32/i64). Use this achieved step for counts
and remainders; tiny requests may report zero. A strict maximum-step policy
must also reserve room for rounding and bias the requested step inward.
The test `achieved_angle_drives_approximate_arc_counts` demonstrates those
margins for fresh i32 directions, both orientations, and exponents 3 and 4.

For runtime precision experiments see [the precision benchmark](cordic-precision.md).

## Numerical choices and contracts

- Public angle: unsigned 32-bit binary turn, about `1.463e-9` radians per unit.
  A 10-bit angle would lose direction information unrelated to the requested
  subdivision resolution. Even Q24 angle rounding alone could accumulate about
  12.6 coordinate units at radius 65536 over 1024 steps. Q32 keeps this term
  below 0.05 units for nearest rounding (below 0.1 for step truncation).
- `cordic::ITERATIONS` sets the full-precision count for both kernels and table
  lengths. Its value 29 is an accuracy choice: after n iterations the residual
  is below `2^-(n-1)` radians. At radius 65536 over 1024 applications, 29
  iterations contribute less than 0.25 coordinate units from the residual
  alone; 28 would give a bound of 0.5. Coefficient truncation and coordinate
  rounding add separate errors. This is not maximum attainable precision or
  a count derived from the coordinate type; increasing it requires extending
  the tables and revisiting the error contract.
- The atan table has 16 additional guard bits (48-bit binary turns internally).
  This avoids accumulating the rounding errors of a table rounded directly to
  Q32. It fits in `i64`, as do the internal rotation coordinates.
- Matrix coefficients are stored in `I`: Q14 for i16, Q30 for i32, Q62 for i64.
  The kernel produces at most Q30 precision. i16 truncates toward zero to Q14;
  i64 lifts Q30 to Q62 exactly, without gaining bits of accuracy. Coefficient
  truncation contributes at most `sqrt(2)/2^min(I::BITS-2,30)` per application.
- Gain compensation: Q60 internally, with a 128-unit inward guard. The integer
  shift error is less than `29 * sqrt(2) * 1.647 < 68` Q60 units, smaller than the
  propagated gain guard. Truncating the resulting coordinates to Q14 or Q30 cannot
  increase norm. In contrast, copying iPhysics's 128-unit **Q30** guard would
  introduce roughly 13 coordinate units of contraction after 1024 steps at
  radius 65536.
- Each matrix application truncates both output components **toward zero**.
  Thus neither the matrix nor storage rounding can increase the input length.
  There is no per-point CORDIC, square root, or normalization. All applications
  use `I::Wide`: i32 for i16, i64 for i32, i128 for i64. With S = 2^(I::BITS-2),
  each product is at most S² and the sum at most 2S², within the signed wide
  range. No coordinate-type branch or conversion through usize is needed.
- Cross/dot products are computed at full input precision. With
  `S=2^(I::BITS-2)`, each component product is at most `S²` and sums/differences
  at most `2S²`; even the conservative i64 bound `2^125` fits signed i128.
  Cross/dot are never squared. Only after preserving exact signs and axis cases
  are their magnitudes reduced to 30 significant bits, then lifted for i64
  vectoring. The conversion also fits a 32-bit host's `usize`.

`Angle::MAX_ERROR` bounds vectoring relative to the **stored** input directions,
not to an original vector before `fast_normalize`. All error bounds are for the
built-in i16/i32/i64 implementations of `IntNumber`.

`Angle` remains a 32-bit binary turn independently of `I`. Coordinate storage,
angle representation, and the requested approximation budget are separate
choices. Making only `Angle` wider would not recover precision beyond the Q30 kernel;
making it narrower would introduce angle quantization even when the consumer
requests high accuracy. `with_precision` controls construction cost without
changing these representations. The 30 retained vectoring input bits and their
guard-bit lift likewise have separate constants, unrelated to iteration count.

The supplied consumer example uses integer ceiling, with an upper sweep bound
`upper = sweep + Angle::MAX_ERROR`. A simple ceiling does not by itself account
for accumulated matrix rounding in the final gap. For fresh i32/i64 normalized
inputs and this step range, it increases `n` until
`upper + 8*n*n <= n*max_step`. Eight binary units cover step division and the
per-application angular error, which is below five units when the vector norm
is at least 0.99. Over at most 1030 applications, freshly normalized i32/i64
vectors remain above that norm. This reserves room for the final exact endpoint
as well as intermediate gaps. Tests reached 1027 segments on almost full turns.
The margin is deliberately part of the consumer example, not an arc policy in
`Angle` or `Rotation`. It does not cover i16, heavily contracted input vectors,
or angular perturbations introduced when a consumer rounds scaled mesh points.

## Accuracy measurements

Run `cargo test --lib int::angle -- --nocapture`. The tests inspect the whole
sequence, including a computed endpoint, without replacing it with the input.

Vectoring: 20,000 deterministic pairs per coordinate type, independent input
lengths, every quadrant, axes, same/opposite rays, and near-degenerate cases.
Maximum observed errors were 3.023 / 3.961 / 3.970 binary units for i16/i32/i64,
below the public conservative bound of 32 units. Extreme i64 axis and near-axis
cases exercise full-width i128 products.

Rotation: 262,193 sampled angles across the full turn. Maximum observed matrix
coefficient error was `4.652e-9`, maximum norm loss `1.3134e-9`. Cardinal
rotations are exact, including after 1024 applications.

Accumulation: 257 start directions and step values spanning the allowed range,
both orientations, every point up to 1024 applications. The large-step cases
intentionally run through multiple turns as a stress test. These are observed
maxima over a deterministic sample, not a claim of exhaustive coverage.

| Storage | Added displacement, R=1024 | Added displacement, R=65536 | Radial loss, R=65536 | Angular error, radians |
|---|---:|---:|---:|---:|
| i32 | 0.004811 | 0.307934 | 0.125796 | 4.531e-6 |
| i64 | 0.004697 | 0.300580 | 0.085662 | 4.542e-6 |

Relative to an ideal unit direction with the same initial angle, including
initial normalization contraction, observed maxima at R=65536 were 0.621145
for i32 and 0.300580 for i64. After `scale(65536)` rounds to the coordinate grid,
added displacement relative to rotation of the stored initial vector reached
0.950276 / 0.925431, respectively. Errors before grid rounding scale linearly
with radius; these endpoints cover radii 2^10 through 2^16. Initial normalization
errors depend on input magnitude and direction and can exceed this sample's
values; the existing approximate normalization contract has not been tightened.

With Q14 coefficients in `Rotation<i16>`, added displacement at R=1024 reached
**120.724** units, with angular error 0.060781 radians after 1024 applications.
Both coefficient quantization and per-step Q14 storage rounding contribute.
The earlier Q30-coefficient implementation measured 42.436 units; that result
no longer describes i16 rotation. The regression bound now sums the documented
coefficient and component-rounding errors over all applications.
This API supports i16 arithmetic but does not make fine, long i16 arcs accurate.
It preserves the length upper bound and documents accumulation explicitly on
`UnitIntVector`. No selectable normalization policy was introduced.

A separate consumer test checks 64 start angles, both traversal directions,
three maximum steps, minor/major arcs, half turns, and near coincident/opposite
rays. With the final-gap reserve above, maximum added displacement at R=65536
was 0.184918 and no angular gap exceeded the allowed maximum (test tolerance
`1e-12` radians). Before adding the reserve, a simple ceiling allowed a final-gap
excess of `3.700e-6` radians. That issue is covered by the regression test.

## Reproducible performance comparison

`examples/cordic_arc_bench.rs` adapts the provided temporary benchmark. Its
`examples/support/bisection.rs` is a benchmark-only snapshot of the current,
uncommitted iOverlay arc implementation, with imports adjusted and unused
API/docs removed. It is not another public arc implementation in iFloat.
Float uses the real `FloatNumber` methods (`libm::acos` and `libm::sincos`) and
ceiling for segment count; the old floor baseline is not used in the results.

```sh
cargo run --release --example cordic_arc_bench > results.csv
cargo run --release --example cordic_arc_bench -- reverse > results-reverse.csv
```

Mac ARM64, rustc 1.98.1, release defaults, libm 0.2.16, 2026-09-13. Each group
contains the same 2048 deterministic arcs as the supplied benchmark: mixed
sweeps 0.1–179.9 degrees or short sweeps 0.1–10 degrees, both orientations.
Input normalization is outside timing. Each algorithm uses warmed, retained
buffers and emits intermediate directions. A 60 ms warmup precedes seven
batches of about 70 ms each; results are medians in ns/arc. Algorithms were
also measured in reversed order. Timing and counts include vectoring, matrix
setup, subdivision guards, and all emitted points. Center/radius/grid placement
is outside the timing for every implementation. These are local microbenchmarks,
not integrated mesh timings; major arcs are accuracy-tested, not benchmarked.

Ranges below span the normal-order and reverse-order medians. Raw CSV files
include minima, maxima, f32 results, and short-arc groups.

| Max step | Bisection i32 | CORDIC i32 | Bisection i64 | CORDIC i64 | f64 ceil |
|---|---:|---:|---:|---:|---:|
| 45° | 32.5–32.6 | 60.5–68.5 | 84.8–85.0 | 70.0–71.5 | 8.5–8.6 |
| 15° | 148.8–149.8 | 73.8–75.5 | 378.7–380.5 | 88.3–89.6 | 13.9–14.0 |
| 5° | 495.3–499.9 | 97.3–97.5 | 1292.6–1301.3 | 133.2–135.6 | 28.5–29.1 |
| 1° | 2428.5–2432.2 | 258.3–258.7 | 6456.8–6594.7 | 397.9–405.8 | 130.0–131.6 |
| 0.351562° | 5258.2–5332.4 | 624.9–635.3 | 15025.9–15208.0 | 1020.8–1029.9 | 381.3–389.2 |

Counts must be considered alongside timing:

| Max step | Bisection points/arc | CORDIC points/arc | f64 ceil points/arc |
|---|---:|---:|---:|
| 5° | 26.191 | 17.256 | 17.256 |
| 1° | 129.973 | 88.350 | 88.342 |
| 0.351562° | 316.454 | 252.383 | 252.225 |

At 5 degrees CORDIC i32 is about 5x faster than bisection; at 1 degree about
9x faster. This improvement combines removal of per-point normalization with
fewer output points. CORDIC remains about 3.4x / 2x slower than f64 at those
steps. The conservative integer final-gap reserve accounts for the small extra
point count compared with f64 at fine steps. i64 CORDIC is about 1.4–1.6x slower
than i32 on these workloads, rather than the roughly 2.6–2.9x bisection gap.

CORDIC loses on short/coarsely subdivided i32 arcs: the 45-degree mixed case
is roughly twice as slow as bisection; short arcs needing no intermediate points
still pay for vectoring. The float baseline can reject those with one dot
comparison. No early-rejection optimization or additional public API was added
just to improve that microbenchmark.

Raw results: [normal order](cordic/results.csv),
[reverse order](cordic/results-reverse.csv).

## Source and integration notes

The worktree started at `1ff6354`, before `UnitIntVector` existed. The necessary
existing changes from the main iFloat checkout were brought forward without
commits: unsigned squared lengths (`1471fc2`, `5eada81`) and unit normalization
(`a3a24c5`). `IntPoint` squared-length/distance return types therefore also
reflect that already existing unsigned API. Unrelated later range-check and
adapter changes were not copied. The only new change to unit-vector behavior
is the explicitly documented rotation operation and its private invariant-
preserving component constructor.

The source iPhysics implementation was verified in
`src/quantity/angle.rs`: Angle(u32), 31 iterations, Q30 output, guarded gain,
exact axes, and no vectoring/atan2. Only the integer rotation approach was
adapted. Its angular velocity, physical units, float conversions, and operators
were not transferred. Vectoring is new here.

No commits, merges, or modifications of the source iPhysics/iOverlay checkouts
were performed. Integration into `outline`, `stroke`, and `variable_stroke`,
and changing iOverlay's `ArcStep` representation, remain separate work.

Initial adaptation validation completed locally, offline: 96 tests in both debug and release with
all features, four doctests, formatting, Clippy with warnings denied, rustdoc
with warnings denied, all five CI feature configurations, Rust 1.85 MSRV,
wasm32-unknown-unknown, and `cargo package --allow-dirty` including verification.
