use super::analysis::unconstructed;
use super::config::Options;
use super::RULE_ID;
use crate::codebase::dependencies::graph::{DepGraph, GraphBuildPlan, ResolvedClassBase};
use crate::codebase::rules::path_filter::{GlobMatcher, RulePathFilter};
use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_source::{is_test_file, relative_slash_path};
use crate::config::v2::schema::RuleDef;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use std::path::Path;

pub(crate) fn graph_plan(config: &NoMistakesConfig) -> Option<GraphBuildPlan> {
    config.rule_configured(RULE_ID).then(|| GraphBuildPlan {
        class_hierarchy: true,
        ..Default::default()
    })
}

pub(crate) fn check_with_graph(
    root: &Path,
    config: &NoMistakesConfig,
    graph: &DepGraph,
) -> Result<Vec<RuleFinding>> {
    let root = crate::codebase::ts_resolver::normalize_path(root);
    let mut findings = Vec::new();
    for application in config.rule_applications(RULE_ID) {
        let options: Options = application.try_rule_options()?;
        let extra_tests =
            GlobMatcher::new(&options.test_files, &format!("{RULE_ID} options.testFiles"))?;
        let path_filter = RulePathFilter::new(&root, config, application)?;
        let is_test = |file: &Path| {
            let relative = relative_slash_path(&root, file);
            // `is_test_file` looks for a `/__tests__/` segment, so give a
            // top-level `__tests__` directory the slash it expects.
            is_test_file(&format!("/{relative}")) || extra_tests.is_match(&relative)
        };
        findings.extend(
            unconstructed(graph, is_test, |file| path_filter.is_match(file))
                .into_iter()
                .map(|class| finding(&root, application, class)),
        );
    }
    Ok(findings)
}

fn finding(root: &Path, application: &RuleDef, class: &ResolvedClassBase) -> RuleFinding {
    let name = class
        .class_scope
        .rsplit_once('/')
        .map_or(class.class_scope.as_str(), |(_, name)| name);
    // An anonymous default export is scoped as `default`, which no class can
    // be named.
    let subject = if name == "default" {
        "default-exported error class".to_string()
    } else {
        format!("exported error class `{name}`")
    };
    let message = application.message.as_deref().map_or_else(
        || format!("{subject} is never constructed or subclassed in non-test source"),
        |message| format!("{message}: `{name}`"),
    );
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: relative_slash_path(root, &class.file),
        line: class.line.max(1) as usize,
        message,
        import: None,
        target: Some(name.to_string()),
    }
}
