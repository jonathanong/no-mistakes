//! Feature-only adapters for the complete request-local demand builder.
use crate::codebase::check_facts::{CheckFactPlan, DemandBuilderFixture};
use std::path::{Path, PathBuf};

pub struct FactDemandFixture(DemandBuilderFixture);

pub fn prepare_fact_demands(
    root: &Path,
    files: Vec<PathBuf>,
    graph_files: Vec<PathBuf>,
    plan: CheckFactPlan,
) -> FactDemandFixture {
    FactDemandFixture(crate::codebase::check_facts::demand_builder_fixture(
        root,
        files,
        graph_files,
        plan,
    ))
}

/// Returns (distinct paths, variants, actual full-plan clones).
/// Count-disabled specializations compile clone accounting out of the builder.
pub fn construct_fact_demands(
    fixture: &FactDemandFixture,
    historical: bool,
    count: bool,
) -> (usize, usize, usize) {
    crate::codebase::check_facts::run_demand_builder(&fixture.0, historical, count)
}
