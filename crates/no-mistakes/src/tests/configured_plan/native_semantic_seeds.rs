use super::lockfile_seeds::{
    dotnet_dependency_seed_candidates, swift_manifest_seed_candidates,
    swift_resolved_seed_candidates,
};
use crate::tests::prepared_plan::PreparedTestPlanRequest;
use crate::tests::{warning_key, SelectedTest, TestFramework, Warning, WarningKey};
use no_mistakes::codebase::dependencies::graph::DepGraph;
use std::collections::HashSet;
use std::path::Path;

pub(crate) struct NativeSemanticSeedResult {
    pub(crate) candidates: Vec<SelectedTest>,
    untraceable: Vec<NativeSemanticArtifact>,
}

struct NativeSemanticArtifact {
    file: String,
    warning_type: &'static str,
}

impl NativeSemanticSeedResult {
    pub(crate) fn warnings(&self) -> impl Iterator<Item = Warning> + '_ {
        self.untraceable.iter().map(|artifact| Warning {
            r#type: artifact.warning_type.to_string(),
            message: format!(
                "`{}` changed a dependency without a causal path to a configured test; full-suite selection requires global fallback opt-in",
                artifact.file
            ),
            file: artifact.file.clone(),
            line: None,
        })
    }

    pub(crate) fn first_untraceable(&self) -> Option<&str> {
        self.untraceable
            .first()
            .map(|artifact| artifact.file.as_str())
    }

    pub(crate) fn extend_warnings(
        &self,
        warnings: &mut Vec<Warning>,
        seen: &mut HashSet<WarningKey>,
    ) {
        for warning in self.warnings() {
            if seen.insert(warning_key(&warning)) {
                warnings.push(warning);
            }
        }
    }
}

pub(crate) fn native_semantic_seed_candidates(
    root: &Path,
    prepared: &PreparedTestPlanRequest,
    graph: &DepGraph,
    all_test_set: &HashSet<std::path::PathBuf>,
    framework: Option<TestFramework>,
) -> NativeSemanticSeedResult {
    let mut candidates = Vec::new();
    let mut untraceable = Vec::new();
    for seed in prepared
        .swift_resolved_analysis
        .seeds
        .iter()
        .filter(|_| matches!(framework, None | Some(TestFramework::Swift)))
    {
        let seeded =
            swift_resolved_seed_candidates(root, std::slice::from_ref(seed), graph, all_test_set);
        record_seed(
            &mut candidates,
            &mut untraceable,
            seeded,
            &seed.0,
            "swift-dependency-untraceable",
            root,
        );
    }
    for manifest in prepared
        .swift_manifest_analysis
        .seeds
        .iter()
        .filter(|_| matches!(framework, None | Some(TestFramework::Swift)))
    {
        let seeded = swift_manifest_seed_candidates(
            root,
            std::slice::from_ref(manifest),
            graph,
            all_test_set,
        );
        record_seed(
            &mut candidates,
            &mut untraceable,
            seeded,
            manifest,
            "swift-dependency-untraceable",
            root,
        );
    }
    for artifact in prepared
        .dotnet_dependency_analysis
        .artifacts
        .iter()
        .filter(|_| matches!(framework, None | Some(TestFramework::Dotnet)))
    {
        let seeded = dotnet_dependency_seed_candidates(
            root,
            std::slice::from_ref(artifact),
            prepared.dotnet_facts(),
            prepared.root_visible_paths(),
            graph,
            all_test_set,
        );
        record_seed(
            &mut candidates,
            &mut untraceable,
            seeded,
            &artifact.path,
            "dotnet-dependency-untraceable",
            root,
        );
    }
    NativeSemanticSeedResult {
        candidates,
        untraceable,
    }
}

fn record_seed(
    candidates: &mut Vec<SelectedTest>,
    untraceable: &mut Vec<NativeSemanticArtifact>,
    seeded: Vec<SelectedTest>,
    path: &Path,
    warning_type: &'static str,
    root: &Path,
) {
    if seeded.is_empty() {
        untraceable.push(NativeSemanticArtifact {
            file: crate::tests::plan::relative_path(root, path),
            warning_type,
        });
    }
    candidates.extend(seeded);
}
