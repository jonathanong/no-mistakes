use super::super::config::{AnalyzerCommand, Options};
use super::Gate;
use crate::codebase::dependencies::extract::InvocationKind;
use crate::codebase::dependencies::graph::{ResolvedCallSite, ResolvedCallTarget, TsFactLookup};
use no_mistakes_rule_kernel::file_diagnostics::AnalysisDiagnostics;
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::codebase::ts_source::module_facts::arguments::{
    ArgumentValue, ArgumentValues, PathOrigin, PathProvenance,
};
use crate::codebase::ts_source::module_facts::TypeScriptModuleFacts;
use crate::config::v2::schema::RuleDef;
use std::path::Path;
use std::sync::Arc;

struct Lookup(TsFileFacts);

impl TsFactLookup for Lookup {
    fn get_ts_facts(&self, _: &Path) -> Option<&TsFileFacts> {
        Some(&self.0)
    }
}

fn call(export_path: &str) -> ResolvedCallSite {
    ResolvedCallSite {
        file: "/repo/tests/orders.test.ts".into(),
        caller: None,
        caller_id: None,
        line: 4,
        offset: 7,
        invocation: InvocationKind::Call,
        source_callee: export_path.into(),
        target: ResolvedCallTarget::ModuleExport {
            specifier: "node:child_process".into(),
            export_path: export_path.into(),
            repository_target: None,
            callable_id: None,
        },
        alternatives: Vec::new(),
        returned_from: Vec::new(),
        typed_from: Vec::new(),
        returned_member_from: Vec::new(),
        target_node: None,
    }
}

fn note(
    export_path: &str,
    values: Vec<ArgumentValue>,
    cwd: Option<PathOrigin>,
) -> AnalysisDiagnostics {
    let options = Options {
        analyzers: vec![AnalyzerCommand {
            command: "widget-audit".to_string(),
            args: vec!["check".to_string()],
        }],
        ..Options::default()
    };
    let dirs = options.fixture_dirs();
    let mut provenance = PathProvenance::default();
    if let Some(origin) = cwd {
        provenance
            .arg_member_paths
            .insert(7, vec![(values.len() - 1, "cwd".to_string(), origin)]);
    }
    let facts = TsFileFacts {
        module_bindings: Some(Arc::new(TypeScriptModuleFacts {
            call_arguments: Some(Arc::new(
                [(
                    7,
                    ArgumentValues {
                        values,
                        spread_from: None,
                        path: None,
                    },
                )]
                .into_iter()
                .collect(),
            )),
            path_provenance: Some(Arc::new(provenance)),
            ..Default::default()
        })),
        ..Default::default()
    };
    let application = RuleDef::default();
    let lookup = Lookup(facts);
    let gate = Gate::new(Path::new("/repo"), &application, &lookup, &options, &dirs);
    let mut diagnostics = AnalysisDiagnostics::default();
    gate.note(&mut diagnostics, &call(export_path));
    diagnostics
}

fn string(text: &str) -> ArgumentValue {
    ArgumentValue::Scalar(serde_json::Value::String(text.into()))
}

pub(super) fn owned() -> PathOrigin {
    let mut origin = PathOrigin::unknown();
    origin.segments = vec!["fixtures".to_string()];
    origin
}

pub(super) fn shell(command: &str, cwd: Option<PathOrigin>) -> AnalysisDiagnostics {
    let mut values = vec![string(command)];
    if cwd.is_some() {
        values.push(ArgumentValue::Object(crate::fx::FxHashMap::default()));
    }
    note("execSync", values, cwd)
}

pub(super) fn argv(program: &str, args: &[&str], cwd: Option<PathOrigin>) -> AnalysisDiagnostics {
    let mut values = vec![
        string(program),
        ArgumentValue::Array(args.iter().copied().map(string).collect()),
    ];
    if cwd.is_some() {
        values.push(ArgumentValue::Object(crate::fx::FxHashMap::default()));
    }
    note("execFileSync", values, cwd)
}

pub(super) fn live(diagnostics: &AnalysisDiagnostics) -> String {
    assert_eq!(diagnostics.findings.len(), 1);
    assert!(diagnostics.advisories.is_empty());
    diagnostics.findings[0].message.clone()
}

pub(super) fn assert_quiet(diagnostics: &AnalysisDiagnostics) {
    assert!(diagnostics.findings.is_empty());
    assert!(diagnostics.advisories.is_empty());
}
