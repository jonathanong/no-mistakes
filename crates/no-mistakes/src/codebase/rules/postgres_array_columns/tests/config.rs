use super::support::expect_err;

#[test]
fn every_option_error_is_reported() {
    for (yaml, snippet) in [
        ("schemaCatalogPath: ''\n", "schemaCatalogPath: required"),
        ("schemaCatalogPath: '   '\n", "schemaCatalogPath: required"),
        (
            "schemaCatalogPath: schema.json\nallowElementTypes: [UUID]\n",
            "option allowElementTypes: overlaps neverAllowElementTypes: UUID",
        ),
        (
            "schemaCatalogPath: schema.json\nneverAllowElementTypes: [text]\nallowElementTypes: [TEXT, int]\n",
            "option allowElementTypes: overlaps neverAllowElementTypes: TEXT",
        ),
        (
            "schemaCatalogPath: schema.json\nallow:\n  - object: 'column:a.b'\n    reason: ' '\n",
            "option allow: entry column:a.b needs a reason",
        ),
        (
            "schemaCatalogPath: schema.json\nallow:\n  - object: nope\n    reason: because\n",
            "option allow: invalid object ref nope",
        ),
        (
            "schemaCatalogPath: schema.json\nallow:\n  - object: 'column:a.b'\n    reason: one\n  - object: 'column:a.b'\n    reason: two\n",
            "option allow: duplicate entry column:a.b",
        ),
    ] {
        expect_err(yaml, snippet);
    }
}
