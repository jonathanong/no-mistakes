use super::{specifier_line, RouteHit};
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::module_facts::arguments::{PathProvenance, RouteKind, RouteSite};
use crate::codebase::ts_source::module_facts::{ModuleImport, ModuleSpan};
use std::path::Path;

impl RouteHit {
    pub(in crate::codebase::rules::test_no_live_repository_scan) fn nondirect(
        file: &Path,
        detail: &str,
        target: &Path,
    ) -> Self {
        Self {
            file: file.to_path_buf(),
            start: 4,
            line: 8,
            kind: RouteKind::NonDirect,
            detail: detail.to_string(),
            target: normalize_path(target),
        }
    }
}

fn import(specifier: &str, start: u32) -> ModuleImport {
    ModuleImport {
        specifier: specifier.to_string(),
        type_only: false,
        span: ModuleSpan { start, end: start },
        bindings: Vec::new(),
        qualifier: None,
        picked: None,
    }
}

#[test]
fn specifier_line_names_the_import_statement() {
    let source = "import { join } from \"node:path\";\nimport { missing } from \"@scope/deep\";\n";
    let start = source.find("@scope/deep").unwrap() as u32;
    let line = specifier_line(
        &PathProvenance::default(),
        &[import("@scope/deep", start)],
        source,
        "@scope/deep",
    );
    assert_eq!(line, 2);
}

#[test]
fn specifier_line_skips_an_import_that_names_a_different_specifier() {
    let source = "import { missing } from \"@scope/other\";\n";
    assert_eq!(
        specifier_line(
            &PathProvenance::default(),
            &[import("@scope/other", 0)],
            source,
            "@scope/deep",
        ),
        1
    );
}

#[test]
fn specifier_line_falls_back_to_the_route_site() {
    let provenance = PathProvenance {
        route_sites: vec![RouteSite {
            start: 0,
            line: 4,
            kind: RouteKind::Dynamic,
            detail: "@scope/deep".to_string(),
        }],
        ..PathProvenance::default()
    };
    assert_eq!(specifier_line(&provenance, &[], "", "@scope/deep"), 4);
    assert_eq!(
        specifier_line(&PathProvenance::default(), &[], "", "@scope/deep"),
        1
    );
}

#[test]
fn specifier_line_keeps_the_recorded_import_line_without_source() {
    let mut provenance = PathProvenance::default();
    provenance.import_lines.insert("@scope/deep".to_string(), 2);
    assert_eq!(specifier_line(&provenance, &[], "", "@scope/deep"), 2);
}
