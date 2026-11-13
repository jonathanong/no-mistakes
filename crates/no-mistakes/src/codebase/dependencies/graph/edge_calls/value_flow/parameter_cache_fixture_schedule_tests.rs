use super::*;
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::time::Duration;

static CONFIGURED: Mutex<Option<(PathBuf, Hook)>> = Mutex::new(None);
static EXCLUSIVE: Mutex<()> = Mutex::new(());

pub(super) fn max_task_len(root: &std::path::Path) -> usize {
    if CONFIGURED
        .lock()
        .unwrap()
        .as_ref()
        .is_some_and(|(selected, _)| selected == root)
    {
        1
    } else {
        usize::MAX
    }
}

pub(super) fn configured(key: &ParameterKey) -> Option<Hook> {
    CONFIGURED
        .lock()
        .unwrap()
        .as_ref()
        .filter(|(root, _)| key.file.starts_with(root))
        .map(|(_, hook)| Arc::clone(hook))
}

struct Reset;
impl Drop for Reset {
    fn drop(&mut self) {
        *CONFIGURED.lock().unwrap() = None;
    }
}

#[derive(Default)]
struct Schedule {
    owned: FxHashMap<ThreadId, ParameterKey>,
    caller_owner: Option<ThreadId>,
    waiter: Option<ThreadId>,
    blocked: Option<ParameterKey>,
    released: bool,
    observed: bool,
    resumed: bool,
    cycle: bool,
    timed_out: bool,
}

fn hook(shared: &Arc<(Mutex<Schedule>, Condvar)>, event: Event, key: &ParameterKey) {
    if key.name != "query" {
        return;
    }
    let file = key.file.file_name().unwrap().to_str().unwrap();
    let current = std::thread::current().id();
    let (lock, changed) = shared.as_ref();
    let mut state = lock.lock().unwrap();
    if file.starts_with("stages-") && event == Event::Computing {
        state.owned.insert(current, key.clone());
    } else if file == "caller.mts"
        && event == Event::Computing
        && state.caller_owner.is_none()
        && state.owned.contains_key(&current)
    {
        state.caller_owner = Some(current);
        let (resumed, timeout) = changed
            .wait_timeout_while(state, Duration::from_secs(10), |state| {
                state.waiter.is_none()
            })
            .unwrap();
        state = resumed;
        state.timed_out |= timeout.timed_out();
    } else if file == "caller.mts"
        && event == Event::Waiting
        && state.caller_owner.is_some_and(|owner| owner != current)
        && state.waiter.is_none()
    {
        if let Some(stage) = state.owned.get(&current).cloned() {
            state.waiter = Some(current);
            state.blocked = Some(stage);
            changed.notify_all();
        }
    } else if file == "caller.mts" && event == Event::Resuming && state.waiter == Some(current) {
        state.resumed = true;
        let (resumed, timeout) = changed
            .wait_timeout_while(state, Duration::from_secs(10), |state| !state.released)
            .unwrap();
        state = resumed;
        state.timed_out |= timeout.timed_out();
    } else if matches!(event, Event::Waiting | Event::Cycle)
        && state.caller_owner == Some(current)
        && state.blocked.as_ref() == Some(key)
    {
        state.observed = true;
        state.cycle = event == Event::Cycle;
        state.released = true;
        changed.notify_all();
    }
}

#[test]
fn saved_stage_workload_schedule_preserves_all_48_exact_findings() {
    let _exclusive = EXCLUSIVE.lock().unwrap();
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/query-reached-per-item/stage-import-index-workload");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let shared = Arc::new((Mutex::new(Schedule::default()), Condvar::new()));
    let observed = Arc::clone(&shared);
    *CONFIGURED.lock().unwrap() = Some((
        fixture.path().to_path_buf(),
        Arc::new(move |event, key| hook(&observed, event, key)),
    ));
    let _reset = Reset;
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(2)
        .build()
        .unwrap();
    let findings = pool
        .install(|| crate::codebase::rules::run_check(fixture.path(), None, None))
        .unwrap();
    let state = shared.0.lock().unwrap();
    let blocked = state.blocked.as_ref().map(|key| (&key.file, key.scope));
    assert!(!state.timed_out, "schedule timed out: blocked={blocked:?}");
    assert!(
        state.observed && state.resumed,
        "schedule was not exercised: blocked={blocked:?}"
    );
    let actual = findings
        .iter()
        .map(|finding| {
            (
                finding.file.as_str(),
                finding.line,
                finding.import.as_deref().unwrap_or_default(),
            )
        })
        .collect::<BTreeSet<_>>();
    let expected = (0..24)
        .flat_map(|stage| {
            [
                ("src/caller.mts", 28 + stage * 4, "stage.run"),
                ("src/caller.mts", 29 + stage * 4, "stage.check"),
            ]
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(
        actual,
        expected,
        "blocked={blocked:?}; cycle={}; missing={:?}; findings={findings:#?}",
        state.cycle,
        expected.difference(&actual).collect::<Vec<_>>()
    );
    assert_eq!(findings.len(), 48);
    assert!(
        !state.cycle,
        "published dependency caused a cycle: {blocked:?}"
    );
}
