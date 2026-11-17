use super::alias_templates::DetachedTemplateBinds;
use super::{Bindings, Provenance};
use crate::codebase::check_facts::{CheckFactMap, CheckFileFacts};
use crate::codebase::dependencies::graph::TsFactLookup;
use crate::codebase::postgres::EmbeddedSqlCall;
use crate::codebase::rules::postgres_sql_cross_file::SqlCall;
use crate::codebase::ts_source::facts::{collect_ts_facts, TsFactPlan};
use std::collections::BTreeMap;
use std::sync::Arc;

#[test]
fn immutable_template_aliases_keep_bind_pairing_and_scoped_builder_identity() {
    let root = crate::test_support::rule_fixture_root("postgres-no-unbounded-sql-identifier");
    let file = root.join("issue-3525/unit/query.ts");
    let collected = collect_ts_facts(
        std::slice::from_ref(&file),
        TsFactPlan {
            value_flow: true,
            identifier_provenance: true,
            ..TsFactPlan::imports()
        },
    );
    let mut facts = CheckFactMap::default();
    facts.files.push(file.clone());
    facts.ts.insert(
        file.clone(),
        Arc::new(CheckFileFacts {
            ts: Arc::new(collected[&file].clone()),
            ..Default::default()
        }),
    );
    let bindings = Bindings::new(&facts);
    let mut proof = DetachedTemplateBinds::new(&bindings);
    let calls = &facts.get_ts_facts(&file).unwrap().value_flow.calls;
    let found: BTreeMap<_, _> = calls
        .iter()
        .filter(|call| call.source_callee.starts_with("read"))
        .map(|call| {
            let embedded = EmbeddedSqlCall::default();
            let source = proof.detached(&SqlCall {
                file: file.clone(),
                span: (call.offset, call.end),
                call: &embedded,
            });
            let name = source.map(|source| {
                let Provenance::Reference { name, members, .. } = source else {
                    panic!("expected local builder");
                };
                assert_eq!(members, ["text"]);
                name
            });
            (call.source_callee.as_str(), name)
        })
        .collect();
    let repeated = calls
        .iter()
        .find(|call| call.source_callee == "readMissing")
        .unwrap();
    assert!(proof
        .detached(&SqlCall {
            file: file.clone(),
            span: (repeated.offset, repeated.end),
            call: &EmbeddedSqlCall::default(),
        })
        .is_some());
    assert_eq!(
        found,
        BTreeMap::from([
            ("readMissing", Some("q".into())),
            ("readMatched", None),
            ("readOptions", None),
            ("readValuesAlias", None),
            ("readLaterSpread", None),
            ("readEarlySpread", Some("q".into())),
            ("readWrong", Some("q".into())),
            ("readMulti", Some("q".into())),
            ("readBuilderAlias", None),
            ("readFree", None),
            ("readDirect", None),
            ("readConstant", Some("literal".into())),
            ("readMutable", None),
            ("readCycle", None),
            ("readAbsent", None),
            ("readShadow", Some("q".into())),
            ("readMutation", None),
            ("readAliasWrite", None),
            ("readAliasCall", None),
            ("readEscape", None),
            ("readWrappedEscape", None),
            ("readMemberReads", Some("q".into())),
            ("readExported", None),
            ("readExportedAlias", None),
            ("readExportedObject", None),
            ("readReturned", None),
        ])
    );
}
