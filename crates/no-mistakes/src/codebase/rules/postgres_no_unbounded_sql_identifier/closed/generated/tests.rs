use super::*;
use crate::codebase::check_facts::CheckFileFacts;
use crate::codebase::dependencies::graph::GraphBuildPlan;
use crate::codebase::ts_resolver::{load_tsconfig, normalize_path};
use crate::codebase::ts_source::facts::{collect_ts_facts, TsFactPlan};

#[test]
fn prepared_generated_returns_preserve_prefixes_async_results_and_negative_controls() {
    let root = normalize_path(
        &crate::test_support::rule_fixture_root("postgres-no-unbounded-sql-identifier")
            .join("issue-3073-generated"),
    );
    let files: Vec<_> = [
        "db.ts",
        "temp-names.ts",
        "writer.ts",
        "open-prefix.ts",
        "fake-uuid.ts",
        "negative-returns.ts",
        "raw.ts",
    ]
    .iter()
    .map(|file| root.join("src").join(file))
    .collect();
    let plan = TsFactPlan {
        value_flow: true,
        ..TsFactPlan::imports()
    };
    let prepared = collect_ts_facts(&files, plan);
    let mut facts = CheckFactMap {
        files: files.clone(),
        graph_plan: plan,
        ..Default::default()
    };
    for file in &files {
        let ts = prepared.get(file).unwrap();
        assert!(ts.parse_error.is_none(), "{}", file.display());
        facts.ts.insert(
            file.clone(),
            CheckFileFacts {
                ts: Arc::new(ts.clone()),
                ..Default::default()
            }
            .into(),
        );
    }
    let tsconfig = load_tsconfig(&root.join("tsconfig.json")).unwrap();
    let graph = DepGraph::build_with_plan_file_list_and_check_facts(
        &root,
        &tsconfig,
        GraphBuildPlan {
            calls: true,
            imports: true,
            ..Default::default()
        },
        files,
        &facts,
    )
    .unwrap();
    let bindings = Bindings::new(&facts);
    for (file, names, expected, count) in [
        ("writer.ts", &["table"][..], true, 2),
        ("open-prefix.ts", &["table"][..], false, 2),
        ("fake-uuid.ts", &["table"][..], false, 1),
        (
            "negative-returns.ts",
            &["mixed", "invalidated", "recursive", "unawaited"][..],
            false,
            4,
        ),
    ] {
        let file = root.join("src").join(file);
        let selected: Vec<_> = facts
            .get_ts_facts(&file)
            .unwrap()
            .value_flow
            .bindings
            .iter()
            .filter(|binding| names.contains(&binding.name.as_str()))
            .collect();
        assert_eq!(selected.len(), count, "{}", file.display());
        for binding in selected {
            assert_eq!(
                proves(&graph, &facts, &bindings, &file, binding),
                expected,
                "{}:{}:{}",
                file.display(),
                binding.name,
                binding.declared_at,
            );
        }
    }
}
