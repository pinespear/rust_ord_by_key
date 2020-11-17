#[cfg(test)]
mod tests {
    use ::core::cmp::Ordering;
    use ord_by_key::ord_eq_by_key_selector;

    #[ord_eq_by_key_selector(|i| i.0)]
    pub struct T1(i32);

    #[ord_eq_by_key_selector(|i| i.0, i.1)]
    pub struct T2(i32, i32);

    #[test]
    fn test_eq() {
        assert!(T1(1).eq(&T1(1)) == true);
        assert!(T1(1).eq(&T1(0)) == false);

        assert!(T2(1, 1).eq(&T2(1, 1)) == true);
        assert!(T2(0, 1).eq(&T2(1, 1)) == false);
        assert!(T2(1, 0).eq(&T2(1, 1)) == false);
    }

    #[test]
    fn test_cmp() {
        assert!(T1(1).cmp(&T1(2)) == Ordering::Less);
        assert!(T1(1).cmp(&T1(1)) == Ordering::Equal);
        assert!(T1(1).cmp(&T1(0)) == Ordering::Greater);

        assert!(T2(1, 1).cmp(&T2(1, 1)) == Ordering::Equal);
        assert!(T2(1, 1).cmp(&T2(2, 1)) == Ordering::Less);
        assert!(T2(1, 1).cmp(&T2(2, 1)) == Ordering::Less);
        assert!(T2(1, 1).cmp(&T2(2, 2)) == Ordering::Less);
        assert!(T2(1, 1).cmp(&T2(1, 2)) == Ordering::Less);
        assert!(T2(1, 2).cmp(&T2(2, 1)) == Ordering::Less);
    }
}
