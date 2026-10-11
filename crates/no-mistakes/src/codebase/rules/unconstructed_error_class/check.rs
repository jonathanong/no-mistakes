use super::analysis::{is_declaration_file, unconstructed};
use super::config::Options;
use super::RULE_ID;
use crate::codebase::dependencies::graph::{ClassDeclaration, DepGraph, GraphBuildPlan};
use crate::codebase::rules::path_filter::{GlobMatcher, RulePathFilter};
use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_source::{is_test_file, relative_slash_path};
use crate::config::v2::schema::RuleDef;
use crate::config::v2::NoMistakesConfig;
use anyhow::{bail, Result};
use std::path::Path;

pub(crate) fn graph_plan(config: &NoMistakesConfig) -> Option<GraphBuildPlan> {
    config.rule_configured(RULE_ID).then(|| GraphBuildPlan {
        calls: true,
        extends: true,
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
        reject_unseen_source(&root, graph, &is_test)?;
        let classes = unconstructed(graph, is_test, |file| path_filter.is_match(file));
        // A use of a namespace keeps its members quiet even in a test file, so
        // a test file that failed to parse might hold the use that would.
        if classes.iter().any(|class| class.namespace.is_some()) {
            reject_unseen_source(&root, graph, &|_: &Path| false)?;
        }
        findings.extend(
            classes
                .into_iter()
                .map(|class| finding(&root, application, class)),
        );
    }
    Ok(findings)
}

/// The rule concludes that no non-test file constructs a class, so a file whose
/// facts could not be collected might hold the construction it looks for.
/// `ignored` files cannot: test files, unless a namespace member is at stake.
/// `path_filter` is not consulted: a construction outside `include` still counts.
/// Declaration files are skipped: they hold no runtime construction.
fn reject_unseen_source(
    root: &Path,
    graph: &DepGraph,
    ignored: &impl Fn(&Path) -> bool,
) -> Result<()> {
    let mut unseen: Vec<_> = graph
        .parse_errors()
        .filter(|(file, _)| !ignored(file) && !is_declaration_file(file))
        .map(|(file, error)| (relative_slash_path(root, file), error))
        .collect();
    unseen.sort();
    let [(file, error), others @ ..] = unseen.as_slice() else {
        return Ok(());
    };
    let others = match others.len() {
        0 => String::new(),
        1 => " (and 1 other file)".to_string(),
        count => format!(" (and {count} other files)"),
    };
    bail!(
        "{RULE_ID}: cannot prove error classes unconstructed: `{file}`{others} failed to parse: \
         {error}"
    );
}

fn finding(root: &Path, application: &RuleDef, class: &ClassDeclaration) -> RuleFinding {
    let bare = class
        .scope
        .rsplit_once('/')
        .map_or(class.scope.as_str(), |(_, name)| name);
    // A namespace member is named by its path, as it is written at a use.
    let qualified = class
        .namespace
        .as_ref()
        .map(|namespace| format!("{namespace}.{bare}"));
    let name = qualified.as_deref().unwrap_or(bare);
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
        source_offset: None,
        rule: RULE_ID.to_string(),
        file: relative_slash_path(root, &class.file),
        line: class.line.max(1) as usize,
        message,
        import: None,
        target: Some(name.to_string()),
    }
}
