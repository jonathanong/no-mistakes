use super::{relative_path, Confidence, ImpactReason, SelectedTest, TestPlan, Warning};
use crate::tests::configured_plan::dep_triggers::{framework_plan, test_runner};
use crate::tests::configured_plan_candidates::merge_selected;
use crate::tests::prepared_plan::PreparedTestPlanRequest;
use crate::tests::{TestFramework, TestPlanGroupResult};
use clap::ValueEnum;
use no_mistakes::changes::ChangesReport;
use no_mistakes::codebase::test_discovery::DiscoveredTests;
use no_mistakes::codebase::ts_source::{FixtureReadIndex, SourceStore};
use no_mistakes::config::v2::NoMistakesConfig;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

mod attributes;
mod matching;
mod selection;
#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod tests;
use matching::CategoryMatches;
pub(super) use selection::apply;

pub(crate) struct ChangesInput {
    file: String,
    pub(crate) report: Result<ChangesReport, String>,
    fixture_reads: Option<Arc<FixtureReadIndex>>,
}

impl ChangesInput {
    pub(crate) fn load_from_parts(
        args: &super::PlanArgs,
        root: &Path,
        config: &NoMistakesConfig,
        sources: &SourceStore,
    ) -> Option<Self> {
        let path = no_mistakes::config::resolve(root, args.changes.as_ref()?);
        let file = relative_path(root, &path);
        let report = sources
            .read_path(&path)
            .map_err(|_| format!("Cannot read changes report `{file}`"))
            .and_then(|source| {
                serde_json::from_str::<ChangesReport>(&source)
                    .map_err(|error| format!("Invalid changes report `{file}`: {error}"))
            })
            .and_then(|report| validate_report(report, config));
        let fixture_reads = report
            .as_ref()
            .ok()
            .and_then(|report| attributes::index(root, config, report));
        Some(Self {
            file,
            report,
            fixture_reads,
        })
    }

    pub(crate) fn fixture_reads(&self) -> Option<&Arc<FixtureReadIndex>> {
        self.fixture_reads.as_ref().filter(|_| self.report.is_ok())
    }

    pub(super) fn has_work(&self) -> bool {
        self.report.as_ref().map_or(true, |report| {
            !report.changed_files.is_empty()
                || report.categories.values().any(|ids| {
                    !(ids.changed.is_empty() && ids.added.is_empty() && ids.removed.is_empty())
                })
        })
    }
}

fn validate_report(
    report: ChangesReport,
    config: &NoMistakesConfig,
) -> Result<ChangesReport, String> {
    if report.schema_version != 1 {
        return Err(format!(
            "Unsupported changes report schema version {}",
            report.schema_version
        ));
    }
    if !report.complete {
        let mut message = "Changes report is incomplete".to_string();
        if !report.reasons.is_empty() {
            message.push_str(&format!(": {}", report.reasons.join("; ")));
        }
        return Err(message);
    }
    if !report.unclassified.is_empty() {
        return Err(format!(
            "Changes report has unclassified paths: {}",
            report.unclassified.join(", ")
        ));
    }
    if let Some(name) = report
        .categories
        .keys()
        .find(|name| !config.changes.categories.contains_key(*name))
    {
        return Err(format!("Changes report uses undeclared category `{name}`"));
    }
    Ok(report)
}

pub(super) fn unplanned_changed_file_reason(
    prepared: &PreparedTestPlanRequest,
    framework: Option<TestFramework>,
    warnings: &[Warning],
) -> Option<String> {
    let config_fallback = match framework {
        Some(fw) => prepared.framework_config_trigger(fw).or_else(|| {
            super::global_config::excluding_v2_config(
                &prepared.root,
                &prepared.collected.files,
                Some(fw),
                prepared,
            )
        }),
        None => super::global_config::global_config_trigger(
            &prepared.root,
            &prepared.collected.files,
            None,
            prepared,
        ),
    };
    if let Some((reason, _)) = config_fallback {
        return Some(reason);
    }
    if let Some(deleted) = prepared.collected.deleted.first() {
        return Some(format!(
            "Deleted file in changes report: {}",
            relative_path(&prepared.root, deleted)
        ));
    }
    warnings.iter().find_map(|warning| {
        matches!(
            warning.r#type.as_str(),
            "fallback-warning"
                | "deleted-file"
                | "lockfile-no-baseline"
                | "lockfile-binary-unsupported"
                | "lockfile-pnpm-unmodeled-installation-section"
                | "package-manifest-no-baseline"
                | "package-dependency-untraceable"
                | "dotnet-dependency-no-baseline"
                | "swift-manifest-no-baseline"
                | "swift-resolved-no-baseline"
        )
        .then(|| warning.message.clone())
    })
}

pub(super) fn apply_union(
    prepared: &PreparedTestPlanRequest,
    input: &ChangesInput,
    plan: &mut TestPlan,
) -> anyhow::Result<()> {
    let mut all_tests: BTreeSet<PathBuf> =
        super::global_config::discover_all_tests_from_prepared(prepared)
            .into_iter()
            .collect();
    let mut targets_by_path: BTreeMap<_, Vec<_>> = BTreeMap::new();
    for &framework in TestFramework::value_variants() {
        let discovered = prepared.discover_tests(framework)?;
        all_tests.extend(discovered.tests);
        for (path, targets) in discovered.targets_by_path {
            targets_by_path.entry(path).or_default().extend(targets);
        }
    }
    let discovered = DiscoveredTests {
        tests: all_tests.into_iter().collect(),
        targets_by_path,
        used_fallback: false,
    };
    apply(prepared, input, None, discovered, plan);
    Ok(())
}
