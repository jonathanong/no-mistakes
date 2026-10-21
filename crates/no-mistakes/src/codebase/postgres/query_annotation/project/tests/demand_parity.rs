use super::*;
use crate::codebase::postgres::query_annotation::evaluate::ExecutionEvents;

const NAMES: &[&str] = &[
    "executor",
    "caller",
    "barrel",
    "entry",
    "cycle-a",
    "cycle-b",
    "unrelated",
    "unrelated-chain",
    "unrelated-cycle-a",
    "unrelated-cycle-b",
    "unresolved",
    "opaque-consumer",
    "computed",
    "dynamic",
    "required",
    "type-only",
    "unknown-callback",
    "literal-await",
];

#[test]
fn actual_executor_seeds_preserve_every_legacy_event_and_both_public_projections() {
    let files = prepared_fixture("helper-tracing-demand-roots", NAMES);
    let candidates = files.keys().cloned().collect::<BTreeSet<_>>();
    let resolve = |specifier: &str, from: &std::path::Path| {
        let target = from.parent().unwrap().join(specifier).with_extension("mts");
        candidates.contains(&target).then_some(target)
    };
    let profiles = super::super::prepare::profiles(&files, &resolve);
    for profile in profiles {
        let old_roots = super::legacy_demand::roots(&profile.files, &profile.options, &resolve);
        let selected = profile
            .roots
            .iter()
            .map(PathBuf::as_path)
            .collect::<BTreeSet<_>>();
        assert!(selected.iter().all(|path| old_roots.contains(path)));
        if profile.options.import_specifier == "@app/db" {
            for name in ["unknown-callback", "literal-await", "cycle-a", "cycle-b"] {
                assert!(selected
                    .iter()
                    .any(|path| path.file_stem().unwrap() == name));
            }
            assert!(old_roots.len() > selected.len());
        }
        let run = |roots: Vec<&std::path::Path>| {
            let mut roots = roots;
            roots.sort();
            let mut events = ExecutionEvents::new();
            for path in roots {
                let mut evaluator = super::super::evaluator::new(
                    &profile.files,
                    &resolve,
                    profile.untrusted_array_intrinsics,
                );
                evaluator.run(path);
                // A removed root must be incapable of any event, even generic
                // events; filtering favorable findings would be unsound.
                if !selected.contains(path) {
                    assert!(evaluator.events.is_empty(), "{}", path.display());
                }
                for (call, values) in evaluator.events {
                    events.entry(call).or_default().extend(values);
                }
            }
            events
        };
        let old_events = run(old_roots);
        let new_events = run(selected.iter().copied().collect());
        assert!(
            old_events == new_events,
            "all event values and order must match"
        );
        let originals = files
            .keys()
            .map(|path| (path.clone(), path.clone()))
            .collect();
        // Annotation and executed-SQL reductions have different contextual
        // preferences. Compare the complete outputs of both, not only findings.
        for collect_executed in [false, true] {
            let apply = |events| {
                let mut output = super::super::ProjectedCalls::new();
                let mut copies = prepared_fixture("helper-tracing-demand-roots", NAMES);
                super::super::merge::apply(
                    &mut copies,
                    &originals,
                    &profile.options,
                    super::super::merge::reduce(events, collect_executed),
                    &mut output,
                );
                let calls = copies
                    .iter()
                    .map(|(path, file)| {
                        (
                            path.clone(),
                            file.query_annotation
                                .iter()
                                .map(|(options, facts)| (options.clone(), facts.calls.clone()))
                                .collect::<Vec<_>>(),
                        )
                    })
                    .collect::<BTreeMap<_, _>>();
                (calls, output)
            };
            assert_eq!(apply(old_events.clone()), apply(new_events.clone()));
        }
    }
}
