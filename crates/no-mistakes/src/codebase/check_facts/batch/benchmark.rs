use super::{demands, test_support, BatchCheckFactRequest, CheckFactPlan};
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(crate) struct DemandBuilderFixture {
    requests: Vec<BatchCheckFactRequest>,
}

pub(crate) fn demand_builder_fixture(
    root: &Path,
    files: Vec<PathBuf>,
    graph_files: Vec<PathBuf>,
    plan: CheckFactPlan,
) -> DemandBuilderFixture {
    let inventory = Arc::new(crate::codebase::ts_source::FileInventory::from_paths(
        &files,
    ));
    let sources = Arc::new(crate::codebase::ts_source::SourceStore::new(inventory));
    DemandBuilderFixture {
        requests: vec![BatchCheckFactRequest {
            root: root.to_path_buf(),
            files,
            graph_files,
            plan,
            playwright: None,
            sources,
        }],
    }
}

pub(crate) fn run_demand_builder(
    fixture: &DemandBuilderFixture,
    historical: bool,
    count: bool,
) -> (usize, usize, usize) {
    match (historical, count) {
        (true, true) => run::<true>(fixture, true),
        (true, false) => run::<false>(fixture, true),
        (false, true) => run::<true>(fixture, false),
        (false, false) => run::<false>(fixture, false),
    }
}

fn run<const COUNT: bool>(
    fixture: &DemandBuilderFixture,
    historical: bool,
) -> (usize, usize, usize) {
    let mut clones = 0;
    if historical {
        let demands = test_support::demands_by_path::<COUNT>(&fixture.requests, &mut clones);
        std::hint::black_box(&demands);
        (
            demands.len(),
            demands
                .values()
                .flatten()
                .map(|demand| {
                    std::hint::black_box(&demand.plan);
                    std::hint::black_box(demand.request);
                    1usize
                })
                .sum(),
            clones,
        )
    } else {
        let mut graph_plans = demands::graph_plans(&fixture.requests);
        let demands =
            demands::demands_by_path::<COUNT>(&fixture.requests, &mut graph_plans, &mut clones);
        std::hint::black_box(&demands);
        (
            demands.len(),
            demands
                .values()
                .flatten()
                .map(|demand| {
                    std::hint::black_box(&demand.plan);
                    std::hint::black_box(demand.request);
                    1usize
                })
                .sum(),
            clones,
        )
    }
}
