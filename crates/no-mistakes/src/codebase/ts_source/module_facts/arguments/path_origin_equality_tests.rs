use super::*;
use crate::codebase::ts_source::module_facts::arguments::EnvDiagnostic;
use std::collections::hash_map::DefaultHasher;

fn hash(origin: &PathOrigin) -> u64 {
    let mut hasher = DefaultHasher::new();
    origin.hash(&mut hasher);
    hasher.finish()
}

#[test]
fn diagnostic_metadata_does_not_change_proof_identity() {
    let unknown = PathOrigin::unknown();
    let mut diagnostic = unknown.clone();
    diagnostic.env_diagnostic = Some(Box::new(EnvDiagnostic {
        name: "SCAN_ROOT".to_owned(),
        fallback: Some("src".to_owned()),
        captured: true,
        line: 3,
    }));
    diagnostic.captured_getter = Some(42);
    assert_eq!(unknown, diagnostic);
    assert_eq!(hash(&unknown), hash(&diagnostic));
    assert_eq!(
        PathOrigin::alternatives(vec![unknown.clone(), diagnostic]),
        unknown
    );
}

#[test]
fn proof_fields_remain_distinct() {
    let unknown = PathOrigin::unknown();
    let mut environment = unknown.clone();
    environment.env = Some("SCAN_ROOT".to_owned());
    assert_ne!(unknown, environment);
    let mut captured = environment.clone();
    captured.env = None;
    captured.captured_env = Some(Box::new(super::super::CapturedEnv {
        name: "SCAN_ROOT".to_owned(),
        offset: 42,
    }));
    assert_ne!(environment, captured);
    let mut later_capture = captured.clone();
    later_capture.captured_env.as_mut().unwrap().offset = 43;
    assert_ne!(captured, later_capture);
    assert_ne!(hash(&captured), hash(&later_capture));
    assert_eq!(
        PathOrigin::alternatives(vec![environment, captured, later_capture])
            .atoms()
            .len(),
        3
    );
    let mut parameter = unknown.clone();
    parameter.param = Some(0);
    assert_ne!(unknown, parameter);
    let mut scalar = unknown.clone();
    scalar.require_scalar = true;
    assert_ne!(unknown, scalar);
}

#[test]
fn duplicate_proofs_discard_conflicting_capture_metadata() {
    let mut first = PathOrigin::unknown();
    first.env_diagnostic = Some(Box::new(EnvDiagnostic {
        name: "FIRST".to_owned(),
        fallback: None,
        captured: true,
        line: 1,
    }));
    first.captured_getter = Some(1);
    let mut second = first.clone();
    second.env_diagnostic.as_mut().unwrap().name = "SECOND".to_owned();
    second.captured_getter = Some(2);
    let merged = PathOrigin::alternatives(vec![first.clone(), second, first]);
    assert_eq!(merged, PathOrigin::unknown());
    assert!(merged.env_diagnostic.is_none());
    assert!(merged.captured_getter.is_none());
    assert!(merged.alternatives.is_empty());
}
