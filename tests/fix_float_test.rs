use i_float::fix_float::FixFloat;
use i_float::fix_float::fast_square_root;

#[test]
fn test_function_0() {
    assert_eq!(FixFloat::PI, 3217);
}

#[test]
fn test_function_1() {
    assert_eq!(FixFloat::FRACTION_BITS, 10);
}

#[test]
fn test_function_2() {
    let a = FixFloat::new_number(1);
    let b = FixFloat::new_number(2);
    let c = a + b;

    assert_eq!(c, FixFloat::new_number(3));
}

#[test]
fn test_function_3() {
    let a = FixFloat::new_number(2);
    let b = FixFloat::new_number(1);
    let c = a - b;

    assert_eq!(c, FixFloat::new_number(1));
}

#[test]
fn test_function_4() {
    let a = fast_square_root(5);
    assert_eq!(a, 2);
}

#[test]
fn test_function_5() {
    let a = fast_square_root(9);
    assert_eq!(a, 3);
}