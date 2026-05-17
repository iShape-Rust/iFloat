use crate::float::compatible::FloatPointCompatible;
use crate::float::number::FloatNumber;
use crate::float::rect::FloatRect;
use crate::int::number::int::IntNumber;
use crate::int::number::wide_int::WideIntNumber;
use crate::int::point::IntPoint;
use core::marker::PhantomData;

#[derive(Clone)]
pub struct FloatPointAdapter<P: FloatPointCompatible, I: IntNumber> {
    pub dir_scale: P::Scalar,
    pub inv_scale: P::Scalar,
    pub offset: P,
    pub rect: FloatRect<P::Scalar>,
    int: PhantomData<I>,
}

impl<P: FloatPointCompatible, I: IntNumber> FloatPointAdapter<P, I> {
    const SCALE_SAFETY_BITS: i32 = 3;

    #[inline]
    pub fn new(rect: FloatRect<P::Scalar>) -> Self {
        let a = rect.width() * FloatNumber::from_float(0.5);
        let b = rect.height() * FloatNumber::from_float(0.5);

        let x = rect.min_x + a;
        let y = rect.min_y + b;

        let offset = P::from_xy(x, y);

        let max = a.max(b);

        // degenerate case
        if max == FloatNumber::from_float(0.0) {
            return Self {
                dir_scale: FloatNumber::from_float(1.0),
                inv_scale: FloatNumber::from_float(1.0),
                offset,
                rect,
                int: PhantomData,
            };
        }

        let log2 = max.log2().to_i32();
        let safe_bits = I::BITS as i32 - Self::SCALE_SAFETY_BITS;
        let ie = safe_bits - log2;
        let e = ie as f64;

        let dir_scale = FloatNumber::from_float(libm::exp2(e));
        let inv_scale = FloatNumber::from_float(libm::exp2(-e));

        Self {
            dir_scale,
            inv_scale,
            offset,
            rect,
            int: PhantomData,
        }
    }

    #[inline]
    pub fn with_scale(rect: FloatRect<P::Scalar>, scale: f64) -> Self {
        let a = rect.width() * FloatNumber::from_float(0.5);
        let b = rect.height() * FloatNumber::from_float(0.5);

        let x = rect.min_x + a;
        let y = rect.min_y + b;

        let offset = P::from_xy(x, y);

        let max = a.max(b);

        // degenerate case
        if max == FloatNumber::from_float(0.0) {
            return Self {
                dir_scale: FloatNumber::from_float(1.0),
                inv_scale: FloatNumber::from_float(1.0),
                offset,
                rect,
                int: PhantomData,
            };
        }

        let dir_scale = FloatNumber::from_float(scale);
        let inv_scale = FloatNumber::from_float(1.0 / scale);

        Self {
            dir_scale,
            inv_scale,
            offset,
            rect,
            int: PhantomData,
        }
    }

    #[inline]
    pub fn with_iter<'a, Q>(iter: Q) -> Self
    where
        Q: Iterator<Item = &'a P>,
        P: 'a,
    {
        Self::new(FloatRect::with_iter(iter).unwrap_or(FloatRect::zero()))
    }

    #[inline(always)]
    pub fn int_to_float(&self, point: &IntPoint<I>) -> P {
        let fx: P::Scalar = FloatNumber::from_int(point.x);
        let fy: P::Scalar = FloatNumber::from_int(point.y);
        let x = fx * self.inv_scale + self.offset.x();
        let y = fy * self.inv_scale + self.offset.y();
        let float = P::from_xy(x, y);

        if cfg!(debug_assertions) {
            let radius = self.rect.height().max(self.rect.width()) * P::Scalar::from_float(0.01);
            if !self.rect.contains_with_radius(&float, radius) {
                panic!(
                    "You are trying to convert a point[{}, {}] which is out of rect: {}",
                    x, y, self.rect
                );
            }
        }

        float
    }

