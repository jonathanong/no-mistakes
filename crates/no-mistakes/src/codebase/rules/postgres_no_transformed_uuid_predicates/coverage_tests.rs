use super::tests::{config, fixture};
use super::*;

// Output cases moved to cli_postgres_snapshot_transformed_uuid_predicates.

#[test]
fn invalid_globs_fail_before_scanning() {
    for field in ["include", "exclude", "sqlInclude"] {
        let root = fixture();
        let mut cfg = config("");
        cfg.rules[0]
            .options
            .as_mapping_mut()
            .unwrap()
            .insert(field.into(), serde_yaml::from_str("['[']").unwrap());
        let result = check_with_files(&root, &cfg, &[root.join("schema.json")]);
        assert!(result.is_err(), "{field}");
    }
}
