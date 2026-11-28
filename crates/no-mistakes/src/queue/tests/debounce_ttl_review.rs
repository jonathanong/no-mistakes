use super::resolver::extract_file_with_factories;
use std::path::PathBuf;

fn line_text(source: &str, line: usize) -> String {
    source
        .lines()
        .nth(line.saturating_sub(1))
        .unwrap_or("")
        .trim()
        .to_string()
}

#[test]
fn review_cases_find_known_ttl_and_abstain_when_unknown() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/queue-ast-hop/debounce-ttl-review/fixture/cases.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let facts = extract_file_with_factories(&path, &[]).unwrap();
    let findings: Vec<_> = facts
        .checks
        .iter()
        .filter(|check| check.kind == "debounce-ttl")
        .map(|check| line_text(&source, check.line))
        .collect();
    let abstained: Vec<_> = facts
        .check_abstentions
        .iter()
        .map(|(line, _)| line_text(&source, *line))
        .collect();
    for marker in [
        "alias-number",
        "spread-then-key",
        "bulk-const",
        "visible-child",
        "shadow-ttl",
        "satisfies-opts",
        "assert-opts",
        "computed-then-explicit",
        "bulk-as",
        "bulk-rest",
        "flow-as",
    ] {
        assert!(
            findings.iter().any(|line| line.contains(marker)),
            "{marker} should be a finding, findings={findings:?} abstained={abstained:?}"
        );
    }
    for marker in ["alias-absent", "bulk-clean", "\"plain\"", "void-ttl"] {
        assert!(
            findings.iter().all(|line| !line.contains(marker))
                && abstained.iter().all(|line| !line.contains(marker)),
            "{marker} should stay clean, findings={findings:?} abstained={abstained:?}"
        );
    }
    for marker in [
        "later-spread",
        "dynamicJobs",
        "getQueueOptions",
        "cond-ttl",
        "member-ttl",
        "call-ttl",
        "computed-key",
        "bulk-spread-rest",
    ] {
        assert!(
            abstained.iter().any(|line| line.contains(marker)),
            "{marker} should abstain, findings={findings:?} abstained={abstained:?}"
        );
        assert!(
            findings.iter().all(|line| !line.contains(marker)),
            "{marker} should not be a finding, findings={findings:?}"
        );
    }
}
