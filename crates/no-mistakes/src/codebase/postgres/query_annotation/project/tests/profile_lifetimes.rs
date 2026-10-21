use super::*;
use std::sync::mpsc;
use std::time::{Duration, Instant};

#[test]
fn completed_profile_releases_its_unique_module_while_another_root_is_running() {
    let mut files = prepared_fixture(
        "helper-tracing-profile-lifetimes",
        &["fast", "slow-entry", "slow"],
    );
    for (path, file) in &mut files {
        let wanted = if path.ends_with("fast.mts") {
            "@app/db"
        } else {
            "@app/other"
        };
        file.query_annotation
            .retain(|(options, _)| options.import_specifier == wanted);
        file.embedded_sql
            .retain(|(options, _)| options.import_specifier == wanted);
    }
    let candidates = files.keys().cloned().collect::<BTreeSet<_>>();
    let resolve = |specifier: &str, from: &std::path::Path| {
        let path = from.parent().unwrap().join(specifier).with_extension("mts");
        candidates.contains(&path).then_some(path)
    };
    let profiles = super::super::prepare::profiles(&files, &resolve);
    assert_eq!(profiles.len(), 2);
    assert!(profiles.iter().all(|profile| profile.roots.len() == 1));
    let fast = profiles[0]
        .files
        .iter()
        .find(|(path, _)| path.ends_with("fast.mts"))
        .unwrap();
    let weak = Arc::downgrade(&fast.1.module);
    assert_eq!(
        weak.strong_count(),
        1,
        "the observed module belongs to only the fast profile"
    );
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let release_rx = Mutex::new(release_rx);
    let gated = |specifier: &str, from: &std::path::Path| {
        if specifier == "./slow.mjs" {
            entered_tx.send(()).unwrap();
            release_rx.lock().unwrap().recv().unwrap();
        }
        resolve(specifier, from)
    };
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(2)
        .build()
        .unwrap();
    std::thread::scope(|threads| {
        let running = threads
            .spawn(|| pool.install(|| super::super::parallel::evaluate(profiles, &gated, false)));
        let entered = entered_rx.recv_timeout(Duration::from_secs(10)).is_ok();
        let deadline = Instant::now() + Duration::from_secs(10);
        while entered && weak.strong_count() != 0 && Instant::now() < deadline {
            std::thread::park_timeout(Duration::from_millis(1));
        }
        let released_early = weak.strong_count() == 0;
        // Release before assertions, including the failing old implementation,
        // so a lifetime regression cannot leave a worker blocked at the gate.
        let _ = release_tx.send(());
        let result = running.join().unwrap();
        assert!(
            entered,
            "the known executor must reach the helper at runtime"
        );
        assert!(
            released_early,
            "completed profile state survived until the pass ended"
        );
        assert_eq!(result.len(), 2);
    });
}

#[path = "module_ids.rs"]
mod module_ids;
