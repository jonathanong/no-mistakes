use super::super::compile::compile;
use super::super::Options;

fn expect_err(yaml: &str, snippet: &str) {
    let options: Options = serde_yaml::from_str(yaml).unwrap();
    let error = match compile(&options, None) {
        Err(error) => error.to_string(),
        Ok(_) => panic!("expected config error containing {snippet}"),
    };
    assert!(error.contains(snippet), "{error}");
}

#[test]
fn every_option_error_is_reported() {
    for (yaml, snippet) in [
        ("schemaCatalogPath: ''\n", "schemaCatalogPath: required"),
        ("schemaCatalogPath: '   '\n", "schemaCatalogPath: required"),
        (
            "schemaCatalogPath: schema.json\ncolumnTypes: []\n",
            "columnTypes: empty",
        ),
        (
            "schemaCatalogPath: schema.json\nnamePatterns: ['[']\n",
            "option namePatterns: invalid regex",
        ),
        (
            "schemaCatalogPath: schema.json\nignoreTablePatterns: ['[']\n",
            "option ignoreTablePatterns: invalid regex",
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
            "schemaCatalogPath: schema.json\nallow:\n  - object: table:orders\n    reason: because\n",
            "option allow: expected a column object ref, got table:orders",
        ),
        (
            "schemaCatalogPath: schema.json\nallow:\n  - object: 'column:a.b'\n    reason: one\n  - object: 'column:a.b'\n    reason: two\n",
            "option allow: duplicate entry column:a.b",
        ),
    ] {
        expect_err(yaml, snippet);
    }
}
