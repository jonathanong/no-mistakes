use super::*;
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::dependencies::graph::GraphBuildPlan;
use crate::codebase::ts_source::facts::{collect_ts_facts_with_session_and_context, TsFactContext};

fn root() -> PathBuf {
    fixture()
        .parent()
        .unwrap()
        .join("declaration-proof-guards/fixture")
}

#[test]
fn declaration_typeof_proof_rejects_optional_computed_signature_and_runtime_shapes() {
    let root = root();
    let report = diagnostic_report(&root);
    assert_eq!(
        reported_channel(&report, "rules"),
        [
            "src/declaration-consumers.ts:5 DefiniteUnusedError",
            "src/declaration-consumers.ts:6 PropertyUnusedError",
            "src/errors.ts:1 RuntimeError",
        ],
        "{report}"
    );
    assert_eq!(
        reported_channel(&report, "advisories"),
        [
            "src/declaration-consumers.ts:7 unanalyzable",
            "src/declaration-consumers.ts:8 unanalyzable",
            "src/declaration-consumers.ts:9 unanalyzable",
            "src/declaration-consumers.ts:10 unanalyzable",
            "src/declaration-consumers.ts:11 unanalyzable",
            "src/declaration-consumers.ts:12 unanalyzable",
            "src/declaration-consumers.ts:13 unanalyzable",
            "src/declaration-consumers.ts:14 unanalyzable",
            "src/runtime.ts:7 unanalyzable",
            "src/runtime.ts:8 unanalyzable",
        ],
        "{report}"
    );
}

#[test]
fn declaration_alias_facts_require_a_required_typeof_class_reference() {
    let root = root();
    let declaration = root.join("node_modules/guarded-constructors/index.d.ts");
    let runtime = root.join("src/runtime.ts");
    let facts = collect_ts_facts_with_session_and_context(
        &AnalysisSession::disabled(),
        &[declaration.clone(), runtime.clone()],
        GraphBuildPlan {
            calls: true,
            extends: true,
            ..Default::default()
        }
        .ts_fact_plan(),
        &TsFactContext {
            root,
            ..Default::default()
        },
    );
    let file = facts.get(&declaration).unwrap();
    assert!(file.parse_error.is_none(), "{:?}", file.parse_error);
    assert_eq!(
        file.ancestry.declaration_aliases,
        [
            ("Constructor".into(), "Root".into()),
            ("ErrorConstructor".into(), "ErrorRoot".into()),
            ("Facade.Required".into(), "Root".into()),
            ("Facade.ErrorRequired".into(), "ErrorRoot".into()),
            ("Facade.Nested".into(), "ns.Deep.Root".into()),
        ]
    );
    let runtime = facts.get(&runtime).unwrap();
    assert!(runtime.parse_error.is_none(), "{:?}", runtime.parse_error);
    assert!(runtime.ancestry.declaration_aliases.is_empty());
}
