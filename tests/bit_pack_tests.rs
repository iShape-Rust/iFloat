#[cfg(test)]
mod tests {
    use i_float::bit_pack::{BitPackFix, BitPackVec};
    use i_float::fix_float::{FIX_MAX, FIX_MIN};
    use i_float::fix_vec::FixVec;

    #[test]
    fn test_0() {
        assert!(FixVec::new(0, 0).bit_pack() < FixVec::new(0, 1).bit_pack());
        assert!(FixVec::new(0, 0).bit_pack() > FixVec::new(0, -1).bit_pack());
    }

    #[test]
    fn test_1() {
        assert!(FixVec::new(0, 0).bit_pack() < FixVec::new(0, FIX_MAX).bit_pack());
        assert!(FixVec::new(0, 0).bit_pack() > FixVec::new(0, FIX_MIN).bit_pack());
    }

    #[test]
    fn test_2() {
        assert!(FixVec::new(FIX_MAX, 0).bit_pack() < FixVec::new(FIX_MAX, 1).bit_pack());
        assert!(FixVec::new(FIX_MAX, 0).bit_pack() > FixVec::new(FIX_MAX, -1).bit_pack());
    }

    #[test]
    fn test_3() {
        assert!(FixVec::new(FIX_MAX, 0).bit_pack() < FixVec::new(FIX_MAX, FIX_MAX).bit_pack());
        assert!(FixVec::new(FIX_MAX, 0).bit_pack() > FixVec::new(FIX_MAX, FIX_MIN).bit_pack());
    }

    #[test]
    fn test_4() {
        assert!(FixVec::new(FIX_MIN, 0).bit_pack() < FixVec::new(FIX_MIN, 1).bit_pack());

        let b0 = FixVec::new(FIX_MIN, 0).bit_pack();
        let b1 = FixVec::new(FIX_MIN, -1).bit_pack();

        assert!(b0 > b1)
    }

    #[test]
    fn test_5() {
        assert!(FixVec::new(FIX_MIN, 0).bit_pack() < FixVec::new(FIX_MIN, FIX_MAX).bit_pack());
        assert!(FixVec::new(FIX_MIN, 0).bit_pack() > FixVec::new(FIX_MIN, FIX_MIN).bit_pack());
    }

    #[test]
    fn test_6() {
        assert_eq!(FixVec::new(FIX_MAX, FIX_MAX).bit_pack(), u64::MAX);
        assert_eq!(FixVec::new(FIX_MIN, FIX_MIN).bit_pack(), 0);
    }

    #[test]
    fn test_7() {
        let p = FixVec::new(-10, 10);
        let b = p.bit_pack();
        let v = b.fix_vec();

        assert_eq!(p, v);
    }

    #[test]
    fn test_8() {
        let p = FixVec::new(10, -10);
        let b = p.bit_pack();
        let v = b.fix_vec();

        assert_eq!(p, v);
    }

    #[test]
    fn test_9() {
        assert_eq!(FixVec::new(FIX_MIN + 1, FIX_MIN + 1).bit_pack().fix_vec(), FixVec::new(FIX_MIN + 1, FIX_MIN + 1));
        assert_eq!(FixVec::new(FIX_MAX - 1, FIX_MAX - 1).bit_pack().fix_vec(), FixVec::new(FIX_MAX - 1, FIX_MAX - 1));
    }

    #[test]
    fn test_10() {
        for i in 1..32 { // Iterate over the bit width of i32
            let v = (1i64 << i) - 1;
            let a = FixVec::new(v, -v);
            let b = FixVec::new(-v, v);
            assert_eq!(a.bit_pack().fix_vec(), a);
            assert_eq!(b.bit_pack().fix_vec(), b);
        }
    }

    #[test]
    fn test_11() {
        let mut b = FixVec::new(FIX_MIN, FIX_MIN);
        for x in i8::MIN..=i8::MAX {
            for y in i8::MIN..=i8::MAX {
                let a = FixVec::new(x as i64, y as i64);
                assert!(b.bit_pack() < a.bit_pack());
                assert_eq!(a.bit_pack().fix_vec(), a);
                b = a;
            }
        }
    }

    #[test]
    fn test_random() {
        let min = i32::MIN;
        let max = i32::MAX;

        for _ in 0..100_000 {
            let x = rand::random::<i32>().clamp(min, max) as i64;
            let y = rand::random::<i32>().clamp(min, max) as i64;
            let p = FixVec::new(x, y);
            let v = p.bit_pack().fix_vec();
            assert_eq!(p, v);
        }
    }
}