use super::*;
use crate::codebase::postgres::query_annotation::evaluate::Value;
use crate::fx::FxHashMap;

fn make_origins() -> CapturedOrigins {
    FxHashMap::from_iter([("value".into(), 0)]).into()
}

#[test]
fn refresh_metadata_is_cold_until_recorded_and_never_changes_equality() {
    let origins = make_origins();
    let scopes = vec![Scope::default(), Scope::default()];
    let frame = &scopes[1];
    assert!(!origins.refresh_is_current(frame, &scopes));
    let independent = make_origins();
    origins.remember_refresh(frame, &FxHashSet::from_iter([0]), &scopes);
    assert!(origins.refresh_is_current(frame, &scopes));
    assert_eq!(origins, independent);
    assert!(origins.clone().refresh_is_current(frame, &scopes));
    let copied = origins.0.as_ref().clone();
    assert!(copied.refresh.lock().unwrap().is_none());
    assert_eq!(*origins.0, copied);
}

#[test]
fn frame_and_origin_epoch_changes_invalidate_without_retaining_old_bindings() {
    let origins = make_origins();
    let mut scopes = vec![Scope::default(), Scope::default()];
    origins.remember_refresh(&scopes[1], &FxHashSet::from_iter([0]), &scopes);
    let previous_origin = scopes[0].clone();
    scopes[0].insert("value".into(), Value::Number(1));
    assert!(!origins.refresh_is_current(&scopes[1], &scopes));
    scopes[0] = previous_origin;
    assert!(origins.refresh_is_current(&scopes[1], &scopes));
    // Only a Weak remains for this unique frame when its equal write begins.
    scopes[1].insert("same".into(), Value::Unknown);
    assert!(!origins.refresh_is_current(&scopes[1], &scopes));
    origins.remember_refresh(&scopes[1], &FxHashSet::from_iter([0]), &scopes);
    assert!(origins.refresh_is_current(&scopes[1], &scopes));
    assert!(!origins.refresh_is_current(&scopes[1], &scopes[1..]));
}

#[test]
fn invalid_dependency_record_discards_old_proof_and_poison_falls_back() {
    let origins = make_origins();
    let scopes = vec![Scope::default(), Scope::default()];
    origins.remember_refresh(&scopes[1], &FxHashSet::from_iter([0]), &scopes);
    origins.remember_refresh(&scopes[1], &FxHashSet::from_iter([9]), &scopes);
    assert!(!origins.refresh_is_current(&scopes[1], &scopes));
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _guard = origins.0.refresh.lock().unwrap();
        panic!("optional cache poison");
    }));
    origins.remember_refresh(&scopes[1], &FxHashSet::from_iter([0]), &scopes);
    assert!(!origins.refresh_is_current(&scopes[1], &scopes));
}

#[test]
fn helper_name_insert_invalidates_unique_and_shared_origin_proofs() {
    for shared in [false, true] {
        let mut origins = make_origins();
        let scopes = vec![Scope::default(), Scope::default()];
        origins.remember_refresh(&scopes[1], &FxHashSet::from_iter([0]), &scopes);
        // Capacity alone changes no dependency or captured name.
        origins.reserve_origins(32);
        assert!(origins.refresh_is_current(&scopes[1], &scopes));
        let snapshot = shared.then(|| origins.clone());
        origins.insert_origin("newly captured missing name".into(), 0);
        assert!(!origins.refresh_is_current(&scopes[1], &scopes));
        if let Some(snapshot) = snapshot {
            assert!(snapshot.refresh_is_current(&scopes[1], &scopes));
        }
    }
}

#[test]
fn remap_clears_unique_and_shared_origin_metadata_even_when_scope_epochs_match() {
    for shared in [false, true] {
        let mut origins = make_origins();
        let mut scopes = vec![Scope::default(), Scope::default(), Scope::default()];
        scopes[0].insert("value".into(), Value::Number(1));
        scopes[2].insert("value".into(), Value::Number(2));
        origins.remember_refresh(&scopes[1], &FxHashSet::from_iter([0]), &scopes);
        let snapshot = shared.then(|| origins.clone());
        origins.remap(&FxHashMap::from_iter([(0, 2)]));
        assert_eq!(origins["value"], 2);
        assert!(!origins.refresh_is_current(&scopes[1], &scopes));
        if let Some(snapshot) = snapshot {
            assert!(snapshot.refresh_is_current(&scopes[1], &scopes));
            assert_eq!(snapshot["value"], 0);
        }
        origins.remember_refresh(&scopes[1], &FxHashSet::from_iter([2]), &scopes);
        origins.remap(&FxHashMap::from_iter([(2, 2)]));
        assert!(origins.refresh_is_current(&scopes[1], &scopes));
        scopes[2].insert("value".into(), Value::Number(3));
        assert!(!origins.refresh_is_current(&scopes[1], &scopes));
    }
}
