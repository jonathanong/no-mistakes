use super::super::scan;
use super::incomplete::{fact_miss_root, scan_fact_miss};
use super::support::{fragment_facts, import, insert, sql_tag};
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::extract::{
    ExportedBinding, ExtractedImport, ImportKind, ImportedBindingKind,
};
use crate::codebase::dependencies::graph::test_support::from_raw_maps;
use crate::codebase::ts_source::facts::TsFileFacts;
use std::collections::HashMap;
use std::sync::Arc;

#[test]
fn local_export_without_offset_is_not_a_gap() {
    let mut consumer = TsFileFacts::default();
    consumer.imported_bindings.push(import(
        "./columns",
        "app",
        "widgetColumns",
        ImportedBindingKind::Named,
    ));
    consumer.exported_bindings.push(ExportedBinding {
        specifier: None,
        local: "app".to_string(),
        exported: "default".to_string(),
    });
    consumer.source = Some(Arc::<str>::from("export default app;\n"));
    let report = scan_fact_miss("reexport.ts", consumer);
    assert!(
        report
            .gaps
            .iter()
            .all(|gap| !gap.message.contains("export offset")),
        "{:?}",
        report.gaps
    );
}

#[test]
fn namespace_reexport_without_offset_is_not_a_gap() {
    let mut consumer = TsFileFacts::default();
    consumer.imported_bindings.push(import(
        "./columns",
        "sdk",
        "*",
        ImportedBindingKind::Namespace,
    ));
    consumer.exported_bindings.push(ExportedBinding {
        specifier: None,
        local: "sdk".to_string(),
        exported: "sdk".to_string(),
    });
    consumer.source = Some(Arc::<str>::from("export { sdk };\n"));
    let report = scan_fact_miss("reexport.ts", consumer);
    assert!(
        report
            .gaps
            .iter()
            .all(|gap| !gap.message.contains("export offset")),
        "{:?}",
        report.gaps
    );
}

#[test]
fn recorded_sourced_export_offset_is_not_a_gap() {
    let mut consumer = TsFileFacts::default();
    consumer.exported_bindings.push(ExportedBinding {
        specifier: Some("./columns".to_string()),
        local: "widgetColumns".to_string(),
        exported: "itemColumns".to_string(),
    });
    consumer.namespaces.sourced_export_offsets.push((
        "./columns".to_string(),
        "widgetColumns".to_string(),
        "itemColumns".to_string(),
        0,
    ));
    consumer.source = Some(Arc::<str>::from(
        "export { widgetColumns as itemColumns } from \"./columns\";\n",
    ));
    let report = scan_fact_miss("reexport.ts", consumer);
    assert!(report.findings.is_empty(), "{:?}", report.findings);
    assert!(
        report
            .gaps
            .iter()
            .all(|gap| !gap.message.contains("export offset")),
        "{:?}",
        report.gaps
    );
}

#[test]
fn missing_star_reexport_offset_is_a_gap() {
    let mut consumer = TsFileFacts::default();
    consumer.exported_bindings.push(ExportedBinding {
        specifier: Some("./columns".to_string()),
        local: "*".to_string(),
        exported: "columns".to_string(),
    });
    consumer.source = Some(Arc::<str>::from(
        "export * as columns from \"./columns\";\n",
    ));
    let report = scan_fact_miss("reexport.ts", consumer);
    assert!(
        report
            .gaps
            .iter()
            .any(|gap| gap.message.contains("export offset for `columns`")),
        "{:?}",
        report.gaps
    );
}

#[test]
fn sourced_reexport_without_fragments_is_not_a_gap() {
    let root = fact_miss_root();
    let mut consumer = TsFileFacts::default();
    consumer.exported_bindings.push(ExportedBinding {
        specifier: Some("./plain".to_string()),
        local: "name".to_string(),
        exported: "name".to_string(),
    });
    consumer.source = Some(Arc::<str>::from("export { name } from \"./plain\";\n"));
    let mut plain = TsFileFacts::default();
    plain.source = Some(Arc::<str>::from("export const name = 1;\n"));
    let mut facts = CheckFactMap {
        graph_files_complete: true,
        ..CheckFactMap::default()
    };
    insert(&mut facts, root.join("plain.ts"), plain);
    insert(&mut facts, root.join("reexport.ts"), consumer);
    let graph = from_raw_maps(root.clone(), HashMap::new(), HashMap::new());
    let report = scan::scan(&root, &[sql_tag()], Some(&graph), &facts, None);
    assert!(
        report
            .gaps
            .iter()
            .all(|gap| !gap.message.contains("export offset")),
        "{:?}",
        report.gaps
    );
}

#[test]
fn bare_sourced_reexport_without_a_graph_is_not_a_gap() {
    let root = fact_miss_root();
    let mut consumer = TsFileFacts::default();
    consumer.exported_bindings.push(ExportedBinding {
        specifier: Some("pkg".to_string()),
        local: "sdk".to_string(),
        exported: "sdk".to_string(),
    });
    consumer.source = Some(Arc::<str>::from("export { sdk } from \"pkg\";\n"));
    let mut facts = CheckFactMap {
        graph_files_complete: true,
        ..CheckFactMap::default()
    };
    insert(
        &mut facts,
        root.join("columns.ts"),
        fragment_facts(&[("widgetColumns", 10)]),
    );
    insert(&mut facts, root.join("reexport.ts"), consumer);
    let report = scan::scan(&root, &[sql_tag()], None, &facts, None);
    assert!(
        report
            .gaps
            .iter()
            .all(|gap| !gap.message.contains("export offset")),
        "{:?}",
        report.gaps
    );
}

#[test]
fn computed_import_or_require_is_a_specifier_gap() {
    for kind in [ImportKind::Dynamic, ImportKind::Require] {
        let mut consumer = TsFileFacts {
            source: Some(Arc::<str>::from("const parts = import(name);\n")),
            ..TsFileFacts::default()
        };
        consumer.imports.push(ExtractedImport {
            specifier: "name".to_string(),
            kind,
            line: 1,
            function_scope: None,
            function_scope_id: None,
            side_effect_only: false,
            re_export: false,
            runtime_reachable: false,
            computed: true,
        });
        let report = scan_fact_miss("static-import.ts", consumer);
        assert!(
            report.gaps.iter().any(|gap| {
                gap.message.contains("`static-import.ts`")
                    && gap.message.contains("no recorded specifier")
            }),
            "{kind:?}: {:?}",
            report.gaps
        );
    }
}
