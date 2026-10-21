use super::*;
use crate::codebase::check_facts::CheckFileFacts;
use crate::codebase::ts_source::facts::{collect_ts_facts, TsFactPlan};
use std::sync::Arc;

#[cfg(feature = "test-instrumentation")]
#[test]
fn alias_diamond_search_visits_input_edges_once() {
    for (scenario, expected) in [
        ("pass-diamond-alias", None),
        ("fail-diamond-alias", Some("SCHEMA")),
    ] {
        let file = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-no-unbounded-sql-identifier/fixture")
            .join(scenario)
            .join("src/query.ts");
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
        let binding = bindings.value(&file, 0, "a18", u32::MAX).unwrap();
        let mut state = Search::default();
        assert_eq!(
            bindings
                .search_env(&file, &binding.value, &[], &mut state)
                .as_deref(),
            expected
        );
        assert!(state.work <= 38, "{scenario}: {} binding edges", state.work);
    }
}
