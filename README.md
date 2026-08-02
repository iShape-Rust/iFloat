# iFloat

`i_float` provides numeric primitives for deterministic 2D geometry:

- generic integer points, vectors, and rectangles;
- wide intermediate integer arithmetic;
- conversion between floating-point and integer coordinate spaces;
- fixed-scale unit ratios for interpolation;
- basic triangle predicates;
- optional `serde` and `glam` integration.

The crate is `no_std` and supports `i16`, `i32`, and `i64` coordinate types.

## Installation

```toml
[dependencies]
i_float = "4.0"
```

The default `core` feature exposes the complete numeric and geometry API.

## Integer geometry

`IntPoint<T>` stores coordinates in `T`. Subtracting two points produces an
`IntVector<T>` whose components use the associated wide integer type: `i32`
coordinates produce an `i64` vector, while `i64` coordinates produce an
`i128` vector.

```rust
use i_float::int::point::IntPoint;
use i_float::triangle::Triangle;

let a = IntPoint::new(0_i32, 0);
let b = IntPoint::new(10, 0);
let c = IntPoint::new(0, 10);

let ab = b - a;
assert_eq!(ab.x, 10_i64);
assert_eq!(ab.y, 0_i64);

assert_eq!(Triangle::area_two(a, b, c), 100_i64);
assert!(!Triangle::is_clockwise(a, b, c));
```

### Coordinate range

Integer geometry intentionally uses a coordinate range narrower than the full
range of the underlying integer. Although point differences are widened, dot
products, cross products, and squared lengths multiply wide values without
widening them again.

A conservative common bound for all point and vector operations is:

```text
-2^(I::BITS - 2) < coordinate < 2^(I::BITS - 2)
```

For example, `i32` coordinates should stay strictly between
`-1_073_741_824` and `1_073_741_824`. This leaves enough headroom for the
difference of two points and for the sum or difference of two products in
`I::Wide`. Operations use normal integer arithmetic and do not perform runtime
range checks.

This bound is deliberately universal and conservative. An algorithm may use a
wider range when it proves that its particular intermediate expressions still
fit. Conversely, an `IntVector` constructed directly from arbitrary wide values
is not covered by the point-coordinate bound.

Floating-point input should normally be mapped with `FloatPointAdapter`. For an
explicit general-purpose safety margin, use `with_coordinate_bits` with at most
`I::BITS - 3`; algorithms with stronger range analysis may select a larger bit
budget.

## Floating-point adapter

`FloatPointAdapter` maps a bounded floating-point coordinate space onto an
integer grid. The same adapter converts results back into the original space.

```rust
use i_float::adapter::FloatPointAdapter;
use i_float::float::rect::FloatRect;
use i_float::int::point::IntPoint;

let bounds = FloatRect::new(-10.0_f64, 10.0, -5.0, 5.0);
let adapter = FloatPointAdapter::<[f64; 2], i32>::new(bounds);

let source = [2.5, -1.25];
let point: IntPoint<i32> = adapter.try_float_to_int(&source).unwrap();
let restored = adapter.try_int_to_float(&point).unwrap();

let tolerance = adapter.inv_scale();
assert!((restored[0] - source[0]).abs() <= tolerance);
assert!((restored[1] - source[1]).abs() <= tolerance);
```

Use `with_coordinate_bits` when an algorithm has an explicit coordinate-bit
budget. The value controls only the converted coordinate magnitude; it does not
prove that every later arithmetic expression is safe. Use `try_with_scale` or
`try_with_scale_and_coordinate_bits` when a caller supplies the scale and
invalid or unsafe scales must be rejected.

## Fixed-scale ratios

`UnitRatio<I>` represents a value in the inclusive range `0..=1`. Its stored
integer value uses `FixedScale<I>::DENOMINATOR` as one. Scaling rounds midpoint
values away from zero.

```rust
use i_float::int::number::unit_ratio::UnitRatio;
use i_float::int::point::IntPoint;

let quarter = UnitRatio::<i32>::from_int(1, 4);
let half = UnitRatio::<i32>::half();

assert_eq!(quarter.scale(10), 3);
assert_eq!(quarter.scale(-10), -3);
assert_eq!(quarter.mid(half), UnitRatio::from_int(3, 8));

let point = IntPoint::new(100, -40);
assert_eq!(quarter.scale_point(point), IntPoint::new(25, -10));
```

Constructors currently expect valid input. In particular, `new` expects a
stored value between zero and `DENOMINATOR`, `from_float` expects a finite value
between zero and one, and `from_int` expects `0 <= numerator <= denominator`.
These preconditions are checked by debug assertions.

## Features

| Feature | Default | Description |
| --- | --- | --- |
| `core` | yes | Integer and floating-point primitives, adapters, and triangle predicates |
| `serde` | no | Enables serialization for supported geometry types and also enables `core` |
| `glam` | no | Adds conversions for `glam::Vec2`, `DVec2`, and `IVec2` and also enables `core` |

Example with optional integration:

```toml
[dependencies]
i_float = { version = "4.0", features = ["serde", "glam"] }
```

## License

Licensed under the MIT License. See the `LICENSE` file.
