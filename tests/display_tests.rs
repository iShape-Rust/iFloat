#[cfg(test)]
mod tests {
    use i_float::fix_vec::FixVec;

    #[test]
    fn test_0() {
        let point = FixVec::new(5, 1);
        let s = format!("{point}");

        assert!(s.eq("[5, 1]"));
    }

}