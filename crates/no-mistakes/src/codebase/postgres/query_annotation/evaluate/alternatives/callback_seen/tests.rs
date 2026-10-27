use super::*;
use crate::codebase::postgres::query_annotation::module_ids::ModuleIds;
use crate::fx::FxHashSet;
use std::path::{Path, PathBuf};

fn keys(frames: &[usize]) -> CallbackSet {
    let modules = ModuleIds::new([PathBuf::from("callbacks.ts")]);
    frames
        .iter()
        .map(|frame| {
            (
                modules.id(Path::new("callbacks.ts")),
                12,
                *frame,
                vec!["value".into()],
            )
        })
        .collect()
}

#[test]
fn unchanged_branches_share_keys_without_intersection_or_remap_copies() {
    let initial = keys(&(0..1000).collect::<Vec<_>>());
    let mut joined = Some(initial.clone());
    let current = Some(initial.clone());
    // A retain or rebuild would detach this shared allocation even if equal.
    for _ in 0..100 {
        accumulate(&mut joined, &current, false);
        remap(&mut joined, 1000, &FxHashMap::default());
        assert!(joined.as_ref().unwrap().shares(&initial));
    }
    let identity = (0..1000).map(|frame| (frame, frame)).collect();
    remap(&mut joined, 0, &identity);
    assert!(joined.as_ref().unwrap().shares(&initial));
}

#[test]
fn branch_intersection_and_compaction_match_owned_sets() {
    let initial = keys(&[0, 1, 3, 4]);
    let mut joined = Some(initial.clone());
    let mut branch = initial.clone();
    let removed = keys(&[1]).iter().next().unwrap().clone();
    branch.remove(&removed);
    assert!(
        initial.contains(&removed),
        "a sibling retains its seen identity"
    );
    accumulate(&mut joined, &Some(branch), false);
    let mut expected = initial.iter().cloned().collect::<FxHashSet<_>>();
    expected.remove(&removed);
    assert_eq!(&**joined.as_ref().unwrap(), &expected);
    let mapping = FxHashMap::from_iter([(3, 2)]);
    remap(&mut joined, 1, &mapping);
    expected = expected
        .into_iter()
        .filter_map(|mut key| {
            if key.2 >= 1 {
                key.2 = *mapping.get(&key.2)?;
            }
            Some(key)
        })
        .collect();
    assert_eq!(&**joined.as_ref().unwrap(), &expected);
    assert!(
        initial.iter().any(|key| key.2 == 4),
        "dropped frames affect only the compacted snapshot"
    );
}

#[test]
fn snapshot_equality_ignores_conservative_frame_bound() {
    let mut seen = keys(&[0, 10]);
    let removed = keys(&[10]).iter().next().unwrap().clone();
    seen.remove(&removed);
    assert_eq!(seen, keys(&[0]));
    let mut joined = None;
    accumulate(&mut joined, &Some(seen.clone()), true);
    assert_eq!(joined, Some(seen));
}

#[test]
fn removed_maximum_and_new_keys_still_remap_and_merge_exactly() {
    let mut seen = keys(&[0, 10]);
    let maximum = keys(&[10]).iter().next().unwrap().clone();
    seen.remove(&maximum);
    let added = keys(&[20]).iter().next().unwrap().clone();
    seen.insert(added);
    let sibling = seen.clone();
    let mut joined = Some(seen);
    // Merging frame identities also deduplicates complete callback keys.
    remap(&mut joined, 1, &FxHashMap::from_iter([(20, 0)]));
    assert_eq!(joined, Some(keys(&[0])));
    assert_eq!(sibling, keys(&[0, 20]));
}
