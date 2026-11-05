use super::super::{extract, SetSpec};
use crate::codebase::rules::source_store_for_files;
use std::path::PathBuf;

#[test]
fn yaml_selectors_share_one_source_session_parse() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/options-audit/fixture/finite-shared-yaml");
    let paths = vec![root.join("values.yml")];
    let sources = source_store_for_files(&paths);
    let mut expected = None;
    for (kind, key) in [
        ("yaml-sequence", "values"),
        ("yaml-string-selector", "values.[]"),
    ] {
        let spec = SetSpec {
            name: "values".into(),
            file: "values.yml".into(),
            kind: kind.into(),
            key: key.into(),
            ..Default::default()
        };
        let extracted =
            extract::extract_set_with_sources(&root, &spec, &paths, &[], &sources, None).unwrap();
        assert_eq!(extracted.values.len(), 2);
        if let Some(expected) = &expected {
            assert_eq!(expected, &extracted.values);
        }
        expected = Some(extracted.values);
    }
    assert_eq!(sources.yaml_parse_count(), 1);
}
