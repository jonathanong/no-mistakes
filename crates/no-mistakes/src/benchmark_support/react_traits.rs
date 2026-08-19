use std::path::PathBuf;

/// Many components in one file so fused trait walks beat per-component visits.
#[derive(Clone)]
pub struct ReactTraitsFixture {
    pub root: PathBuf,
    pub file: PathBuf,
}

/// Stable counts from file-level React trait analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ReactTraitsSummary {
    pub components: usize,
    pub with_state: usize,
    pub with_props: usize,
    pub with_memo: usize,
    pub with_context: usize,
    pub with_suspense: usize,
    pub with_fetch: usize,
    pub with_children: usize,
}

pub fn react_traits_many_components_fixture() -> ReactTraitsFixture {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/performance/react-traits"),
    );
    ReactTraitsFixture {
        file: root.join("many-components.tsx"),
        root,
    }
}

pub fn analyze_react_traits_file(fixture: &ReactTraitsFixture) -> ReactTraitsSummary {
    let analysis = crate::react_traits::analyze::file::analyze_file(&fixture.file, &fixture.root)
        .unwrap_or_else(|error| {
            panic!(
                "Failed to analyze React trait benchmark fixture '{}': {error}. \
                 Fix the fixture syntax or analyzer support. The benchmark requires stable \
                 trait counts to measure file-level analysis.",
                fixture.file.display()
            )
        });
    let mut summary = ReactTraitsSummary {
        components: analysis.components.len(),
        ..ReactTraitsSummary::default()
    };
    for component in analysis.components.iter() {
        summary.with_state += usize::from(component.has_state);
        summary.with_props += usize::from(component.has_props);
        summary.with_memo += usize::from(component.uses_memo);
        summary.with_context += usize::from(component.uses_context_provider);
        summary.with_suspense += usize::from(component.uses_suspense);
        summary.with_fetch += usize::from(!component.fetches.is_empty());
        summary.with_children += usize::from(!component.children.is_empty());
    }
    summary
}
