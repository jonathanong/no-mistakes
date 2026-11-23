use std::path::Path;

use super::super::{modules::ResolvedModules, options, targets::matching_target};
use super::{call_site, check, configured};
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::graph::{
    test_support::from_raw_maps, KnownCallTarget, ResolvedCallAlternatives, ResolvedCallCandidate,
    ResolvedCallTarget,
};
use crate::codebase::rules::no_single_enqueue_in_loop;

#[test]
fn a_single_export_pattern_requests_the_call_graph() {
    let config = configured("      singleExportPattern: '^enqueue[A-Z]'\n");
    assert!(no_single_enqueue_in_loop::demands_analysis(&config));
    let plan = no_single_enqueue_in_loop::graph_plan(&config).unwrap();
    assert!(plan.calls);
}

#[test]
fn invalid_patterns_name_the_option_before_the_graph_is_required() {
    for (field, value) in [
        ("singleExportPattern", "'['"),
        ("singleExportPattern", "''"),
        ("bulkExportPattern", "'['"),
        ("bulkExportPattern", "''"),
        ("moduleGlobs", "['[']"),
        ("moduleGlobs", "['']"),
    ] {
        let config = configured(&format!("      {field}: {value}\n"));
        let error = check(Path::new("/repo"), &config, None, &CheckFactMap::default()).unwrap_err();
        assert!(error.to_string().contains(field), "{error:#}");
    }
}

#[test]
fn all_call_alternatives_must_match_the_single_pattern() {
    let config = configured(concat!(
        "      singleExportPattern: '^enqueue[A-Z]'\n",
        "      bulkExportPattern: '^enqueueBulk'\n",
        "      moduleGlobs: ['./queue']\n",
        "      bulk: [{ module: ./queue, export: enqueueLegacy }]\n",
    ));
    let targets = options::validate(&config.rules[0].try_rule_options().unwrap()).unwrap();
    let graph = from_raw_maps("/repo".into(), Default::default(), Default::default());
    let resolved = ResolvedModules::empty();

    let match_alternatives = |second_target, has_unknown| {
        let known = vec![imported_export("./queue", "enqueuePrimary"), second_target];
        let alternatives = ResolvedCallAlternatives::new(known, has_unknown);
        let site = call_site(ResolvedCallTarget::Alternatives(alternatives.into()));
        matching_target(&graph, &targets, &site, &resolved)
    };
    assert_eq!(
        match_alternatives(imported_export("./queue", "enqueueBackup"), false).as_deref(),
        Some("./queue#enqueueBackup, ./queue#enqueuePrimary")
    );
    assert_eq!(
        match_alternatives(imported_export("./queue", "enqueueBackup"), true),
        None
    );
    for non_single in [
        imported_export("./queue", "enqueueBulkWidgets"),
        imported_export("./queue", "enqueueLegacy"),
        imported_export("./other", "enqueueBackup"),
        ResolvedCallCandidate {
            target: KnownCallTarget::Global {
                name: "enqueueWidget".into(),
            },
            node: None,
        },
    ] {
        assert_eq!(match_alternatives(non_single, false), None);
    }
}

fn imported_export(module: &str, export: &str) -> ResolvedCallCandidate {
    ResolvedCallCandidate {
        target: KnownCallTarget::ModuleExport {
            specifier: module.into(),
            export_path: export.into(),
            repository_target: None,
            callable_id: None,
        },
        node: None,
    }
}
