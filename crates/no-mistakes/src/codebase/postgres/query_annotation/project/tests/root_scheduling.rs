use super::*;
use crate::codebase::postgres::query_annotation::evaluate::ExecutionEvents;
use rayon::prelude::*;
use std::path::Path;

const FIXTURE: &str = "helper-tracing-root-scheduling";
const NAMES: &[&str] = &[
    "entry-0", "entry-1", "entry-2", "entry-3", "entry-4", "entry-5", "entry-6", "entry-7",
    "shared", "cycle-a", "cycle-b",
];

fn resolver(files: &FileIdMap<CheckFileFacts>) -> impl Fn(&str, &Path) -> Option<PathBuf> + Sync {
    let paths = files.keys().cloned().collect::<BTreeSet<_>>();
    move |specifier, from| {
        let target = from.parent().unwrap().join(specifier).with_extension("mts");
        paths.contains(&target).then_some(target)
    }
}

#[test]
fn demanded_roots_remain_individually_stealable_and_indexed() {
    let files = prepared_fixture(FIXTURE, NAMES);
    let resolve = resolver(&files);
    let profiles = super::super::prepare::profiles(&files, &resolve);
    let roots = profiles
        .into_iter()
        .find(|profile| profile.options.import_specifier == "@app/db")
        .unwrap()
        .roots;
    assert_eq!(roots.len(), 8);
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(1)
        .build()
        .unwrap();
    pool.install(|| {
        let tasks = || super::super::parallel::root_tasks(roots.clone());
        assert_eq!(tasks().collect::<Vec<_>>(), roots);
        // One worker removes stealing races: fold exposes the actual leaf grain.
        let leaves = tasks().fold(|| 0, |count, _| count + 1).collect::<Vec<_>>();
        assert_eq!(leaves.iter().sum::<usize>(), roots.len());
        assert_eq!(leaves, vec![1; roots.len()]);
        // Independent old-policy control proves this guard detects batching.
        let legacy = roots
            .clone()
            .into_par_iter()
            .fold(|| 0, |count, _| count + 1)
            .collect::<Vec<_>>();
        assert_eq!(legacy.iter().sum::<usize>(), roots.len());
        assert!(legacy.iter().any(|count| *count > 1));
    });
}

fn merge_events(mut per_root: Vec<(PathBuf, ExecutionEvents)>) -> ExecutionEvents {
    per_root.sort_by(|left, right| left.0.cmp(&right.0));
    let mut events = ExecutionEvents::new();
    for (_, file_events) in per_root {
        for (call, values) in file_events {
            events.entry(call).or_default().extend(values);
        }
    }
    events
}

#[test]
fn singleton_scheduling_preserves_serial_events_and_complete_projections() {
    let files = prepared_fixture(FIXTURE, NAMES);
    let resolve = resolver(&files);
    for jobs in [1, 2, 4] {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(jobs)
            .build()
            .unwrap();
        for collect_executed in [false, true] {
            let profiles = super::super::prepare::profiles(&files, &resolve);
            let mut reference = Vec::new();
            for profile in &profiles {
                let run = |path: PathBuf| {
                    let mut evaluator = super::super::evaluator::new(
                        &profile.files,
                        &resolve,
                        profile.untrusted_array_intrinsics,
                    );
                    evaluator.run(&path);
                    (path, evaluator.events)
                };
                let serial = merge_events(profile.roots.iter().cloned().map(&run).collect());
                let scheduled = pool.install(|| {
                    merge_events(
                        super::super::parallel::root_tasks(profile.roots.clone())
                            .map(run)
                            .collect(),
                    )
                });
                assert!(
                    serial == scheduled,
                    "all event values and order, jobs={jobs}"
                );
                reference.push(super::super::parallel::ProfileProjection {
                    options: profile.options.clone(),
                    projection: super::super::merge::reduce(serial, collect_executed),
                });
            }
            let parallel = pool
                .install(|| super::super::parallel::evaluate(profiles, &resolve, collect_executed));
            let apply = |projections: Vec<super::super::parallel::ProfileProjection>| {
                let mut copies = prepared_fixture(FIXTURE, NAMES);
                let originals = copies
                    .keys()
                    .map(|path| (path.clone(), path.clone()))
                    .collect();
                let mut output = super::super::ProjectedCalls::new();
                for profile in projections {
                    super::super::merge::apply(
                        &mut copies,
                        &originals,
                        &profile.options,
                        profile.projection,
                        &mut output,
                    );
                }
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
            let expected = apply(reference);
            assert_eq!(
                expected,
                apply(parallel),
                "jobs={jobs}, executed={collect_executed}"
            );
            for index in 0..8 {
                let (_, profiles) = expected
                    .0
                    .iter()
                    .find(|(path, _)| {
                        path.file_stem().unwrap() == format!("entry-{index}").as_str()
                    })
                    .unwrap();
                let calls = &profiles
                    .iter()
                    .find(|(options, _)| options.import_specifier == "@app/db")
                    .unwrap()
                    .1;
                assert_eq!(calls.len(), 3);
                assert!(calls
                    .values()
                    .any(|value| value.as_deref() == Some(format!("SELECT {index}").as_str())));
                assert!(calls
                    .values()
                    .any(|value| value.as_deref() == Some("SELECT 90")));
            }
            if collect_executed {
                for index in 0..8 {
                    assert!(expected
                        .1
                        .values()
                        .filter_map(|result| result.as_ref().ok())
                        .flatten()
                        .any(|sql| sql.complete && sql.text == format!("SELECT {index}")));
                }
            } else {
                assert!(expected.1.is_empty());
            }
        }
    }
}
