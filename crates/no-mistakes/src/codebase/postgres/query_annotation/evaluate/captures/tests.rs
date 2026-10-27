use super::*;

impl CapturedOrigins {
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn strong_count_probe(
        &self,
    ) -> impl Fn() -> usize {
        let weak = Arc::downgrade(&self.0);
        move || weak.strong_count()
    }

    pub(in crate::codebase::postgres::query_annotation::evaluate) fn insert_origin(
        &mut self,
        name: String,
        origin: Environment,
    ) {
        let origins = Arc::make_mut(&mut self.0);
        origins.bindings.insert(name, origin);
        origins.max_origin.take();
    }

    pub(in crate::codebase::postgres::query_annotation::evaluate) fn reserve_origins(
        &mut self,
        capacity: usize,
    ) {
        Arc::make_mut(&mut self.0).bindings.reserve(capacity);
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

#[test]
fn cached_prefix_guard_rejects_private_and_out_of_range_origins() {
    let mut bindings: CapturedBindings =
        FxHashMap::from_iter([(1, FxHashMap::from_iter([("name".into(), 0)]).into())]);
    assert!(prefix_is_unchanged(&bindings, 2));
    assert!(
        !prefix_is_unchanged(&bindings, 1),
        "owner at base is private"
    );

    let mut out_of_range: CapturedOrigins = FxHashMap::from_iter([("name".into(), 99)]).into();
    assert!(
        out_of_range.0.max_origin.get().is_none(),
        "construction is cold"
    );
    assert_eq!(out_of_range.max_origin(), Some(99));
    assert_eq!(out_of_range.0.max_origin.get(), Some(&Some(99)));
    assert!(!prefix_is_unchanged(
        &FxHashMap::from_iter([(0, out_of_range.clone())]),
        2
    ));
    out_of_range.remap(&FxHashMap::from_iter([(99, 1)]));
    assert_eq!(out_of_range.max_origin(), Some(1));
    assert_eq!(out_of_range.0.max_origin.get(), Some(&Some(1)));
    bindings.insert(0, out_of_range);
    assert!(prefix_is_unchanged(&bindings, 2));

    let private: CapturedOrigins = FxHashMap::from_iter([("name".into(), 99)]).into();
    assert!(!prefix_is_unchanged(
        &FxHashMap::from_iter([(2, private.clone())]),
        2
    ));
    assert!(
        private.0.max_origin.get().is_none(),
        "private owner short-circuits"
    );
}

#[test]
fn origin_cache_is_lazy_shared_and_ignored_by_logical_equality() {
    let cold: CapturedOrigins = FxHashMap::from_iter([("name".into(), 99)]).into();
    let clone = cold.clone();
    assert!(Arc::ptr_eq(&cold.0, &clone.0));
    assert!(cold.0.max_origin.get().is_none());

    assert_eq!(clone.max_origin(), Some(99));
    assert_eq!(cold.0.max_origin.get(), Some(&Some(99)));
    let separately_constructed: CapturedOrigins =
        FxHashMap::from_iter([("name".into(), 99)]).into();
    assert!(separately_constructed.0.max_origin.get().is_none());
    assert_eq!(
        cold, separately_constructed,
        "cache warmth is not semantic state"
    );
}
