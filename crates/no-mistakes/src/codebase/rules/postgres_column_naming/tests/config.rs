use super::support::expect_err;

#[test]
fn unknown_option_keys_are_rejected() {
    let parsed = serde_yaml::from_str::<super::super::Options>(
        "schemaCatalogPath: schema.json\ntypeRule: []\n",
    );
    let Err(error) = parsed else {
        panic!("expected unknown field");
    };
    let error = error.to_string();
    assert!(error.contains("unknown field"), "{error}");
}

#[test]
fn every_option_error_is_reported() {
    for (yaml, snippet) in [
        ("schemaCatalogPath: ''\n", "schemaCatalogPath: required"),
        ("schemaCatalogPath: '   '\n", "schemaCatalogPath: required"),
        (
            "schemaCatalogPath: schema.json\ntypeRules:\n  - types: []\n    namePattern: _at$\n",
            "typeRules: empty types",
        ),
        (
            "schemaCatalogPath: schema.json\ntypeRules:\n  - types: [boolean]\n    namePattern: '['\n",
            "typeRules: invalid regex",
        ),
        (
            "schemaCatalogPath: schema.json\ntypeRules:\n  - types: [boolean]\n    namePattern: _at$\n    hint: '   '\n",
            "typeRules: empty hint",
        ),
        (
            "schemaCatalogPath: schema.json\nnameTypeRules:\n  - types: [uuid]\n",
            "nameTypeRules: namePattern is required",
        ),
        (
            "schemaCatalogPath: schema.json\nnameTypeRules:\n  - namePattern: '   '\n    types: [uuid]\n",
            "nameTypeRules: namePattern is required",
        ),
        (
            "schemaCatalogPath: schema.json\nnameTypeRules:\n  - namePattern: _at$\n    types: []\n",
            "nameTypeRules: empty types",
        ),
        (
            "schemaCatalogPath: schema.json\nnameTypeRules:\n  - namePattern: _at$\n    types: [uuid]\n    hint: ''\n",
            "nameTypeRules: empty hint",
        ),
        (
            "schemaCatalogPath: schema.json\nnameTypeRules:\n  - namePattern: _at$\n    types: [uuid]\n    hint: '  '\n",
            "nameTypeRules: empty hint",
        ),
        (
            "schemaCatalogPath: schema.json\nnameTypeRules:\n  - namePattern: '['\n    types: [uuid]\n",
            "nameTypeRules: invalid regex",
        ),
        (
            "schemaCatalogPath: schema.json\nignoreTablePatterns: ['[']\n",
            "ignoreTablePatterns: invalid regex",
        ),
        (
            "schemaCatalogPath: schema.json\nforeignKeys:\n  targetSuffixes:\n    - tables: []\n      suffixes: ['_id']\n",
            "targetSuffixes: empty tables",
        ),
        (
            "schemaCatalogPath: schema.json\nforeignKeys:\n  targetSuffixes:\n    - tables: [users]\n      suffixes: []\n",
            "targetSuffixes: empty suffixes",
        ),
        (
            "schemaCatalogPath: schema.json\nforeignKeys:\n  targetSuffixes:\n    - tables: [users]\n      suffixes: ['']\n",
            "targetSuffixes: empty suffix",
        ),
        (
            "schemaCatalogPath: schema.json\nforeignKeys:\n  targetSuffixes:\n    - tables: [users]\n      suffixes: ['_user_id']\n    - tables: [users]\n      suffixes: ['_by_id']\n",
            "targetSuffixes: table users listed twice",
        ),
        (
            "schemaCatalogPath: schema.json\nforeignKeys:\n  reservedSuffixes:\n    - suffix: ''\n      tables: [users]\n",
            "reservedSuffixes: empty suffix",
        ),
        (
            "schemaCatalogPath: schema.json\nforeignKeys:\n  reservedSuffixes:\n    - suffix: _id\n      tables: []\n",
            "reservedSuffixes: empty tables",
        ),
        (
            "schemaCatalogPath: schema.json\nforeignKeys:\n  reservedSuffixes:\n    - suffix: _id\n      tables: [users]\n    - suffix: _id\n      tables: [teams]\n",
            "reservedSuffixes: duplicate suffix _id",
        ),
        (
            "schemaCatalogPath: schema.json\nforeignKeys:\n  reservedSuffixes:\n    - suffix: _id\n      tables: [users]\n      hint: ' '\n",
            "reservedSuffixes: empty hint",
        ),
        (
            "schemaCatalogPath: schema.json\nforeignKeys:\n  targetMatch: sideways\n",
            "targetMatch: unknown value sideways",
        ),
        (
            "schemaCatalogPath: schema.json\nforeignKeys:\n  targetNames:\n    - tablePattern: '['\n      name: item\n",
            "targetNames: invalid regex",
        ),
        (
            "schemaCatalogPath: schema.json\nforeignKeys:\n  targetNames:\n    - tablePattern: '^a$'\n      name: ''\n",
            "targetNames: empty name",
        ),
        (
            "schemaCatalogPath: schema.json\nforeignKeys:\n  targetNames:\n    - tablePattern: '^archived_(.+)_snapshots$'\n      name: '$2'\n",
            "$2 is outside the pattern's groups",
        ),
        (
            "schemaCatalogPath: schema.json\nforeignKeys:\n  requireForeignKey:\n    types: []\n    namePattern: _id$\n",
            "requireForeignKey: empty types",
        ),
        (
            "schemaCatalogPath: schema.json\nforeignKeys:\n  requireForeignKey:\n    types: [uuid]\n    namePattern: ''\n",
            "requireForeignKey: namePattern is required",
        ),
        (
            "schemaCatalogPath: schema.json\nforeignKeys:\n  requireForeignKey:\n    types: [uuid]\n    namePattern: '['\n",
            "requireForeignKey: invalid regex",
        ),
        (
            "schemaCatalogPath: schema.json\nforeignKeys:\n  requireForeignKey:\n    types: [uuid]\n    namePattern: _id$\n    exempt:\n      - namePattern: '^cursor_'\n        reason: '   '\n",
            "exempt entry ^cursor_ needs a reason",
        ),
        (
            "schemaCatalogPath: schema.json\nforeignKeys:\n  requireForeignKey:\n    types: [uuid]\n    namePattern: _id$\n    exempt:\n      - namePattern: '['\n        reason: because\n",
            "requireForeignKey: invalid regex",
        ),
        (
            "schemaCatalogPath: schema.json\nforeignKeys:\n  requireForeignKey:\n    types: [uuid]\n    namePattern: _id$\n    exempt:\n      - namePattern: '^cursor_'\n        reason: one\n      - namePattern: '^cursor_'\n        reason: two\n",
            "duplicate namePattern ^cursor_",
        ),
        (
            "schemaCatalogPath: schema.json\nforbiddenColumnNames:\n  - pattern: ''\n    hint: no\n",
            "forbiddenColumnNames: empty pattern",
        ),
        (
            "schemaCatalogPath: schema.json\nforbiddenColumnNames:\n  - pattern: '   '\n    hint: no\n",
            "forbiddenColumnNames: empty pattern",
        ),
        (
            "schemaCatalogPath: schema.json\nforbiddenColumnNames:\n  - pattern: '['\n    hint: no\n",
            "forbiddenColumnNames: invalid regex",
        ),
        (
            "schemaCatalogPath: schema.json\nforbiddenColumnNames:\n  - pattern: table\n    hint: ' '\n",
            "forbiddenColumnNames: empty hint",
        ),
        (
            "schemaCatalogPath: schema.json\nforbiddenColumnNames:\n  - pattern: table\n    hint: one\n  - pattern: table\n    hint: two\n",
            "duplicate pattern table",
        ),
        (
            "schemaCatalogPath: schema.json\nallow:\n  - object: 'column:a.b'\n    reason: ' '\n",
            "allow: entry column:a.b needs a reason",
        ),
        (
            "schemaCatalogPath: schema.json\nallow:\n  - object: nope\n    reason: because\n",
            "allow: invalid object ref nope",
        ),
        (
            "schemaCatalogPath: schema.json\nallow:\n  - object: 'column:a.b'\n    reason: one\n  - object: 'column:a.b'\n    reason: two\n",
            "allow: duplicate entry column:a.b",
        ),
    ] {
        expect_err(yaml, snippet);
    }
}
