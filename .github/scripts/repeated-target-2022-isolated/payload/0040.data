//! Shared unmodified-production-library harness; no report formatting in graph loops.
use no_mistakes::codebase::dependencies::{
    graph::{DepGraph, GraphBuildPlan},
    TsConfig,
};
use no_mistakes::codebase::ts_source::facts::{collect_ts_facts, TsFactPlan};
use no_mistakes::diagnostics::{with_observer, InvocationObserver};
use std::{hint::black_box, path::PathBuf};
fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(args.len(), 4, "MODE INPUT ITERATIONS");
    let input = PathBuf::from(&args[2]).canonicalize().unwrap();
    let count: usize = args[3].parse().unwrap();
    assert!(count > 0);
    let mut signature = None;
    for _ in 0..count {
        let observed = if args[1].starts_with("facts-") {
            let facts = collect_ts_facts(
                std::slice::from_ref(&input),
                TsFactPlan {
                    imports: true,
                    function_calls: args[1] == "facts-values",
                    value_flow: args[1] == "facts-values",
                    ..Default::default()
                },
            );
            let file = facts.get(&input).unwrap();
            assert!(file.operational_error.is_none() && file.parse_error.is_none());
            let value =
                file.function_calls.len() + file.imports.len() + file.value_flow.bindings.len();
            black_box(facts);
            value
        } else {
            assert!(matches!(
                args[1].as_str(),
                "graph" | "graph-no-calls" | "graph-observer"
            ));
            let config = TsConfig {
                dir: input.clone(),
                paths: vec![],
                paths_dir: input.clone(),
                base_url: None,
            };
            let observer = (args[1] == "graph-observer").then(|| InvocationObserver::new(true));
            let graph = with_observer(observer, || {
                DepGraph::build_with_plan(
                    &input,
                    &config,
                    GraphBuildPlan {
                        imports: true,
                        calls: args[1] != "graph-no-calls",
                        ..Default::default()
                    },
                )
                .unwrap()
            });
            let value = graph.resolved_call_sites().len();
            black_box(graph);
            value
        };
        if let Some(prior) = signature {
            assert_eq!(prior, observed);
        }
        signature = Some(observed);
    }
    println!("{}", signature.unwrap());
}
