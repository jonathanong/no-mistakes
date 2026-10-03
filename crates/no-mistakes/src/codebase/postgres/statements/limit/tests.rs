#[test]
fn separators_require_digits_on_both_sides() {
    for (text, value) in [
        ("1_000", 1000),
        ("1_0_0", 100),
        ("5_0", 50),
        ("0xF_F", 255),
        ("0o7_0", 56),
        ("0b1_0", 2),
    ] {
        assert_eq!(super::numeric_literal(text).unwrap(), value);
    }
    for text in ["1__0", "_1", "1_", "0x_F", "0b1_2"] {
        assert!(super::numeric_literal(text).is_err(), "{text}");
    }
}
