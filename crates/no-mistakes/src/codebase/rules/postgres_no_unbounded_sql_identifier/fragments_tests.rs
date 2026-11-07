use super::*;
use crate::codebase::check_facts::CheckFileFacts;
use crate::codebase::postgres::EmbeddedSqlCall;
use crate::codebase::rules::postgres_sql_cross_file::{compile, Options};
use crate::codebase::ts_source::facts::{collect_ts_facts, TsFactPlan};
use std::sync::Arc;

#[test]
fn prepared_fragment_proofs_distinguish_tags_and_mutations() {
    let root = crate::test_support::rule_fixture_root("postgres-no-unbounded-sql-identifier")
        .join("issue-3073-fragments");
    let graph = crate::codebase::dependencies::graph::test_support::from_raw_maps(
        root.clone(),
        Default::default(),
        Default::default(),
    );
    for (file_name, expected) in [
        (
            "text.ts",
            vec![
                ("fragment", true),
                ("oldFragment", true),
                ("relation", true),
            ],
        ),
        ("joins.ts", vec![]),
        (
            "negative.ts",
            vec![
                ("fragment", false),
                ("altered", false),
                ("plainCall", false),
                ("nestedPush", false),
                ("nestedSplice", false),
                ("nestedWrite", false),
                ("nestedAlias", false),
                ("nestedComputed", false),
            ],
        ),
    ] {
        let file = root.join("src").join(file_name);
        let collected = collect_ts_facts(
            std::slice::from_ref(&file),
            TsFactPlan {
                imports: true,
                function_calls: true,
                value_flow: true,
                ..Default::default()
            },
        );
        let mut facts = CheckFactMap::default();
        facts.files.push(file.clone());
        facts.ts.insert(
            file.clone(),
            CheckFileFacts {
                ts: Arc::new(collected.get(&file).unwrap().clone()),
                ..Default::default()
            }
            .into(),
        );
        let bindings = Bindings::new(&facts);
        let opts = compile(
            RULE_ID,
            &serde_json::from_value::<Options>(serde_json::json!({
                "trustedSqlTags": [{"module": "@example/sql", "name": "sql"}]
            }))
            .unwrap(),
        )
        .unwrap();
        let embedded = EmbeddedSqlCall::default();
        let call = SqlCall {
            file: file.clone(),
            span: (0, u32::MAX),
            call: &embedded,
        };
        let flow = &facts.get_ts_facts(&file).unwrap().value_flow;
        for (name, expected) in expected {
            assert!(
                flow.bindings.iter().any(|binding| binding.name == name),
                "{file_name}: missing {name}"
            );
            for binding in flow.bindings.iter().filter(|binding| binding.name == name) {
                let reference = Provenance::Reference {
                    name: name.into(),
                    scope: binding.scope,
                    offset: u32::MAX,
                    members: vec!["text".into()],
                };
                assert_eq!(
                    fragments::proven(&graph, &opts, &facts, &bindings, &call, &reference),
                    expected,
                    "{file_name}: {name}"
                );
            }
        }
        if file_name == "joins.ts" {
            assert!(flow
                .sql_placeholder_lists
                .iter()
                .any(|(_, name, _)| name == "conditions"));
            assert!(flow
                .bounded_bindings
                .iter()
                .any(|binding| binding.name == "where"));
        } else if file_name == "negative.ts" {
            assert!(!flow
                .sql_placeholder_lists
                .iter()
                .any(|(_, name, _)| name == "statements"));
            assert!(flow
                .registry_mutations
                .iter()
                .any(|binding| binding.name == "changed"));
        }
    }
}
