//! Shared integer CORDIC kernel. Public coefficients stay Q30; approximate
//! Rotation construction selects iterations from a relative angular budget.
// Accuracy budget, independent of coordinate storage: n rotations leave a
// residual below 2^-(n-1) radians. At n=29, 1024 steps at radius 2^16
// contribute <0.25 coordinate units from that residual alone (n=28: <0.5).
// Q30 coefficient truncation and application rounding add separate errors.
pub(crate) const ITERATIONS: usize = 29;
// Vectoring retains 30 significant input bits, then lifts the top bit to 58.
// These widths are independent of the iteration count and fit 32-bit usize
// conversion and i64 CORDIC growth, respectively.
pub(crate) const VECTOR_INPUT_BITS: u32 = 30;
pub(crate) const VECTOR_GUARD_BITS: u32 = 59 - VECTOR_INPUT_BITS;
pub(crate) const COEFFICIENT_BITS: u32 = 30;
// atan(2^-i), 48-bit binary turns (16 guard bits beyond the public angle).
const ATAN: [i64; ITERATIONS] = [
    35184372088832,
    20770547670515,
    10974586953444,
    5570871696862,
    2796246208089,
    1399486241028,
    699913886760,
    349978300884,
    174991820497,
    87496244017,
    43748163730,
    21874087080,
    10937044192,
    5468522177,
    2734261099,
    1367130551,
    683565276,
    341782638,
    170891319,
    85445659,
    42722830,
    21361415,
    10680707,
    5340354,
    2670177,
    1335088,
    667544,
    333772,
    166886,
];

// floor(2^60 / product(sqrt(1 + 2^-2i), i=0..n-1)) - 128.
// A gain for each iteration count avoids conflating early termination with
// using the wrong gain. The inward guard dominates shift rounding (<68 units).
const GAINS: [i64; ITERATIONS] = [
    815238614083298760,
    729171583589189357,
    707400343138147019,
    701937710475640438,
    700570741874588230,
    700228916656934686,
    700143455142409185,
    700122089437857531,
    700116747991345093,
    700115412628443506,
    700115078787638515,
    700114995327432293,
    700114974462380426,
    700114969246117440,
    700114967942051693,
    700114967616035256,
    700114967534531146,
    700114967514155119,
    700114967509061112,
    700114967507787611,
    700114967507469235,
    700114967507389641,
    700114967507369743,
    700114967507364768,
    700114967507363525,
    700114967507363214,
    700114967507363136,
    700114967507363116,
    700114967507363112,
];

/// Smallest iteration count covering |signed angle| / 2^precision, down to
/// the full-precision floor. The rounding budget covers coefficient and
/// table rounding in the selected output format. Any exponent is valid, including >= 32 (full precision).
#[inline]
pub(crate) fn rotation_iterations(bits: u32, precision: u32, rounding_budget: u32) -> usize {
    let magnitude = (bits as i32).unsigned_abs();
    let budget = magnitude.checked_shr(precision).unwrap_or(0);
    if budget <= rounding_budget {
        return ITERATIONS;
    }
    let residual_budget = budget - rounding_budget;
    if residual_budget >= 683_565_276 {
        return 1;
    }
    // The residual bound is ceil((2^32/tau) / 2^(n-1)). Start from the next
    // power of two and check one fewer iteration: no division or float log.
    let mut n = 31 - residual_budget.ilog2();
    let shift = n - 2;
    let previous_bound = (683_565_276u32 + (1 << shift) - 1) >> shift;
    if previous_bound <= residual_budget {
        n -= 1;
    }
    (n as usize).min(ITERATIONS)
}

/// First-quadrant vectoring, positive coordinates scaled to top bit 58.
/// The caller handles exact axes and folds the result into the original quadrant.
#[inline]
pub(crate) fn vectoring(mut x: i64, mut y: i64, iterations: usize) -> i64 {
    debug_assert!((1..=ITERATIONS).contains(&iterations));
    let mut z = 0i64;
    for (shift, angle) in ATAN[..iterations].iter().enumerate() {
        let old_x = x;
        if y > 0 {
            x += y >> shift;
            y -= old_x >> shift;
            z += angle;
        } else if y < 0 {
            x -= y >> shift;
            y += old_x >> shift;
            z -= angle;
        }
    }
    (z + (1 << 15)) >> 16
}

#[inline]
pub(crate) fn sin_cos(bits: u32, iterations: usize, coefficient_bits: u32) -> (i32, i32) {
    let (sin, cos, _) = rotation(bits, iterations, coefficient_bits);
    (sin, cos)
}

/// Returns sine, cosine and achieved binary angle from the final residual.
/// The angle precedes coefficient quantization; the Q30 angular discrepancy,
/// including rounding the angle to 32 bits, is bounded by two binary units.
#[inline]
pub(crate) fn rotation(bits: u32, iterations: usize, coefficient_bits: u32) -> (i32, i32, u32) {
    debug_assert!((1..=ITERATIONS).contains(&iterations));
    debug_assert!((14..=COEFFICIENT_BITS).contains(&coefficient_bits));
    let one = 1 << coefficient_bits;
    match bits {
        0 => return (0, one, bits),
        0x4000_0000 => return (one, 0, bits),
        0x8000_0000 => return (0, -one, bits),
        0xc000_0000 => return (-one, 0, bits),
        _ => {}
    }
    let mut z = (bits as i32 as i64) << 16;
    let mut sign = 1;
    if z > 1 << 46 {
        z -= 1 << 47;
        sign = -1;
    } else if z < -(1 << 46) {
        z += 1 << 47;
        sign = -1;
    }
    let (mut x, mut y) = (GAINS[iterations - 1], 0i64);
    for (shift, angle) in ATAN[..iterations].iter().enumerate() {
        let old_x = x;
        if z >= 0 {
            x -= y >> shift;
            y += old_x >> shift;
            z -= angle;
        } else {
            x += y >> shift;
            y -= old_x >> shift;
            z += angle;
        }
    }
    let scale = 1i64 << (60 - coefficient_bits);
    let (sin, cos) = ((sign * (y / scale)) as i32, (sign * (x / scale)) as i32);
    debug_assert!(sin as i64 * sin as i64 + cos as i64 * cos as i64 <= 1i64 << (2 * coefficient_bits));
    // Keep the original unsigned angle here: folding above only changes the
    // CORDIC convergence interval. Casting back to u32 wraps across zero.
    let achieved = ((((bits as i64) << 16) - z + (1 << 15)) >> 16) as u32;
    (sin, cos, achieved)
}
