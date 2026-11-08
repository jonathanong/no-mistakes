use super::super::*;
use crate::codebase::ts_source::{
    facts::{TsFactPlan, TsFileFacts},
    SourceStore,
};
use crate::config::v2::Unanalyzable;
use std::path::PathBuf;

struct FailedFacts<'a> {
    base: &'a dyn TsFactLookup,
    path: PathBuf,
    failure: TsFileFacts,
}
impl TsFactLookup for FailedFacts<'_> {
    fn get_ts_facts(&self, path: &Path) -> Option<&TsFileFacts> {
        if path == self.path {
            Some(&self.failure)
        } else {
            self.base.get_ts_facts(path)
        }
    }
    fn covers_ts_fact_plan(&self, plan: TsFactPlan) -> bool {
        self.base.covers_ts_fact_plan(plan)
    }
    fn graph_files(&self) -> Option<&[PathBuf]> {
        self.base.graph_files()
    }
}

pub(super) fn assert_policies(
    root: &Path,
    config: &NoMistakesConfig,
    facts: &dyn TsFactLookup,
    sources: &SourceStore,
    build: impl Fn(&dyn TsFactLookup) -> DepGraph,
) {
    for path in ["src/producers.mts", "src/assertions.mts"] {
        for fatal_only in [false, true] {
            // Fault injection replaces a saved fixture's prepared failure entry;
            // no permissions tricks, filesystem fixtures, or second parser are needed.
            let failed = FailedFacts {
                base: facts,
                path: root.join(path),
                failure: TsFileFacts {
                    operational_error: (!fatal_only)
                        .then(|| "saved fixture source read failed".into()),
                    fatal_parse_error: fatal_only,
                    ..Default::default()
                },
            };
            let before = sources.physical_read_count();
            let graph = build(&failed);
            for from in graph.all_files() {
                for (to, kind) in graph.dependencies_of_node(from).into_iter().flatten() {
                    assert!(
                        *kind != EdgeKind::AssertionOwner
                            || (from.as_file() != Some(failed.path.as_path())
                                && to.as_file() != Some(failed.path.as_path()))
                    );
                }
            }
            for policy in [
                Unanalyzable::Report,
                Unanalyzable::Advise,
                Unanalyzable::Ignore,
            ] {
                let mut configured = config.clone();
                configured.unanalyzable = Some(policy);
                let error = check(root, &configured, &graph, &failed, sources)
                    .err()
                    .expect("failed source must remain an error");
                let input = error
                    .downcast_ref::<crate::codebase::check_outcome::CheckInputFailure>()
                    .expect("typed source failure");
                assert_eq!(input.file.as_ref(), Some(&failed.path));
                assert!(input.message.contains("source failure in"));
                assert!(input.message.contains(path));
                if !fatal_only {
                    assert!(input.message.contains("saved fixture source read failed"));
                }
            }
            assert_eq!(
                sources.physical_read_count(),
                before,
                "faults must stay on the prepared facts path"
            );
        }
    }
}
