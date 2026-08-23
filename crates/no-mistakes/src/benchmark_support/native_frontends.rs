use crate::codebase::dotnet::{collect_dotnet_facts_with_sources, configured_projects};
use crate::codebase::swift::collect_swift_facts_with_sources;
use crate::codebase::ts_source::{discover_visible_paths, FileInventory, SourceStore};
use crate::config::v2::schema::DotnetConfig;
use std::path::PathBuf;
use std::sync::Arc;

/// Stable fixture inputs for the Swift and .NET production fact collectors.
pub struct NativeFrontendFixture {
    swift_root: PathBuf,
    swift_files: Vec<PathBuf>,
    swift_packages: Vec<String>,
    dotnet_root: PathBuf,
    dotnet_files: Vec<PathBuf>,
    dotnet_config: DotnetConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeFrontendSummary {
    pub files: usize,
    pub parsed_files: usize,
    pub physical_reads: usize,
}

pub fn native_frontend_fixture() -> NativeFrontendFixture {
    let test_cases =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../test-cases/codebase-analysis");
    let swift_root = normalize(test_cases.join("swift-test-plan/fixture"));
    let dotnet_root = normalize(test_cases.join("dotnet-test-plan/fixture"));
    NativeFrontendFixture {
        swift_files: discover_visible_paths(&swift_root),
        swift_root,
        swift_packages: vec!["swift-clients/core".into(), "swift-clients/ui".into()],
        dotnet_files: discover_visible_paths(&dotnet_root),
        dotnet_root,
        dotnet_config: DotnetConfig {
            solutions: vec!["dotnet-clients/App.sln".into()],
            ..DotnetConfig::default()
        },
    }
}

pub fn collect_swift_frontend_facts(fixture: &NativeFrontendFixture) -> NativeFrontendSummary {
    let sources = SourceStore::new(Arc::new(FileInventory::from_paths(&fixture.swift_files)));
    let facts = collect_swift_facts_with_sources(
        &fixture.swift_root,
        &fixture.swift_files,
        &fixture.swift_packages,
        Some(&sources),
    );
    NativeFrontendSummary {
        files: fixture.swift_files.len(),
        parsed_files: facts.files.len(),
        physical_reads: sources.physical_read_count(),
    }
}

pub fn collect_dotnet_frontend_facts(fixture: &NativeFrontendFixture) -> NativeFrontendSummary {
    let sources = SourceStore::new(Arc::new(FileInventory::from_paths(&fixture.dotnet_files)));
    let projects = configured_projects(&fixture.dotnet_root, &fixture.dotnet_config);
    let facts = collect_dotnet_facts_with_sources(
        &fixture.dotnet_root,
        &fixture.dotnet_files,
        &projects,
        Some(&sources),
    );
    NativeFrontendSummary {
        files: fixture.dotnet_files.len(),
        parsed_files: facts.files.len(),
        physical_reads: sources.physical_read_count(),
    }
}

fn normalize(path: PathBuf) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(&path)
}

#[cfg(test)]
#[path = "native_frontends/tests.rs"]
mod tests;
