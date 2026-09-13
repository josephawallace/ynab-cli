use std::str::FromStr;

use proptest::prelude::*;
use rust_decimal::Decimal;
use ynab_sdk::{Milliunits, PlanId, PlanMonth};

#[test]
fn plan_selectors_round_trip() {
    assert_eq!(
        PlanId::from_str("last-used").expect("selector").to_string(),
        "last-used"
    );
    assert_eq!(
        PlanId::from_str("default").expect("selector").to_string(),
        "default"
    );
}

#[test]
fn plan_month_requires_first_day() {
    assert!(PlanMonth::from_str("2026-01-01").is_ok());
    assert!(PlanMonth::from_str("2026-01-02").is_err());
}

proptest! {
    #[test]
    fn milliunits_round_trip(value in -9_000_000i64..9_000_000i64) {
        let decimal = Decimal::new(value, 3);
        prop_assert_eq!(Milliunits::from_decimal(decimal), Some(Milliunits(value)));
    }
}
