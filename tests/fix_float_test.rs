use i_float::fix_float::{FIX_FRACTION_BITS, FIX_PI, FixConvert, FixMath};

#[test]
fn test_function_0() {
    assert_eq!(FIX_PI, 3217);
}

#[test]
fn test_function_1() {
    assert_eq!(FIX_FRACTION_BITS, 10);
}

#[test]
fn test_function_2() {
    let a = 1.fix();
    let b = 2.fix();
    let c = a + b;

    assert_eq!(c, 3.fix());
}

#[test]
fn test_function_3() {
    let a = 2.fix();
    let b = 1.fix();
    let c = a - b;

    assert_eq!(c, 1.fix());
}

#[test]
fn test_function_4() {
    let a = 5.sqrt();
    assert_eq!(a, 2);
}

#[test]
fn test_function_5() {
    let a = 9.sqrt();
    assert_eq!(a, 3);
}