    #[inline(always)]
    pub fn float_to_int(&self, point: &P) -> IntPoint<I> {
        if cfg!(debug_assertions) {
            let radius = self.rect.height().max(self.rect.width()) * P::Scalar::from_float(0.01);
            if !self.rect.contains_with_radius(point, radius) {
                panic!(
                    "You are trying to convert a point[{}, {}] which is out of rect: {}",
                    point.x(),
                    point.y(),
                    self.rect
                );
            }
        }

        let sx = (point.x() - self.offset.x()) * self.dir_scale;
        let sy = (point.y() - self.offset.y()) * self.dir_scale;

        let x = I::from_float_round(sx);
        let y = I::from_float_round(sy);
        IntPoint { x, y }
    }

    #[inline(always)]
    pub fn sqr_float_to_int(&self, value: P::Scalar) -> I::Wide {
        let scale = self.dir_scale;
        let sqr_scale = scale * scale;
        I::Wide::from_float_round(sqr_scale * value)
    }

    #[inline(always)]
    pub fn len_float_to_int(&self, value: P::Scalar) -> I {
        I::from_float_round(self.dir_scale * value)
    }
}

#[cfg(test)]
mod tests {
    use crate::adapter::FloatPointAdapter;
    use crate::float::compatible::FloatPointCompatible;
    use crate::float::point::FloatPoint;
    use crate::float::rect::FloatRect;
    use crate::int::point::IntPoint;

    #[test]
    fn test_0() {
        let rect = FloatRect {
            min_x: 1.0,
            max_x: 1.0,
            min_y: -2.0,
            max_y: -2.0,
        };

        let adapter = FloatPointAdapter::<FloatPoint<f64>, i32>::new(rect);

        assert_eq!(adapter.dir_scale, 1.0);
        assert_eq!(adapter.inv_scale, 1.0);
    }

    #[test]
    fn test_1() {
        let rect = FloatRect {
            min_x: 0.0,
            max_x: 10.0,
            min_y: 0.0,
            max_y: 100.0,
        };

        let adapter = FloatPointAdapter::<[f64; 2], i32>::new(rect);

        let f0 = [10.0, 2.0];
        let p0 = adapter.float_to_int(&f0);
        let f1: [f64; 2] = adapter.int_to_float(&p0);

        assert_eq!((f0.x() - f1.x()).abs() < 0.000_0001, true);
        assert_eq!((f0.y() - f1.y()).abs() < 0.000_0001, true);
    }

    #[test]
    fn test_2() {
        let points = [[-2.0, -4.0], [-2.0, 3.0], [5.0, 3.0], [5.0, -4.0]];

        let adapter = FloatPointAdapter::<[f64; 2], i32>::with_iter(points.iter());

        let f0 = [1.0, 2.0];
        let p0 = adapter.float_to_int(&f0);
        let f1: [f64; 2] = adapter.int_to_float(&p0);

        assert_eq!((f0.x() - f1.x()).abs() < 0.000_0001, true);
        assert_eq!((f0.y() - f1.y()).abs() < 0.000_0001, true);
    }

    #[test]
    fn test_i64_scale() {
        let rect = FloatRect {
            min_x: -1000.0,
            max_x: 1000.0,
            min_y: -1000.0,
            max_y: 1000.0,
        };

        let adapter = FloatPointAdapter::<[f64; 2], i64>::new(rect);
        let p = adapter.float_to_int(&[1.25, -2.5]);
        let f: [f64; 2] = adapter.int_to_float(&p);

        assert_eq!(p.x.abs() > i32::MAX as i64, true);
        assert_eq!((f.x() - 1.25).abs() < 0.000_0001, true);
        assert_eq!((f.y() + 2.5).abs() < 0.000_0001, true);
    }

    #[test]
    fn test_round_point_round_length() {
        let adapter =
            FloatPointAdapter::<[f64; 2], i32>::with_scale(FloatRect::new(-1.0, 1.0, -1.0, 1.0), 10.0);

        assert_eq!(adapter.float_to_int(&[0.16, -0.16]), IntPoint::new(2, -2));
        assert_eq!(adapter.len_float_to_int(0.16), 2);
    }
}
