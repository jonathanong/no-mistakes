pub(super) const CASES: [(&str, &str, &str, Option<&str>); 12] = [
    (
        "guarded-arm",
        "guardedArm",
        "value.get",
        Some("primary.get"),
    ),
    ("guarded-opposite", "guardedOpposite", "value.get", None),
    ("guarded-after", "guardedAfter", "value.get", None),
    ("guarded-child", "guardedChild", "value.get", None),
    ("guarded-rhs", "guardedRhs", "value.get", None),
    ("guarded-logical", "guardedLogical", "value.get", None),
    ("guarded-restore", "guardedRestore", "value.get", None),
    ("guarded-dynamic", "guardedDynamic", "value.get", None),
    ("guarded-foreign", "guardedForeign", "value.get", None),
    ("guarded-descendant", "guardedDescendant", "value.get", None),
    (
        "guarded-ancestor-rhs",
        "guardedAncestorRhs",
        "value.get",
        None,
    ),
    (
        "guarded-ancestor-conditional",
        "guardedAncestorConditional",
        "value.get",
        None,
    ),
];
