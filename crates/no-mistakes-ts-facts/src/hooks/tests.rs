use super::{
    record_work_with, trace_current_file_with, trace_file_with, DiagnosticsHooks, FilePass,
};
use std::path::Path;
use std::sync::Mutex;

#[derive(Default)]
struct RecordingHooks {
    events: Mutex<Vec<String>>,
}

impl DiagnosticsHooks for RecordingHooks {
    fn record_work(&self, metric: &'static str, count: u64) {
        self.events
            .lock()
            .unwrap()
            .push(format!("work:{metric}:{count}"));
    }

    fn trace_file(&self, pass: FilePass, path: &Path, units: u64, operation: &mut dyn FnMut()) {
        self.events
            .lock()
            .unwrap()
            .push(format!("file:{pass:?}:{}:{units}", path.display()));
        operation();
    }

    fn trace_current_file(&self, pass: FilePass, units: u64, operation: &mut dyn FnMut()) {
        self.events
            .lock()
            .unwrap()
            .push(format!("current:{pass:?}:{units}"));
        operation();
    }
}

#[test]
fn hooks_default_to_passthrough_and_forward_when_registered() {
    record_work_with(None, "noop", 1);
    assert_eq!(
        trace_file_with(None, FilePass::Parse, Path::new("a.ts"), 3, || 7),
        7
    );
    assert_eq!(
        trace_current_file_with(None, FilePass::ValueFlowExtract, 5, || 11),
        11
    );

    let hooks = RecordingHooks::default();
    record_work_with(Some(&hooks), "facts.call_arguments.collections", 2);
    assert_eq!(
        trace_file_with(Some(&hooks), FilePass::Parse, Path::new("b.ts"), 9, || 13),
        13
    );
    assert_eq!(
        trace_current_file_with(Some(&hooks), FilePass::ValueFlowFinalize, 4, || 17),
        17
    );
    assert_eq!(
        *hooks.events.lock().unwrap(),
        vec![
            "work:facts.call_arguments.collections:2",
            "file:Parse:b.ts:9",
            "current:ValueFlowFinalize:4",
        ]
    );
}
