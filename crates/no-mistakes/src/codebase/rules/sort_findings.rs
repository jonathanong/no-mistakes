use super::RuleFinding;

pub(crate) fn sort_findings(findings: &mut Vec<RuleFinding>) {
    findings.sort();
    findings.dedup();
}

pub(super) fn sort_by_file(findings: &mut [RuleFinding]) {
    findings.sort_by(|a, b| a.file.cmp(&b.file));
}

pub(super) fn sort_by_file_and_message(findings: &mut [RuleFinding]) {
    findings.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));
}
