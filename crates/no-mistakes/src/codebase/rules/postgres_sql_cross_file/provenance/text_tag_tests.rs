use super::*;
use crate::codebase::check_facts::{CheckFactMap, CheckFileFacts};
use crate::codebase::dependencies::graph::test_support::from_raw_maps;
use crate::codebase::postgres::EmbeddedSqlCall;
use crate::codebase::rules::postgres_sql_cross_file::SqlCall;
use crate::codebase::ts_source::facts::{collect_ts_facts, TsFactPlan};
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::Arc;

const SOURCE: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
    "/../../test-cases/rules/postgres-no-unbounded-sql-identifier/fixture/issue-3073-fragments/src/text-raw.ts"));

fn prepared() -> (CheckFactMap, PathBuf) {
    let file = crate::test_support::rule_fixture_root("postgres-no-unbounded-sql-identifier")
        .join("issue-3073-fragments/src/text-raw.ts");
    let collected = collect_ts_facts(
        std::slice::from_ref(&file),
        TsFactPlan {
            imports: true,
            function_calls: true,
            value_flow: true,
            ..Default::default()
        },
    );
    let mut facts = CheckFactMap {
        files: vec![file.clone()],
        ..Default::default()
    };
    facts.ts.insert(
        file.clone(),
        CheckFileFacts {
            ts: Arc::new(collected.get(&file).unwrap().clone()),
            ..Default::default()
        }
        .into(),
    );
    (facts, file)
}

fn tags() -> [TrustedSqlTag; 1] {
    [TrustedSqlTag {
        module: "@example/sql".into(),
        name: "sql".into(),
    }]
}

fn tag_for<'a>(facts: &'a CheckFactMap, file: &Path, name: &str) -> &'a ValueCall {
    let flow = &facts.get_ts_facts(file).unwrap().value_flow;
    let binding = flow
        .bindings
        .iter()
        .find(|binding| binding.name == name)
        .unwrap();
    let ValueExpression::Call(start, end) = binding.value else {
        panic!("tag binding {name}");
    };
    flow.calls
        .iter()
        .find(|call| (call.offset, call.end) == (start, end))
        .unwrap()
}

#[test]
fn text_tag_proofs_follow_aliases_and_reject_raw_or_opaque_slots() {
    let (facts, file) = prepared();
    let tags = tags();
    let bindings = Bindings::new(&facts).with_trusted_tags(&tags);
    for (name, expected) in [
        ("literals", true),
        ("aliasedScalar", true),
        ("parameterMember", true),
        ("localMember", true),
        ("legacyRawValue", true),
        ("diamond", true),
        ("rawSlot", false),
        ("rawAliasSlot", false),
        ("rawFactorySlot", false),
        ("opaqueSlot", false),
        ("optionalSlot", false),
        ("optionalAliasSlot", false),
        ("objectSlot", false),
        ("objectMemberSlot", false),
        ("mixedSlot", false),
        ("optionalDefaultSlot", false),
        ("rawDefaultSlot", false),
        ("aliasDefaultSlot", false),
        ("capturedDefaultSlot", false),
        ("literalDefaultSlot", true),
        ("untypedSlot", true),
        ("destructuredSlot", true),
        ("nominalSlot", true),
        ("destructuredLiteralSlot", true),
        ("destructuredRawSlot", false),
        ("destructuredOptionalSlot", false),
    ] {
        assert_eq!(
            is_text_tag(&bindings, &file, tag_for(&facts, &file, name), &tags),
            expected,
            "{name}"
        );
    }
    let literal_tag = tag_for(&facts, &file, "literals");
    assert_eq!(
        facts
            .get_ts_facts(&file)
            .unwrap()
            .value_flow
            .tagged_templates
            .get(&(literal_tag.offset, literal_tag.end))
            .unwrap()
            .literal_arguments,
        [1, 2]
    );
}

#[test]
fn text_tag_alias_diamonds_evaluate_each_binding_once() {
    let (facts, file) = prepared();
    let tags = tags();
    let bindings = Bindings::new(&facts).with_trusted_tags(&tags);
    let mut state = ProofState::default();
    assert!(text_tag(
        &bindings,
        &file,
        tag_for(&facts, &file, "diamond"),
        &tags,
        &mut state
    ));
    assert_eq!(state.work, 13);
}

#[test]
fn active_text_proof_cycles_remain_unproven() {
    let mut state = ProofState::default();
    assert!(!memoized(&mut state, ProofKey::Call(1, 2), |state| {
        memoized(state, ProofKey::Call(1, 2), |_| true)
    }));
    assert_eq!(state.work, 1);
}

#[test]
fn raw_text_and_direct_configured_tags_keep_environment_sources() {
    let (facts, file) = prepared();
    let tags = tags();
    let bindings = Bindings::new(&facts).with_trusted_tags(&tags);
    let graph = from_raw_maps(
        file.parent().unwrap().to_path_buf(),
        Default::default(),
        Default::default(),
    );
    let flow = &facts.get_ts_facts(&file).unwrap().value_flow;
    let embedded = EmbeddedSqlCall::default();
    for (marker, expected) in [
        ("BOUND_VALUE", vec![]),
        ("INLINE_RAW", vec!["INLINE_RAW"]),
        ("ALIAS_TABLE", vec!["ALIAS_TABLE"]),
        ("OPTIONAL_TABLE", vec!["OPTIONAL_TABLE"]),
        ("DIRECT_TABLE", vec!["DIRECT_TABLE"]),
    ] {
        let query = flow
            .calls
            .iter()
            .find(|call| {
                call.source_callee == "query"
                    && SOURCE[call.offset as usize..call.end as usize].contains(marker)
            })
            .unwrap();
        let call = SqlCall {
            file: file.clone(),
            span: (query.offset, query.end),
            call: &embedded,
        };
        let actual: BTreeSet<_> =
            super::super::interpolations::interpolations(&graph, &facts, &call, &bindings)
                .into_iter()
                .filter_map(|source| match source {
                    super::super::Provenance::ProcessEnv(key) => Some(key),
                    _ => None,
                })
                .collect();
        assert_eq!(
            actual,
            expected.into_iter().map(String::from).collect(),
            "{marker}"
        );
    }
}
