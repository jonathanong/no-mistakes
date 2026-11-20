use super::super::config::{AnalyzerCommand, Options};
use super::Gate;
use crate::codebase::dependencies::extract::InvocationKind;
use crate::codebase::dependencies::graph::{ResolvedCallSite, ResolvedCallTarget, TsFactLookup};
use crate::codebase::rules::analysis_policy::AnalysisDiagnostics;
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::codebase::ts_source::module_facts::arguments::{
    ArgumentValue, ArgumentValues, PathOrigin, PathProvenance,
};
use crate::codebase::ts_source::module_facts::TypeScriptModuleFacts;
use crate::config::v2::schema::RuleDef;
use std::path::Path;
use std::sync::Arc;

struct Lookup(Option<TsFileFacts>);

impl TsFactLookup for Lookup {
    fn get_ts_facts(&self, _: &Path) -> Option<&TsFileFacts> {
        self.0.as_ref()
    }
}

fn site(target: ResolvedCallTarget) -> ResolvedCallSite {
    ResolvedCallSite {
        file: "/repo/tests/orders.test.ts".into(),
        caller: None,
        caller_id: None,
        line: 4,
        offset: 7,
        invocation: InvocationKind::Call,
        source_callee: "execFileSync".into(),
        target,
        alternatives: Vec::new(),
        returned_from: Vec::new(),
        typed_from: Vec::new(),
        returned_member_from: Vec::new(),
        target_node: None,
    }
}

fn launcher() -> ResolvedCallSite {
    site(ResolvedCallTarget::ModuleExport {
        specifier: "node:child_process".into(),
        export_path: "execFileSync".into(),
        repository_target: None,
        callable_id: None,
    })
}

fn options() -> Options {
    Options {
        analyzers: vec![AnalyzerCommand {
            command: "widget-audit".to_string(),
            args: vec!["check".to_string()],
        }],
        ..Options::default()
    }
}

fn note(lookup: &Lookup, call: &ResolvedCallSite) -> AnalysisDiagnostics {
    let options = options();
    let dirs = options.fixture_dirs();
    let application = RuleDef::default();
    let gate = Gate::new(Path::new("/repo"), &application, lookup, &options, &dirs);
    let mut diagnostics = AnalysisDiagnostics::default();
    gate.note(&mut diagnostics, call);
    diagnostics
}

fn bindings(arguments: ArgumentValues, provenance: Option<PathProvenance>) -> TsFileFacts {
    TsFileFacts {
        module_bindings: Some(Arc::new(TypeScriptModuleFacts {
            call_arguments: Some(Arc::new([(7, arguments)].into_iter().collect())),
            path_provenance: provenance.map(Arc::new),
            ..Default::default()
        })),
        ..Default::default()
    }
}

fn widget_call() -> ArgumentValues {
    ArgumentValues {
        values: vec![
            ArgumentValue::Scalar(serde_json::Value::String("widget-audit".into())),
            ArgumentValue::Array(vec![ArgumentValue::Scalar(serde_json::Value::String(
                "check".into(),
            ))]),
            ArgumentValue::Object(crate::fx::FxHashMap::default()),
        ],
        spread_from: None,
        path: None,
    }
}

fn assert_unresolved(diagnostics: &AnalysisDiagnostics) {
    assert!(diagnostics.findings.is_empty());
    assert_eq!(diagnostics.advisories.len(), 1);
    assert_eq!(
        diagnostics.advisories[0].target.as_deref(),
        Some("unresolved root")
    );
    assert!(diagnostics.advisories[0]
        .message
        .contains("widget-audit check"));
}

#[test]
fn missing_facts_and_arguments_skip_the_launch() {
    let call = launcher();
    for lookup in [
        Lookup(None),
        Lookup(Some(TsFileFacts::default())),
        Lookup(Some(bindings(
            ArgumentValues {
                values: Vec::new(),
                spread_from: None,
                path: None,
            },
            None,
        ))),
    ] {
        let diagnostics = note(&lookup, &call);
        assert!(diagnostics.findings.is_empty());
        assert!(diagnostics.advisories.is_empty());
    }
}

#[test]
fn an_unrecorded_cwd_origin_is_an_unresolved_root() {
    let call = launcher();
    let mut provenance = PathProvenance::default();
    provenance
        .arg_member_paths
        .insert(7, vec![(2, "cwd".to_string(), PathOrigin::unknown())]);
    assert_unresolved(&note(
        &Lookup(Some(bindings(widget_call(), Some(provenance)))),
        &call,
    ));
    assert_unresolved(&note(&Lookup(Some(bindings(widget_call(), None))), &call));
}

#[test]
fn a_local_callable_is_not_an_analyzer_launcher() {
    let local = site(ResolvedCallTarget::Global {
        name: "execFileSync".into(),
    });
    assert!(super::command::launcher_name(&local).is_none());
    let mut provenance = PathProvenance::default();
    provenance
        .arg_member_paths
        .insert(7, vec![(2, "cwd".to_string(), PathOrigin::unknown())]);
    let diagnostics = note(
        &Lookup(Some(bindings(widget_call(), Some(provenance)))),
        &local,
    );
    assert!(diagnostics.findings.is_empty());
    assert!(diagnostics.advisories.is_empty());
}
