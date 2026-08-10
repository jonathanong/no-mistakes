use super::*;

#[test]
fn malformed_steps_and_numbers_are_rejected() {
    assert!(!cron_field_part_valid("*/*/2", 0));
    assert!(!cron_number_in_range("invalid", 0, false));
}
