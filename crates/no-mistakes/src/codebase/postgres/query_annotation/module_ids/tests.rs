use super::*;
use std::hash::{Hash, Hasher};

pub(in crate::codebase::postgres::query_annotation) fn saved_paths() -> Vec<PathBuf> {
    let root = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("helper-tracing-module-ids");
    let names: Vec<String> =
        serde_json::from_str(&std::fs::read_to_string(root.join("paths.json")).unwrap()).unwrap();
    names.into_iter().map(|name| root.join(name)).collect()
}

#[derive(Default)]
pub(in crate::codebase::postgres::query_annotation) struct HashBytes(pub usize);
impl Hasher for HashBytes {
    fn finish(&self) -> u64 {
        0
    }
    fn write(&mut self, bytes: &[u8]) {
        self.0 += bytes.len();
    }
}

#[test]
fn unique_logical_paths_have_one_compact_identity_in_existing_path_order() {
    let mut paths = saved_paths();
    for path in &paths {
        assert!(path.is_file());
    }
    let mut shuffled = paths.iter().rev().cloned().collect::<Vec<_>>();
    shuffled.push(paths[0].clone());
    let ids = ModuleIds::new(shuffled);
    paths.sort();
    assert_eq!(ids.paths.len(), paths.len());
    let mut keys = paths.iter().map(|path| ids.id(path)).collect::<Vec<_>>();
    let sorted = keys.clone();
    keys.sort();
    assert_eq!(keys, sorted, "callback ordering must retain Path::Ord");
    assert_ne!(
        ids.id(&paths[1]),
        ids.id(&paths[2]),
        "same basenames are distinct modules"
    );
    for path in paths {
        assert_eq!(ids.path(ids.id(&path)), path.as_path());
    }
    assert_eq!(
        std::mem::size_of::<ModuleId>(),
        std::mem::size_of::<usize>()
    );
}

#[test]
fn compact_key_hash_work_is_independent_of_saved_path_length() {
    let paths = saved_paths();
    let ids = ModuleIds::new(paths.clone());
    let mut compact = HashBytes::default();
    let mut original = HashBytes::default();
    for _ in 0..100 {
        for path in &paths {
            ids.id(path).hash(&mut compact);
            path.hash(&mut original);
        }
    }
    assert_eq!(compact.0, 100 * paths.len() * std::mem::size_of::<usize>());
    assert!(
        original.0 > compact.0,
        "old Path-key work is a required positive control"
    );
}

#[test]
#[should_panic(expected = "module path missing from the request's identity table")]
fn missing_saved_module_reports_the_identity_table_invariant() {
    let mut paths = saved_paths();
    let missing = paths.pop().unwrap();
    // Keep the file real while deliberately omitting it from this request table.
    ModuleIds::new(paths).id(&missing);
}

mod owners;
