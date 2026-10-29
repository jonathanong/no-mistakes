use crate::codebase::postgres::embedded::helpers::program::{Expr, Program, Stmt};
use crate::codebase::postgres::sql_identifier_syntax::helper_recovery::{
    budget::Limits,
    catalog::PreparedCatalog,
    values::{Cache, Engine},
};
use crate::codebase::ts_source::facts::{
    collect_file_facts_from_program, TsFactContext, TsFactPlan,
};
use crate::codebase::{check_facts::CheckFileFacts, ts_source::FileIdMap};
use std::sync::Arc;

#[test]
fn irrelevant_arguments_keep_skeletons_and_references_without_full_ir() {
    let path = crate::test_support::rule_fixture_root("postgres-no-offset")
        .join("low-model-demand/src/model-demand.mts");
    let source = std::fs::read_to_string(&path).unwrap();
    crate::ast::with_program(&path, &source, |program, _| {
        let ts = collect_file_facts_from_program(
            &path,
            TsFactPlan {
                module_bindings: true,
                ..Default::default()
            },
            &TsFactContext::default(),
            &source,
            program,
            None,
            None,
        );
        let facts = super::super::collect(program, &ts, &source, &[super::tests::profile()]);
        let models = &facts.recovery;
        let call = |marker: &str| {
            let line = source
                .lines()
                .position(|text| text.contains(marker))
                .unwrap() as u32
                + 1;
            models
                .calls
                .iter()
                .find(|call| {
                    crate::codebase::ts_source::byte_offset_to_line(&source, call.span.0 as usize)
                        == line
                })
                .unwrap()
        };
        let body = |marker| call(marker).arguments[0].program.body.as_slice();
        assert!(matches!(
            body("unrelated({"),
            [Stmt::Return(Some(Expr::Opaque), _)]
        ));
        assert!(matches!(
            body("configured sink keeps full SQL"),
            [Stmt::Return(Some(Expr::Lit(_)), _)]
        ));
        assert!(matches!(
            body("selected executor forwarding"),
            [Stmt::Return(Some(Expr::Ident(_)), _)]
        ));
        for marker in [
            "opaque escape keeps its reference",
            "opaque throw call keeps its skeleton",
        ] {
            assert!(matches!(
                body(marker),
                [Stmt::Return(Some(Expr::Opaque), _)]
            ));
            assert_eq!(call(marker).arguments[0].references.len(), 1);
        }
        assert!(matches!(
            body("escaped local cannot prove SQL"),
            [Stmt::Return(Some(Expr::Member(_, _)), _)]
        ));
        assert!(models.calls.iter().any(|call| {
            source[call.span.0 as usize..call.span.1 as usize].starts_with("unrelated(local)")
                && models
                    .argument_owners
                    .values()
                    .any(|owner| models.calls[*owner].span == call.span)
        }));
        assert!(models.functions.values().any(|function| {
            source[function.span.0 as usize..function.span.1 as usize]
                .contains("throw unrelated(local)")
        }));
        let unknown = facts
            .profiles
            .iter()
            .flat_map(|(_, calls)| calls)
            .find(|call| {
                source[call.span.0 as usize..call.span.1 as usize].starts_with("read(local.sql)")
            })
            .unwrap();
        assert!(matches!(unknown.input, super::super::Input::Unknown));
        assert_eq!(
            unknown.line,
            source
                .lines()
                .position(|line| line.contains("read(local.sql)"))
                .unwrap() as u32
                + 1
        );
        let opaque = call("unrelated({").arguments[0].clone();
        let mut files = FileIdMap::default();
        files.insert(
            path.clone(),
            CheckFileFacts {
                ts: Arc::new(ts),
                sql_identifier_syntax: facts,
                ..Default::default()
            },
        );
        let catalog = PreparedCatalog {
            files: &files,
            application: None,
            profile: None,
            resolve: &|_, _| None,
        };
        let mut cache = Cache::default();
        let mut engine = Engine::new(&catalog, &mut cache, Limits::default());
        assert!(engine
            .evaluate_at(&path, &opaque, None)
            .is_none_or(|values| values.is_empty()));
        assert!(engine.residual(), "opaque argument must not prove a value");
    })
    .unwrap();
}

#[test]
fn opaque_argument_programs_share_only_within_one_parsed_file() {
    let root =
        crate::test_support::rule_fixture_root("postgres-no-offset").join("low-model-demand/src");
    let collect = |name: &str| {
        let path = root.join(name);
        let source = std::fs::read_to_string(&path).unwrap();
        crate::ast::with_program(&path, &source, |program, _| {
            let ts = collect_file_facts_from_program(
                &path,
                TsFactPlan {
                    module_bindings: true,
                    ..Default::default()
                },
                &TsFactContext::default(),
                &source,
                program,
                None,
                None,
            );
            super::super::collect(program, &ts, &source, &[super::tests::profile()])
        })
        .unwrap()
    };
    let first = collect("model-demand.mts");
    let second = collect("model-demand-second.mts");
    let repeated = collect("model-demand.mts");
    let first_programs = opaque_programs(&first);
    let second_programs = opaque_programs(&second);
    let repeated_programs = opaque_programs(&repeated);
    assert!(first_programs.len() >= 2 && second_programs.len() >= 2);
    assert!(Arc::ptr_eq(first_programs[0], first_programs[1]));
    assert!(Arc::ptr_eq(second_programs[0], second_programs[1]));
    assert!(!Arc::ptr_eq(first_programs[0], second_programs[0]));
    assert!(!Arc::ptr_eq(first_programs[0], repeated_programs[0]));
    // The shared opaque Return still fails proof; each call keeps its own references.
    assert!(first
        .recovery
        .calls
        .iter()
        .any(|call| call.arguments.iter().any(|arg| {
            Arc::ptr_eq(&arg.program, first_programs[0]) && !arg.references.is_empty()
        })));
}

fn opaque_programs(facts: &super::super::FileFacts) -> Vec<&Arc<Program>> {
    facts
        .recovery
        .calls
        .iter()
        .flat_map(|call| &call.arguments)
        .filter(|arg| {
            matches!(
                arg.program.body.as_slice(),
                [Stmt::Return(Some(Expr::Opaque), _)]
            )
        })
        .map(|arg| &arg.program)
        .collect()
}
