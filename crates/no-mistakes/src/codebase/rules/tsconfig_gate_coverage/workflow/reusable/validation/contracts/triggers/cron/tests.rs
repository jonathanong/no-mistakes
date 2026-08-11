use super::*;

#[test]
fn malformed_steps_and_numbers_are_rejected() {
    assert!(!cron_field_part_valid("*/*/2", 0));
    assert!(!cron_number_in_range("invalid", 0, false));
}

#[test]
fn named_months_share_numeric_month_positions() {
    assert!(!cron_atom_or_range_valid("FEB-1", 3));
    assert!(cron_atom_or_range_valid("JAN-1", 3));
    assert!(cron_atom_or_range_valid("FEB-2", 3));
    assert!(cron_atom_or_range_valid("SUN-0", 4));
}
