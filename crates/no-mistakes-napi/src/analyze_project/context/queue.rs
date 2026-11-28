use super::{
    cached_analysis, canonical_filter_key, json_raw, CachedAnalysis, PreparedScope, ReportCache,
};
use crate::{options::ProjectOptions, queue::parse_queue_direction};
use anyhow::{bail, Result};
use no_mistakes::queue::{
    analyze_project_with_prepared_facts_and_catalog_and_session,
    analyze_project_with_prepared_facts_indexed_and_catalog_and_session, CheckReport,
    PreparedProjectReport, ProjectReport,
};
use serde_json::value::RawValue;

#[derive(Default)]
pub(super) struct QueueReports {
    pub(super) plain: ReportCache<ProjectReport>,
    pub(super) indexed: ReportCache<PreparedProjectReport>,
}

impl PreparedScope {
    pub(super) fn queue_report(
        &self,
        report_type: &str,
        options: &ProjectOptions,
    ) -> Result<Box<RawValue>> {
        let key = canonical_filter_key(&options.filters)?;
        let traversal = matches!(report_type, "queueEdges" | "queueRelated")
            || self.queue_traversal_keys.contains(&key);
        let root = self.traversal.root();
        let tsconfig_catalog = self.traversal.tsconfig_catalog();
        let config = self.traversal.config();
        let session = self.traversal.session_arc();
        let facts = &self.facts;
        let report = cached_analysis(
            &self.queue_reports.plain,
            &self.queue_reports.indexed,
            &key,
            traversal,
            &|| {
                analyze_project_with_prepared_facts_and_catalog_and_session(
                    root,
                    tsconfig_catalog,
                    config,
                    &options.filters,
                    facts,
                    &session,
                )
            },
            &|| {
                analyze_project_with_prepared_facts_indexed_and_catalog_and_session(
                    root,
                    tsconfig_catalog,
                    config,
                    &options.filters,
                    facts,
                    &session,
                )
            },
        );
        let report = report?;
        match report {
            CachedAnalysis::Plain(report) => {
                render_queue_report(report_type, options, &report, None)
            }
            CachedAnalysis::Indexed(indexed) => {
                let traversal_report = matches!(report_type, "queueEdges" | "queueRelated");
                render_queue_report(
                    report_type,
                    options,
                    indexed.report(),
                    traversal_report.then_some(&indexed),
                )
            }
        }
    }
}

fn render_queue_report(
    report_type: &str,
    options: &ProjectOptions,
    report: &ProjectReport,
    indexed: Option<&PreparedProjectReport>,
) -> Result<Box<RawValue>> {
    match report_type {
        "queues" => Ok(json_raw(report)),
        "queueEdges" => {
            let depth = no_mistakes::cli::root_scoped_edge_depth(&options.files, options.depth);
            Ok(json_raw(
                &indexed
                    .expect("typed queue traversal is prepared")
                    .edge_view(&options.files, depth),
            ))
        }
        "queueRelated" => {
            if options.files.is_empty() {
                bail!("files must contain at least one file");
            }
            let direction = parse_queue_direction(options.direction.as_deref())?;
            Ok(json_raw(
                &indexed
                    .expect("typed queue traversal is prepared")
                    .related(&options.files, direction),
            ))
        }
        "queueCheck" if options.include_diagnostics => Ok(json_raw(&CheckReport {
            findings: &report.check,
            diagnostics: &report.diagnostics,
        })),
        "queueCheck" => Ok(json_raw(&report.check)),
        _ => bail!("unknown queue report type: {report_type}"),
    }
}

pub(super) fn demand_per_item_calls(
    config: &no_mistakes::config::v2::NoMistakesConfig,
    plan: &mut no_mistakes::codebase::check_facts::CheckFactPlan,
) {
    plan.graph.per_item_calls |= config
        .rule_configured(no_mistakes::codebase::rules::QUERY_REACHED_PER_ITEM)
        || no_mistakes::codebase::rules::no_single_enqueue_in_loop::demands_analysis(config);
}
