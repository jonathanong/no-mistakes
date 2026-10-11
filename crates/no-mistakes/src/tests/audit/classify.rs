use super::super::SelectedTest;
use super::*;

pub(super) enum ClassifiedTrace {
    Selected(TestAuditExecutionEvidence),
    Missed(TestAuditExecutionEvidence),
    WithoutExecution(TestAuditSelectionEvidence),
    Incomplete(String),
}

pub(super) fn trace(
    trace: &TestAuditObservation,
    selected: Option<&SelectedTest>,
    file_changes: &BTreeSet<&str>,
    symbol_changes: &BTreeSet<&TestAuditSymbol>,
) -> Option<ClassifiedTrace> {
    let matched_files: Vec<_> = trace
        .executed_files
        .iter()
        .map(String::as_str)
        .filter(|file| file_changes.contains(file))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(str::to_string)
        .collect();
    let matched_symbols: Vec<_> = trace
        .executed_symbols
        .iter()
        .filter(|symbol| symbol_changes.contains(symbol))
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    if !matched_files.is_empty() || !matched_symbols.is_empty() {
        let evidence = TestAuditExecutionEvidence {
            test_file: trace.test_file.clone(),
            matched_files,
            matched_symbols,
        };
        return Some(if selected.is_some() {
            ClassifiedTrace::Selected(evidence)
        } else {
            ClassifiedTrace::Missed(evidence)
        });
    }
    selected.map(|test| {
        if trace.trace_complete {
            ClassifiedTrace::WithoutExecution(TestAuditSelectionEvidence {
                test_file: trace.test_file.clone(),
                reasons: test.reasons.clone(),
            })
        } else {
            ClassifiedTrace::Incomplete(trace.test_file.clone())
        }
    })
}
