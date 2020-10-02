use sp_arithmetic::{FixedI64, FixedPointNumber};

#[macro_export]
macro_rules! fx64 {
    ($i: expr, $f:expr) => {{
        let mut fraq_str = String::from(stringify!($f));
        let existing_zeros_num = fraq_str.len() - fraq_str.trim_end_matches('0').len();

        fraq_str.push_str("000000000");
        let fraq_len = fraq_str[0..9].trim_end_matches('0').len();

        let mut fraq_div = 1i64;

        for _ in 0..existing_zeros_num {
            fraq_div = fraq_div * 10;
        }

        let mut fraq_mul = 1i64;

        for _ in 0..(9 - fraq_len) {
            fraq_mul = fraq_mul * 10;
        }

        FixedI64::from_inner($i * 1_000_000_000i64 + $f / fraq_div * fraq_mul)
    }};
}

#[macro_export]
macro_rules! assert_eq_fx64 {
    ($left:expr, $right:expr, $prec:expr) => {{
        let delta = ($left - $right).into_inner().abs();

        let mut max_delta = 1;

        for _ in 0..(9 - $prec) {
            max_delta = max_delta * 10;
        }

        assert!(
            delta < max_delta,
            "{:?} ({:?}) is not equals to right {:?} ({:?}) with precision {:?}",
            stringify!($left),
            $left,
            stringify!($right),
            $right,
            $prec
        );
    }};
}

pub fn to_prec(n: i32, x: FixedI64) -> FixedI64 {
    let mut y = 1i64;

    for _ in 0..(9 - n) {
        y = y * 10;
    }

    FixedI64::from_inner((x.into_inner() / y) * y)
}

mod test {
    #![cfg(test)]

    use super::to_prec;
    use sp_arithmetic::{FixedI64, FixedPointNumber};

    #[test]
    fn test_fx64_trailing_zeros() {
        let actual = fx64!(0, 0016000);
        let expected = FixedI64::saturating_from_rational(16, 10000);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_fx64_no_trailing_zeros() {
        let actual = fx64!(0, 0016);
        let expected = FixedI64::saturating_from_rational(16, 10000);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_to_prec() {
        assert_eq!(to_prec(2, fx64!(0, 1234)), fx64!(0, 12))
    }
}
