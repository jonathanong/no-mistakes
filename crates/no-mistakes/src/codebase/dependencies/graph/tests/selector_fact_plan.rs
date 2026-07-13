use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};

struct CountingFacts {
    facts: TsFactMap,
    graph_files: Vec<PathBuf>,
    lookups: AtomicUsize,
}

impl TsFactLookup for CountingFacts {
    fn get_ts_facts(&self, path: &Path) -> Option<&TsFileFacts> {
        self.lookups.fetch_add(1, Ordering::Relaxed);
        self.facts.get(path)
    }

    fn covers_ts_fact_plan(&self, required: TsFactPlan) -> bool {
        self.facts.plan().covers(required)
    }

    fn graph_files(&self) -> Option<&[PathBuf]> {
        Some(&self.graph_files)
    }
}

fn fixture_root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/codebase/dependencies/selector-text-sparse-universe/fixture"),
    )
}

#[test]
fn selector_only_plan_does_not_request_ts_facts() {
    let plan = GraphBuildPlan {
        playwright_selectors: true,
        ..GraphBuildPlan::default()
    };

    assert!(plan.ts_fact_plan().is_empty());
    assert!(effective_ts_fact_plan(plan, None).is_empty());
}

#[test]
fn text_locator_selector_only_graph_lazily_reads_import_facts() {
    let root = fixture_root();
    let graph_files = GraphFiles::discover(&root);
    let facts = CountingFacts {
        facts: collect_ts_facts(graph_files.indexable(), TsFactPlan::imports()),
        graph_files: graph_files.all().to_vec(),
        lookups: AtomicUsize::new(0),
    };
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths_dir: root.clone(),
        ..TsConfig::default()
    };

    let graph = DepGraph::build_with_plan_files_config_and_facts(
        &root,
        &tsconfig,
        GraphBuildPlan {
            playwright_selectors: true,
            ..GraphBuildPlan::default()
        },
        &graph_files,
        None,
        Some(&facts),
    )
    .expect("selector graph builds");

    assert!(facts.lookups.load(Ordering::Relaxed) > 0);
    let dependencies = graph.deps_of(
        &[NodeId::File(root.join("tests/e2e/app.spec.ts"))],
        None,
        Some(&HashSet::from([EdgeKind::Selector])),
    );
    assert!(dependencies.iter().any(|entry| {
        entry.node == NodeId::File(root.join("web/app/components/discuss-button.tsx"))
    }));
}
