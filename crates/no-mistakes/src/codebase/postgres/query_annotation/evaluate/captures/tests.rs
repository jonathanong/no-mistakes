use super::*;
use std::ops::DerefMut;

// Legacy full-remap test oracles mutate their owned maps directly. Production
// exposes only the guarded origin remap, never an unconstrained mutable view.
impl DerefMut for CapturedOrigins {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Arc::make_mut(&mut self.0)
    }
}

fn fixture() -> CapturedBindings {
    let path = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("capture-snapshot-sharing.json");
    let saved: Vec<(usize, Vec<(String, usize)>)> =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    saved
        .into_iter()
        .map(|(reader, origins)| (reader, origins.into_iter().collect()))
        .collect()
}

fn copied_names(before: &CapturedBindings, after: &CapturedBindings) -> usize {
    after
        .values()
        .flat_map(|origins| origins.keys())
        .filter(|name| {
            !before.values().any(|origins| {
                origins
                    .get_key_value(name.as_str())
                    .is_some_and(|(old, _)| old.as_ptr() == name.as_ptr())
            })
        })
        .count()
}

#[test]
fn saved_snapshot_clone_restore_and_arm_extend_copy_zero_capture_names() {
    let original = fixture();
    let snapshot = original.clone();
    let mut restored = CapturedBindings::default();
    restored.clone_from(&snapshot);
    let mut arm = CapturedBindings::default();
    arm.extend(snapshot.clone());
    let names = original.values().map(|map| map.len()).sum::<usize>();
    assert!(names > 0);
    for state in [&snapshot, &restored, &arm] {
        assert_eq!(state, &original);
        assert_eq!(copied_names(&original, state), 0);
        for (reader, origins) in state {
            assert!(Arc::ptr_eq(&origins.0, &original[reader].0));
        }
    }
    // The old nested HashMap clone duplicates the actual String buffers. This
    // positive control prevents a vacuous zero from the structural work count.
    let old: CapturedBindings = original
        .iter()
        .map(|(reader, origins)| (*reader, (**origins).clone().into()))
        .collect();
    assert_eq!(old, original);
    assert_eq!(copied_names(&original, &old), names);
}

#[test]
fn only_maps_with_changed_origins_detach_and_snapshots_keep_old_origins() {
    let original = fixture();
    let mut arm = original.clone();
    let mapping = FxHashMap::from_iter([(2, 1), (8, 8)]);
    for origins in arm.values_mut() {
        origins.remap(&mapping);
    }
    let expected_copies = original
        .values()
        .filter(|map| map.values().any(|origin| *origin == 2))
        .map(|map| map.len())
        .sum::<usize>();
    assert!(expected_copies > 0);
    assert_eq!(copied_names(&original, &arm), expected_copies);
    for (reader, before) in &original {
        let changed = before.values().any(|origin| *origin == 2);
        assert_eq!(Arc::ptr_eq(&before.0, &arm[reader].0), !changed);
        for (name, origin) in before {
            assert_eq!(
                arm[reader][name],
                mapping.get(origin).copied().unwrap_or(*origin)
            );
        }
    }
    assert_eq!(original, fixture());
}

#[test]
fn identity_empty_and_unshared_remaps_preserve_buffers_until_a_real_change() {
    let mut original = fixture();
    let before = original.clone();
    for origins in original.values_mut() {
        origins.remap(&FxHashMap::default());
        let identity = origins.values().map(|id| (*id, *id)).collect();
        origins.remap(&identity);
    }
    assert_eq!(copied_names(&before, &original), 0);
    let mut empty = CapturedOrigins::default();
    empty.remap(&FxHashMap::from_iter([(2, 1)]));
    assert!(empty.is_empty());
    let mut unique: CapturedOrigins = FxHashMap::from_iter([("owner".into(), 2)]).into();
    let pointer = unique.get_key_value("owner").unwrap().0.as_ptr();
    unique.remap(&FxHashMap::from_iter([(2, 1)]));
    assert_eq!(unique["owner"], 1);
    assert_eq!(unique.get_key_value("owner").unwrap().0.as_ptr(), pointer);
}
