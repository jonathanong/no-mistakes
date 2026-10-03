#[test]
fn separators_respect_radix_prefix_and_digit_boundaries() {
    for (text, value) in [
        ("1_000", 1000),
        ("1_0_0", 100),
        ("5_0", 50),
        ("0xF_F", 255),
        ("0o7_0", 56),
        ("0b1_0", 2),
        ("0o_1_755", 1005),
        ("0b_1_0", 2),
        ("0x_F_F", 255),
    ] {
        assert_eq!(super::numeric_literal(text).unwrap(), value);
    }
    for text in ["1__0", "_1", "1_", "0x__F", "0x_", "0o__7", "0b1_2"] {
        assert!(super::numeric_literal(text).is_err(), "{text}");
    }
}
