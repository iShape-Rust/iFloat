#![cfg(feature = "core")]

#[test]
fn exported_macro_does_not_require_an_int_point_import() {
    let point = i_float::int_pnt!(3_i64, -4_i64);
    assert_eq!((point.x, point.y), (3_i64, -4_i64));
}
