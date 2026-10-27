// cspell:ignore xuser xdelivery
use super::super::Options;

#[test]
fn referenced_column_option_rejects_unknown_keys_and_non_booleans() {
    for (yaml, expected) in [
        (
            "foreignKeys:\n  allowReferencedColumnNames: true\n",
            "unknown field",
        ),
        (
            "foreignKeys:\n  allowReferencedColumnName: sometimes\n",
            "boolean",
        ),
    ] {
        let error = serde_yaml::from_str::<Options>(yaml)
            .err()
            .unwrap()
            .to_string();
        assert!(error.contains(expected), "{error}");
    }
}
