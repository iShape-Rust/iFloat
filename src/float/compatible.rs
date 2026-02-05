use crate::float::number::FloatNumber;

pub trait FloatPointCompatible<T: FloatNumber>
where
    Self: Copy,
{
    fn from_xy(x: T, y: T) -> Self;
    fn x(&self) -> T;
    fn y(&self) -> T;
}

impl<T: FloatNumber> FloatPointCompatible<T> for [T; 2] {
    #[inline(always)]
    fn from_xy(x: T, y: T) -> Self {
        [x, y]
    }

    #[inline(always)]
    fn x(&self) -> T {
        self[0]
    }

    #[inline(always)]
    fn y(&self) -> T {
        self[1]
    }
}

#[cfg(test)]
mod tests {
    use crate::float::compatible::FloatPointCompatible;

    #[test]
    fn test_0() {
        let a0 = [2.0, 5.0];
        let x = a0.x();
        let y = a0.y();
        let a1 = <[f64; 2]>::from_xy(x, y);

        assert_eq!(a0, a1);
    }
}
