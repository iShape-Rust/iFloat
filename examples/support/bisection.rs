// Benchmark-only snapshot of iOverlay mesh/int/arc.rs on 2026-09-13.
//! Directed unit-circle arcs built by integer bisection.

use i_float::int::number::int::IntNumber;
use i_float::int::number::uint::UIntNumber;
use i_float::int::number::wide_int::WideIntNumber;
use i_float::int::unit_vector::UnitIntVector;
use i_float::int::vector::IntVector;
use std::vec::Vec;

/// Direction of traversal in Cartesian coordinates (y increases upward).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArcDirection {
    Clockwise,
    Counterclockwise,
}

/// Maximum subdivision step expressed as a squared unit-circle chord.
///
/// The integer scale is `UnitIntVector::<I>::DENOMINATOR.pow(2)`: a value
/// of one in real units represents a chord as long as the radius. This is
/// independent of the radius used later to place the arc in a mesh.
///
/// Limits correspond approximately to 1/1024 of a turn and 45 degrees.
/// Actual angular accuracy depends on [`IntVector::fast_normalize`]
/// (especially for `i16`); this is not an exact angular-error guarantee.
#[derive(Clone, Copy)]
pub struct ArcStep<I: IntNumber = i32> {
    squared_chord: I::WideUInt,
}

impl<I: IntNumber> ArcStep<I> {
    /// Clamps a squared chord in the unit-vector scale to the supported limits.
    /// Zero selects the finest subdivision; large values select the coarsest.
    pub fn from_squared_chord(squared_chord: I::WideUInt) -> Self {
        Self {
            squared_chord: squared_chord.clamp(Self::min_squared_chord(), Self::max_squared_chord()),
        }
    }

    /// Squared chord for approximately 360/1024 degrees, rounded upward.
    pub fn min_squared_chord() -> I::WideUInt {
        // ceil(4 * sin(pi / 1024)^2 * 2^60). No runtime trigonometry.
        Self::rescale(43_406_843_014_579, true)
    }

    /// Squared chord for approximately 45 degrees, rounded downward.
    pub fn max_squared_chord() -> I::WideUInt {
        // floor((2 - sqrt(2)) * 2^60).
        Self::rescale(675_365_781_047_096_175, false)
    }

    fn rescale(value: u64, round_up: bool) -> I::WideUInt {
        let bits = 2 * (I::BITS - 2);
        if bits >= 60 {
            I::WideUInt::from_u64(value) << (bits - 60)
        } else {
            let shift = 60 - bits;
            let value = if round_up {
                (value + (1u64 << shift) - 1) >> shift
            } else {
                value >> shift
            };
            I::WideUInt::from_u64(value)
        }
    }
}

impl<I: IntNumber> Default for ArcStep<I> {
    /// Selects the coarsest supported step (approximately 45 degrees).
    fn default() -> Self {
        Self {
            squared_chord: Self::max_squared_chord(),
        }
    }
}

/// Reusable storage for constructing directed arcs without trigonometry.
///
/// Output contains intermediate directions in traversal order, excluding both
/// input endpoints. The caller applies the center and radius and retains the
/// original contact points to avoid rounding seams. Equal directions describe
/// an empty arc; full turns are not supported. Opposite directions describe a
/// semicircle whose side is selected by `ArcDirection`.
///
/// Arcs are first split into spans of at most 90 degrees, then bisected at
/// `normalize(from + to)`. Each span has at most eight bisection levels, so a
/// build produces at most 1279 intermediate directions. Subdivision also stops
/// when rounding prevents a midpoint strictly inside its span. Both buffers
/// retain their capacity between builds.
///
pub struct ArcBuilder<I: IntNumber = i32> {
    stack: Vec<ArcSpan<I>>,
    directions: Vec<UnitIntVector<I>>,
}

struct ArcSpan<I: IntNumber> {
    from: UnitIntVector<I>,
    to: UnitIntVector<I>,
    depth: u8,
}

