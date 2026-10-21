use super::*;

#[test]
fn profile_batch_merges_in_the_same_order_with_one_or_four_workers() {
    let run = |threads| {
        let mut files = prepared();
        let candidates = files.keys().cloned().collect::<BTreeSet<_>>();
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .unwrap()
            .install(|| {
                project(&mut files, |specifier, from| {
                    let target = from.parent().unwrap().join(specifier).with_extension("mts");
                    candidates.contains(&target).then_some(target)
                });
            });
        files
            .iter()
            .flat_map(|(path, file)| {
                file.query_annotation.iter().map(move |(profile, facts)| {
                    ((path.clone(), profile.clone()), facts.calls.clone())
                })
            })
            .collect::<BTreeMap<_, _>>()
    };
    // Contextual preference depends on root order; worker scheduling must not.
    assert_eq!(run(1), run(4));
}

#[test]
fn mixed_profile_scopes_match_separate_profile_projections() {
    let run = |only: Option<&str>| {
        let mut files = prepared();
        for (path, file) in &mut files {
            // Model distinct include scopes while retaining shared/cyclic helpers.
            file.query_annotation.retain(|(profile, _)| {
                let selected = if path.components().any(|part| part.as_os_str() == "nested-a") {
                    profile.import_specifier == "@app/db"
                } else if path.components().any(|part| part.as_os_str() == "nested-b") {
                    profile.import_specifier == "@app/other"
                } else {
                    true
                };
                selected && only.is_none_or(|name| profile.import_specifier == name)
            });
        }
        let candidates = files.keys().cloned().collect::<BTreeSet<_>>();
        project(&mut files, |specifier, from| {
            let target = from.parent().unwrap().join(specifier).with_extension("mts");
            candidates.contains(&target).then_some(target)
        });
        files
            .iter()
            .flat_map(|(path, file)| {
                file.query_annotation.iter().map(move |(profile, facts)| {
                    ((path.clone(), profile.clone()), facts.calls.clone())
                })
            })
            .collect::<BTreeMap<_, _>>()
    };
    let mut separate = run(Some("@app/db"));
    separate.extend(run(Some("@app/other")));
    assert_eq!(run(None), separate);
}
