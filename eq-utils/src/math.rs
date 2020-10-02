use super::*;
use substrate_fixed;
use substrate_fixed::transcendental;

type InnerFixed = substrate_fixed::FixedI128<substrate_fixed::types::extra::U64>;

fn to_inner_fixed(x: FixedI64) -> InnerFixed {
    let nom = InnerFixed::from_num(x.into_inner());
    let denom = InnerFixed::from_num(FixedI64::DIV);

    nom / denom
}

fn from_inner_fixed(x: InnerFixed) -> FixedI64 {
    let y = x * InnerFixed::from_num(FixedI64::DIV);
    let raw = y.round().to_bits() >> 64;

    FixedI64::from_inner(raw as i64)
}

pub trait MathUtils
where
    Self: Sized,
{
    fn sqrt(self) -> Result<Self, ()>;
    fn ln(self) -> Result<Self, ()>;

    fn sqr(self) -> Self; // add Option or result?

    fn exp(self) -> Result<Self, ()>;

    fn pow(self, y: Self) -> Result<Self, ()>;
}

impl MathUtils for FixedI64 {
    fn sqrt(self) -> Result<Self, ()> {
        let result = transcendental::sqrt(to_inner_fixed(self))?;
        Ok(from_inner_fixed(result))
    }

    fn sqr(self) -> Self {
        self.saturating_mul(self)
    }

    fn ln(self) -> Result<Self, ()> {
        let result = transcendental::ln(to_inner_fixed(self))?;
        Ok(from_inner_fixed(result))
    }

    fn exp(self) -> Result<Self, ()> {
        let result = transcendental::exp(to_inner_fixed(self))?;
        Ok(from_inner_fixed(result))
    }

    fn pow(self, y: Self) -> Result<Self, ()> {
        let result = transcendental::pow(to_inner_fixed(self), to_inner_fixed(y))?;
        Ok(from_inner_fixed(result))
    }
}

#[test]
fn sqrt_for_integers() {
    assert_eq!(
        FixedI64::saturating_from_integer(25).sqrt().unwrap(),
        FixedI64::saturating_from_integer(5)
    );
    assert_eq!(
        FixedI64::saturating_from_integer(1729225).sqrt().unwrap(),
        FixedI64::saturating_from_integer(1315)
    );
}

#[test]
fn sqrt_for_floats() {
    assert!(
        (FixedI64::from_inner(123455224880000).sqrt().unwrap()
            - FixedI64::from_inner(351361957000))
        .into_inner()
        .abs()
            < 100_000
    );

    assert!(
        (FixedI64::from_inner(98745023550000).sqrt().unwrap() - FixedI64::from_inner(314237209000))
            .into_inner()
            .abs()
            < 100_000
    );
}

#[test]
fn ln_test() {
    assert!(
        (FixedI64::from_inner(1).ln().unwrap() - FixedI64::from_inner(-20723265836))
            .into_inner()
            .abs()
            < 100_000
    );
    assert!(
        (FixedI64::saturating_from_integer(25).ln().unwrap() - FixedI64::from_inner(3218875798))
            .into_inner()
            .abs()
            < 100_000
    );
    assert!(
        (FixedI64::saturating_from_integer(97965987).ln().unwrap()
            - FixedI64::from_inner(18400130904))
        .into_inner()
        .abs()
            < 100_000
    );
    assert!(
        (FixedI64::from_inner(123215425400000).ln().unwrap() - FixedI64::from_inner(11721690000))
            .into_inner()
            .abs()
            < 100_000
    );
    assert!(
        (FixedI64::from_inner(846841412120000).ln().unwrap() - FixedI64::from_inner(13649269000))
            .into_inner()
            .abs()
            < 100_000
    );
    assert!(
        (FixedI64::from_inner(5155355121353000).ln().unwrap() - FixedI64::from_inner(15455547000))
            .into_inner()
            .abs()
            < 100_000
    );
}

#[test]
fn ln_fails() {
    assert!(FixedI64::saturating_from_integer(-1).ln().is_err());
}

#[test]
fn sqrt_fails() {
    assert!(FixedI64::saturating_from_integer(-1).sqrt().is_err());
}

#[test]
fn exp_test() {
    assert_eq_fx64!(fx64!(1, 0).exp().unwrap(), fx64!(2, 718281828), 7)
}

#[test]
fn pow_test() {
    let x = fx64!(10, 0);
    let y = fx64!(0, 75);

    let actual = x.pow(y).unwrap();
    let expected = fx64!(5, 623413252);
    assert_eq_fx64!(actual, expected, 4);
}

#[test]
fn pow_test_0() {
    let x = fx64!(33, 0);
    let y = fx64!(0, 0);

    let actual = x.pow(y).unwrap();
    let expected = fx64!(1, 0);
    assert_eq_fx64!(actual, expected, 4);
}

#[test]
fn pow_test_e() {
    let e = fx64!(2, 718281828);
    let y = fx64!(1, 0);

    let actual = e.pow(y).unwrap();
    let expected = fx64!(2, 718281828);
    assert_eq_fx64!(actual, expected, 3);
}

#[test]
fn pow_test_neg() {
    let e = fx64!(2, 0);
    let y = fx64!(-2, 0);

    let actual = e.pow(y).unwrap();
    let expected = fx64!(0, 25);
    assert_eq_fx64!(actual, expected, 4);
}

#[test]
fn ln_test_e() {
    let e = fx64!(2, 718281828);

    let actual = e.ln().unwrap();
    let expected = fx64!(1, 0);
    assert_eq_fx64!(actual, expected, 4);
}

#[test]
fn sqrt_test_small_num() {
    let actual = fx64!(0, 001886109).sqrt().unwrap();
    let expected = fx64!(0, 043429356);
    assert_eq_fx64!(actual, expected, 5);
}

#[test]
fn inner_fixed_conversions() {
    let num = fx64!(12, 345);
    let inner_fixed = to_inner_fixed(num);
    let actual = from_inner_fixed(inner_fixed);

    assert_eq!(actual, num);
}
