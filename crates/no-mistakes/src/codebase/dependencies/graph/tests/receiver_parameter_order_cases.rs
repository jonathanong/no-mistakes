use super::cases::Case;

pub(super) fn cases() -> &'static [Case] {
    &[
        (
            "nested-parameter-order",
            Some("parameterArgument"),
            "fn",
            Some("suppliedParameter.get"),
        ),
        (
            "nested-parameter-order",
            Some("parameterDefault"),
            "fn",
            None,
        ),
        (
            "nested-parameter-order",
            Some("parameterMultiArgument"),
            "fn",
            Some("suppliedMulti.get"),
        ),
        (
            "nested-parameter-order",
            Some("parameterMultiDefault"),
            "fn",
            None,
        ),
        ("nested-parameter-order", Some("parameterLate"), "fn", None),
    ]
}