impl<I: IntNumber> ArcBuilder<I> {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            directions: Vec::new(),
        }
    }

    /// Clears the previous result and returns the intermediate directions.
    pub fn build(
        &mut self,
        from: UnitIntVector<I>,
        to: UnitIntVector<I>,
        direction: ArcDirection,
        step: ArcStep<I>,
    ) -> &[UnitIntVector<I>] {
        self.stack.clear();
        self.directions.clear();
        let end = vector(to);
        let start = vector(from);
        if start.cross_product(end) == I::Wide::ZERO && start.dot_product(end) > I::Wide::ZERO {
            return &self.directions;
        }

        let mut from = from;
        // Split at exact cardinal directions so approximate normalization
        // cannot rotate a boundary past the endpoint and select a major arc.
        // At most four cardinal boundaries precede the final span.
        for _ in 0..4 {
            let start = vector(from);
            if directed_cross(start, end, direction) >= I::Wide::ZERO
                && start.dot_product(end) >= I::Wide::ZERO
            {
                break;
            }
            let boundary = next_axis(start, direction);
            self.build_span(from, boundary, direction, step);
            from = boundary;
        }
        self.build_span(from, to, direction, step);
        // Leaves emit their end, including quarter boundaries, exactly once.
        // Only the final input endpoint is omitted from the returned buffer.
        self.directions.pop();
        &self.directions
    }

    fn build_span(
        &mut self,
        from: UnitIntVector<I>,
        to: UnitIntVector<I>,
        direction: ArcDirection,
        step: ArcStep<I>,
    ) {
        self.stack.push(ArcSpan { from, to, depth: 0 });
        while let Some(span) = self.stack.pop() {
            let a = vector(span.from);
            let b = vector(span.to);
            let chord = IntVector::<I>::new(b.x - a.x, b.y - a.y);
            if span.depth < 8 && chord.sqr_length() > step.squared_chord {
                if let Some(mid) = IntVector::<I>::new(a.x + b.x, a.y + b.y).fast_normalize() {
                    let m = vector(mid);
                    // Compare directions, not just stored components: unequal
                    // approximate lengths can still represent the same ray.
                    if directed_cross(a, m, direction) > I::Wide::ZERO
                        && directed_cross(m, b, direction) > I::Wide::ZERO
                    {
                        let depth = span.depth + 1;
                        self.stack.push(ArcSpan {
                            from: mid,
                            to: span.to,
                            depth,
                        });
                        self.stack.push(ArcSpan {
                            from: span.from,
                            to: mid,
                            depth,
                        });
                        continue;
                    }
                }
            }
            self.directions.push(span.to);
        }
    }
}

impl<I: IntNumber> Default for ArcBuilder<I> {
    fn default() -> Self {
        Self::new()
    }
}

fn vector<I: IntNumber>(unit: UnitIntVector<I>) -> IntVector<I> {
    IntVector::new(unit.x().to_wide(), unit.y().to_wide())
}

fn directed_cross<I: IntNumber>(a: IntVector<I>, b: IntVector<I>, direction: ArcDirection) -> I::Wide {
    let cross = a.cross_product(b);
    match direction {
        ArcDirection::Counterclockwise => cross,
        ArcDirection::Clockwise => -cross,
    }
}

fn next_axis<I: IntNumber>(v: IntVector<I>, direction: ArcDirection) -> UnitIntVector<I> {
    let zero = I::Wide::ZERO;
    let one = I::Wide::ONE;
    let axis = match direction {
        ArcDirection::Counterclockwise => {
            if v.x > zero && v.y >= zero {
                IntVector::new(zero, one)
            } else if v.x <= zero && v.y > zero {
                IntVector::new(-one, zero)
            } else if v.x < zero && v.y <= zero {
                IntVector::new(zero, -one)
            } else {
                IntVector::new(one, zero)
            }
        }
        ArcDirection::Clockwise => {
            if v.x >= zero && v.y > zero {
                IntVector::new(one, zero)
            } else if v.x > zero && v.y <= zero {
                IntVector::new(zero, -one)
            } else if v.x <= zero && v.y < zero {
                IntVector::new(-one, zero)
            } else {
                IntVector::new(zero, one)
            }
        }
    };
    axis.fast_normalize().expect("an axis direction is nonzero")
}
