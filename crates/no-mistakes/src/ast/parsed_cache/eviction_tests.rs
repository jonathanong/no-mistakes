use super::ParsedProgramCache;
use std::sync::Arc;

thread_local! {
    pub(super) static EVICTION_PROBES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

macro_rules! eviction_probe {
    () => {
        eviction_tests::EVICTION_PROBES.with(|probes| probes.set(probes.get() + 1))
    };
}

#[test]
fn eviction_releases_the_path_key_and_every_mode_only_for_that_file() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check-facts/bounded-session-memory");
    let first = root.join("source.ts");
    let second = root.join("outer.ts");
    EVICTION_PROBES.set(0);
    let cache = ParsedProgramCache::default();
    for path in [&first, &second] {
        let text: Arc<str> = std::fs::read_to_string(path).unwrap().into();
        cache.with_program(path, text.clone(), |_, _| ()).unwrap();
        cache
            .with_recovered_typescript_program_observed(path, text, || {}, |_, _, _| ())
            .unwrap();
    }
    assert_eq!(cache.entry_count(), 4);
    assert_eq!(cache.entries.borrow().paths.len(), 2);
    cache.remove_path(&root.join("./source.ts"));
    assert_eq!(cache.entry_count(), 2);
    assert!(cache.contains_path(&second));
    assert!(!cache.contains_path(&first));
    assert_eq!(cache.entries.borrow().paths.len(), 1);
    cache.remove_path(&first);
    cache.remove_path(&second);
    assert_eq!(cache.entry_count(), 0);
    assert!(cache.entries.borrow().paths.is_empty());
}

fn saved_source(name: &str) -> (std::path::PathBuf, Arc<str>) {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check-facts/bounded-session-memory")
        .join(name);
    let source = std::fs::read_to_string(&path).unwrap().into();
    (path, source)
}

#[test]
fn one_entry_eviction_examines_one_entry() {
    let (path, source) = saved_source("source.ts");
    EVICTION_PROBES.set(0);
    let cache = ParsedProgramCache::default();
    cache.with_program(&path, source, |_, _| ()).unwrap();
    assert_eq!(cache.entry_count(), 1);
    cache.remove_path(&path);
    assert_eq!(EVICTION_PROBES.get(), 1);
    assert_eq!(cache.entry_count(), 0);
    assert!(cache.entries.borrow().paths.is_empty());
}

#[test]
fn path_eviction_preserves_the_other_path() {
    EVICTION_PROBES.set(0);
    let cache = ParsedProgramCache::default();
    let (first, first_source) = saved_source("source.ts");
    let (second, second_source) = saved_source("outer.ts");
    cache.with_program(&first, first_source, |_, _| ()).unwrap();
    cache
        .with_program(&second, second_source, |_, _| ())
        .unwrap();
    cache.remove_path(&first);
    assert_eq!(EVICTION_PROBES.get(), 1);
    assert_eq!(cache.entry_count(), 1);
    assert!(cache.contains_path(&second));
    assert!(!cache.contains_path(&first));
    assert_eq!(cache.entries.borrow().paths.len(), 1);
}

#[test]
fn path_eviction_handles_a_fallback_only_entry_without_a_standard_error() {
    EVICTION_PROBES.set(0);
    let cache = ParsedProgramCache::default();
    let (path, source) = saved_source("source.ts");
    cache
        .with_recovered_typescript_program_observed(&path, source, || {}, |_, _, _| ())
        .unwrap();
    // A cached fallback is not evidence of a Standard-mode parse failure.
    assert!(cache.parse_error(&path).is_none());
    assert_eq!(cache.entry_count(), 1);
    cache.remove_path(&path);
    assert_eq!(EVICTION_PROBES.get(), 1);
    assert_eq!(cache.entry_count(), 0);
    assert!(cache.entries.borrow().paths.is_empty());
}

#[test]
fn path_eviction_stays_constant_after_the_program_count_shrinks() {
    EVICTION_PROBES.set(0);
    let cache = ParsedProgramCache::default();
    let (first, first_source) = saved_source("source.ts");
    let (second, second_source) = saved_source("outer.ts");
    for (path, source) in [(&first, first_source), (&second, second_source)] {
        cache.with_program(path, source.clone(), |_, _| ()).unwrap();
        cache
            .with_recovered_typescript_program_observed(path, source, || {}, |_, _, _| ())
            .unwrap();
    }
    assert_eq!(cache.entry_count(), 4);
    cache.remove_path(&first);
    assert_eq!(EVICTION_PROBES.get(), 1);
    assert_eq!(cache.entry_count(), 2);
    assert!(cache.contains_path(&second));
    assert_eq!(cache.entries.borrow().paths.len(), 1);
    // Evicting modes must not turn subsequent eviction into a table scan.
    EVICTION_PROBES.set(0);
    cache.remove_path(&first);
    assert_eq!(EVICTION_PROBES.get(), 1);
    EVICTION_PROBES.set(0);
    cache.remove_path(&second);
    assert_eq!(EVICTION_PROBES.get(), 1);
    assert_eq!(cache.entry_count(), 0);
    assert!(cache.entries.borrow().paths.is_empty());
}

#[test]
fn every_mode_is_evicted_with_one_path_lookup() {
    EVICTION_PROBES.set(0);
    let cache = ParsedProgramCache::default();
    let (first, first_source) = saved_source("source.ts");
    let (second, second_source) = saved_source("outer.ts");
    for (path, source) in [(&first, first_source), (&second, second_source)] {
        cache.with_program(path, source.clone(), |_, _| ()).unwrap();
        cache
            .with_recovered_typescript_program_observed(path, source, || {}, |_, _, _| ())
            .unwrap();
    }
    assert_eq!(cache.entry_count(), 4);
    cache.remove_path(&first);
    assert_eq!(EVICTION_PROBES.get(), 1);
    assert_eq!(cache.entry_count(), 2);
    assert!(cache.contains_path(&second));
    assert!(!cache.contains_path(&first));
}

#[test]
fn reentrant_observation_keeps_the_program_count_exact() {
    let cache = ParsedProgramCache::default();
    let (path, source) = saved_source("source.ts");
    cache
        .with_program_observed(
            &path,
            source.clone(),
            || {
                cache
                    .with_program(&path, source.clone(), |_, _| ())
                    .unwrap();
            },
            |_, _| (),
        )
        .unwrap();
    assert_eq!(cache.entry_count(), 1);
    assert_eq!(cache.entries.borrow().paths.len(), 1);
    cache.remove_path(&path);
    assert_eq!(cache.entry_count(), 0);
    assert!(cache.entries.borrow().paths.is_empty());
}
