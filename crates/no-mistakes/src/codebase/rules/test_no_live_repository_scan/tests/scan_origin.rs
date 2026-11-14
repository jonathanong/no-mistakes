use super::super::enumerators::uses_glob_cwd;
use super::super::provenance::scan_origin;
use super::{classify, origin, PathAnchor, PathOrigin, Verdict, DEFAULT_FIXTURE_DIRS};
use crate::codebase::dependencies::extract::InvocationKind;
use crate::codebase::dependencies::graph::{ResolvedCallSite, ResolvedCallTarget};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::codebase::ts_source::module_facts::arguments::{ArgumentValues, PathProvenance};
use crate::codebase::ts_source::module_facts::TypeScriptModuleFacts;
use std::sync::Arc;

fn site(target: ResolvedCallTarget) -> ResolvedCallSite {
    ResolvedCallSite {
        file: "/repo/policy.mts".into(),
        caller: None,
        caller_id: None,
        line: 1,
        offset: 7,
        invocation: InvocationKind::Call,
        source_callee: "globSync".into(),
        target,
        alternatives: Vec::new(),
        returned_from: Vec::new(),
        typed_from: Vec::new(),
        returned_member_from: Vec::new(),
        target_node: None,
    }
}

fn facts(projected: PathOrigin, raw: PathOrigin) -> TsFileFacts {
    TsFileFacts {
        module_bindings: Some(Arc::new(TypeScriptModuleFacts {
            call_arguments: Some(Arc::new(
                [(
                    7,
                    ArgumentValues {
                        values: Vec::new(),
                        spread_from: None,
                        path: Some(projected),
                    },
                )]
                .into_iter()
                .collect(),
            )),
            path_provenance: Some(Arc::new(PathProvenance {
                arg_paths: [(7, vec![(0, raw)])].into_iter().collect(),
                ..Default::default()
            })),
            ..Default::default()
        })),
        ..Default::default()
    }
}

#[test]
fn a_known_repository_function_cannot_use_glob_cwd_to_bound_its_root() {
    let owned = origin(PathAnchor::Temp, &[]);
    let live = origin(PathAnchor::Cwd, &["src"]);
    let facts = facts(owned.clone(), live.clone());
    let mut site = site(ResolvedCallTarget::RepositoryFunction {
        file: "/repo/helper.mts".into(),
        scope: "globSync".into(),
    });
    assert!(!site.has_unknown_target());
    assert!(!uses_glob_cwd(&site));
    assert_eq!(scan_origin(&facts, &site), Some(&live));
    assert!(matches!(
        classify(scan_origin(&facts, &site), DEFAULT_FIXTURE_DIRS),
        Verdict::Live(_)
    ));

    site.target = ResolvedCallTarget::ModuleExport {
        specifier: "node:fs".into(),
        export_path: "globSync".into(),
        repository_target: None,
        callable_id: None,
    };
    assert!(uses_glob_cwd(&site));
    assert_eq!(scan_origin(&facts, &site), Some(&owned));
    assert_eq!(
        classify(scan_origin(&facts, &site), DEFAULT_FIXTURE_DIRS),
        Verdict::Allowed
    );
}

#[test]
fn missing_prepared_path_provenance_keeps_recorded_argument_facts_unresolved() {
    let fixture = origin(PathAnchor::Relative, &["fixtures"]);
    let mut facts = facts(fixture.clone(), fixture.clone());
    let mut site = site(ResolvedCallTarget::ModuleExport {
        specifier: "node:fs".into(),
        export_path: "readdirSync".into(),
        repository_target: None,
        callable_id: None,
    });
    site.source_callee = "readdirSync".into();
    assert_eq!(scan_origin(&facts, &site), Some(&fixture));
    Arc::make_mut(facts.module_bindings.as_mut().unwrap()).path_provenance = None;
    assert!(
        super::super::super::forbidden_calls::call_argument_values(&facts, site.offset).is_some()
    );
    assert_eq!(scan_origin(&facts, &site), None);
    assert_eq!(
        classify(scan_origin(&facts, &site), DEFAULT_FIXTURE_DIRS),
        Verdict::Unresolved(
            "the path argument was not recorded (spread argument or unparsed module)"
        )
    );
}
