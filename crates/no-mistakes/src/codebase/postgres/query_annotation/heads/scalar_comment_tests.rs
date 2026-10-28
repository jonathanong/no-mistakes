use super::super::{collect_prepared, HeadCallSpans};
use crate::codebase::postgres::EmbeddedSqlOptions;
use crate::codebase::ts_source::facts::{
    collect_file_facts_from_program, TsFactContext, TsFactPlan,
};
use crate::codebase::ts_source::module_facts::{ModuleDiagnostic, ModuleSpan};
use std::sync::Arc;

#[test]
fn scalar_comment_prepared_ownership_gaps_never_create_head_proofs() {
    for name in ["work.mts", "ctor-namespace.mts"] {
        let path = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
            .join("scalar-comment-names/src")
            .join(name);
        let source = std::fs::read_to_string(&path).unwrap();
        crate::ast::with_program(&path, &source, |program, _| {
            let facts = collect_file_facts_from_program(
                &path,
                TsFactPlan {
                    imports: true,
                    module_bindings: true,
                    ..Default::default()
                },
                &TsFactContext::default(),
                &source,
                program,
                None,
                None,
            );
            let calls = crate::codebase::postgres::extract_embedded_sql_from_program(
                &path,
                program,
                &source,
                &EmbeddedSqlOptions::configured("@bench/db", &["read".into()]),
            )
            .call_spans;
            let collect = |facts: &_| {
                collect_prepared(
                    program,
                    facts,
                    HeadCallSpans {
                        confirmed: calls.iter().copied().collect(),
                        pending: Default::default(),
                    },
                    &Default::default(),
                    true,
                    false,
                    None,
                )
            };
            if name == "ctor-namespace.mts" {
                assert!(!collect(&facts).accepts(calls[0]));
                // Losing the other runtime import owner must not hide its default mutation.
                let mut missing = facts.clone();
                Arc::make_mut(missing.module_bindings.as_mut().unwrap())
                    .bindings
                    .retain(|binding| binding.name != "moduleSql");
                assert!(!collect(&missing).accepts(calls[0]));
                return;
            }
            assert!(collect(&facts).accepts(calls[0]));
            for needle in [
                "function positive",
                "append(label)",
                "sql`/*",
                "read(query)",
            ] {
                let start = source.find(needle).unwrap() as u32;
                let mut facts = facts.clone();
                Arc::make_mut(facts.module_bindings.as_mut().unwrap())
                    .diagnostics
                    .push(ModuleDiagnostic {
                        kind: "semanticError".into(),
                        message: "saved scalar ownership gap".into(),
                        span: Some(ModuleSpan {
                            start,
                            end: start + needle.len() as u32,
                        }),
                    });
                assert!(!collect(&facts).accepts(calls[0]), "{needle}");
            }
        })
        .unwrap();
    }
}
