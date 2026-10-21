use super::*;

type Captures = FxHashMap<Environment, FxHashMap<String, Environment>>;

// This is the original full reverse-reader reconstruction, including the
// per-edge owned-name clone even when the same name already has other readers.
fn rebuild(captured: &Captures) -> (Readers, usize) {
    let mut readers = Readers::default();
    let mut name_buffers = 0;
    for (frame, bindings) in captured {
        for (name, origin) in bindings {
            let owned = name.clone();
            name_buffers += usize::from(!name.is_empty());
            readers
                .entry(*origin)
                .or_default()
                .entry(owned)
                .or_default()
                .insert(*frame);
        }
    }
    (readers, name_buffers)
}

pub(in super::super) fn assert_matches_forward(
    readers: &Readers,
    captured: &crate::codebase::postgres::query_annotation::evaluate::CapturedBindings,
) {
    let owned = captured
        .iter()
        .map(|(reader, origins)| (*reader, (**origins).clone()))
        .collect();
    assert_eq!(
        readers,
        &rebuild(&owned).0,
        "every real compaction must match the full old reverse reconstruction"
    );
}

fn forward(captured: Captures, preserved: usize, indices: &FxHashMap<usize, usize>) -> Captures {
    captured
        .into_iter()
        .filter_map(|(env, mut origins)| {
            let key = if env < preserved {
                Some(env)
            } else {
                indices.get(&env).copied()
            }?;
            for origin in origins.values_mut() {
                if let Some(index) = indices.get(origin) {
                    *origin = *index;
                }
            }
            Some((key, origins))
        })
        .collect()
}

fn compare(captured: Captures, preserved: usize, indices: FxHashMap<usize, usize>) -> Readers {
    let destinations = indices.values().copied().collect::<FxHashSet<_>>();
    assert_eq!(
        destinations.len(),
        indices.len(),
        "injective surviving frames"
    );
    assert!(destinations.iter().all(|new| *new >= preserved));
    let (mut actual, _) = rebuild(&captured);
    let pointers = actual
        .values()
        .flat_map(|names| {
            names
                .keys()
                .map(|name| (name.clone(), name.as_ptr() as usize))
        })
        .collect::<Vec<_>>();
    let expected_captures = forward(captured, preserved, &indices);
    let (expected, copied_names) = rebuild(&expected_captures);
    let mut uncounted = actual.clone();
    let count = remap::<true>(&mut actual, preserved, &indices);
    assert_eq!(remap::<false>(&mut uncounted, preserved, &indices), 0);
    assert_eq!(
        uncounted, actual,
        "diagnostics cannot change the private index"
    );
    assert_eq!(
        count,
        expected.values().map(|names| names.len()).sum::<usize>()
    );
    assert_eq!(
        actual, expected,
        "complete private reverse index, no field masking"
    );
    let mut retained_names = 0;
    for names in actual.values() {
        for name in names.keys() {
            assert!(
                pointers
                    .iter()
                    .any(|(old, pointer)| { old == name && *pointer == name.as_ptr() as usize }),
                "surviving name must keep an original owned buffer"
            );
            retained_names += 1;
        }
    }
    eprintln!("READER_WORK old_name_clones={copied_names} retained_name_buffers={retained_names} new_name_buffers=0");
    actual
}

#[test]
fn remapping_matches_complete_forward_rebuild_for_all_small_compactions() {
    for preserved in 0..=3 {
        for mask in 0u32..64 {
            let retained = (preserved..6).filter(|old| mask & (1 << old) != 0);
            let indices = retained
                .enumerate()
                .map(|(new, old)| (old, preserved + new))
                .collect();
            // Cyclic origins and a shared name model retained module initials
            // and nested captures; missing origin mappings intentionally survive.
            let captured = (0..6)
                .map(|frame| {
                    (
                        frame,
                        FxHashMap::from_iter([
                            ("shared".into(), (frame + 1) % 6),
                            ("cycle".into(), (frame + 3) % 6),
                        ]),
                    )
                })
                .collect();
            compare(captured, preserved, indices);
        }
    }
}

#[test]
fn moved_origin_destinations_and_dropped_readers_preserve_prefix_and_cycles() {
    let captured = FxHashMap::from_iter([
        (
            0,
            FxHashMap::from_iter([("prefix".into(), 0), ("cycle".into(), 4)]),
        ),
        (2, FxHashMap::from_iter([("shared".into(), 4)])),
        (3, FxHashMap::from_iter([("dead".into(), 3)])),
        (
            4,
            FxHashMap::from_iter([("shared".into(), 2), ("cycle".into(), 0)]),
        ),
    ]);
    compare(captured, 1, FxHashMap::from_iter([(2, 1), (4, 2)]));
}

#[test]
fn unmapped_origin_collision_merges_exact_forward_relation() {
    let captured = FxHashMap::from_iter([
        (0, FxHashMap::from_iter([("shared".into(), 2)])),
        (
            3,
            FxHashMap::from_iter([("shared".into(), 1), ("unique".into(), 2)]),
        ),
    ]);
    compare(captured, 1, FxHashMap::from_iter([(2, 1), (3, 2)]));
}

mod sparse;
