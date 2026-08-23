use super::*;
use crate::codebase::ts_resolver::{ImportClassification, ImportResolution};
use std::collections::BTreeSet;
use std::path::PathBuf;

struct NoopResolver;

impl ImportResolution for NoopResolver {
    fn resolve(&self, _: &str, _: &Path) -> Option<PathBuf> {
        None
    }

    fn resolution_candidates(&self, _: &str, _: &Path) -> BTreeSet<PathBuf> {
        BTreeSet::new()
    }

    fn visible_files(&self) -> Option<&dyn crate::codebase::ts_resolver::VisiblePathLookup> {
        None
    }

    fn classify_import(
        &self,
        _: &str,
        _: &Path,
        _: &crate::codebase::workspaces::IndexedWorkspaceMap,
        _: &dyn crate::codebase::ts_resolver::VisiblePathLookup,
    ) -> ImportClassification {
        unreachable!("json workspace tests do not classify imports")
    }
}

fn parse_json(source: &str) -> Result<Vec<ConfigProject>> {
    parse(
        source,
        Path::new("/repo/vitest.workspace.json"),
        Path::new("/repo"),
        Path::new("/repo"),
        &NoopResolver,
    )
}

#[test]
fn json_project_parse_rejects_invalid_payloads() {
    for source in [
        "{",
        "{}",
        "[1]",
        r#"[{"test": 1}]"#,
        r#"[{"name": 1}]"#,
        r#"[{"root": 1}]"#,
        r#"[{"extends": "yes"}]"#,
        r#"[{"include": true}]"#,
        r#"[{"include": [1]}]"#,
    ] {
        assert!(parse_json(source).is_err(), "{source}");
    }
}

#[test]
fn json_project_parse_reads_object_fields_and_nested_test() {
    let projects = parse_json(
        r#"[{
            "name": {"label": "web"},
            "root": "apps/web",
            "include": "src/**/*.ts",
            "exclude": ["**/*.spec.ts"],
            "setupFiles": [],
            "globalSetup": ["setup.ts"],
            "extends": true,
            "test": {
                "name": "nested",
                "include": ["test/**/*.ts"],
                "setupFiles": ["nested-setup.ts"],
                "extends": false
            }
        }]"#,
    )
    .unwrap();
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0].policy_name.as_deref(), Some("nested"));
}

#[test]
fn json_project_parse_keeps_string_and_negated_workspace_entries() {
    let projects = parse_json(r#"["./pkg", "!./skip", {"name": "inline"}]"#).unwrap();
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0].policy_name.as_deref(), Some("inline"));
}
