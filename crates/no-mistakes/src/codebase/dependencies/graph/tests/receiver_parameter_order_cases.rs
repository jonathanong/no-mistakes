use super::cases::Case;

pub(super) fn cases() -> &'static [Case] {
    &[
        (
            "unique-parameter-phase",
            Some("escapedArgument"),
            "fn",
            None,
        ),
        (
            "unique-parameter-phase",
            Some("uniqueArgument"),
            "fn",
            Some("suppliedUnique.get"),
        ),
        ("unique-parameter-phase", Some("uniqueDefault"), "fn", None),
        (
            "unique-parameter-phase",
            Some("multipleArgument"),
            "fn",
            None,
        ),
        (
            "unique-parameter-phase",
            Some("recursiveArgument"),
            "fn",
            None,
        ),
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
