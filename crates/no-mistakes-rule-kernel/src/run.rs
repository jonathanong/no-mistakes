use crate::finding::RuleFinding;

#[doc(hidden)]
pub struct PreparedRuleFindings {
    pub findings: Vec<RuleFinding>,
    pub suppression_sources: Vec<Option<String>>,
    pub advisories: Vec<RuleFinding>,
}

pub fn finalize_findings(
    findings: Vec<RuleFinding>,
    suppression_sources: Vec<Option<String>>,
    mut advisories: Vec<RuleFinding>,
) -> PreparedRuleFindings {
    let mut paired = findings
        .into_iter()
        .zip(suppression_sources)
        .collect::<Vec<_>>();
    paired.sort_by(|(a, _), (b, _)| a.cmp(b));
    paired.dedup_by(|(a, source_a), (b, source_b)| a == b && source_a == source_b);
    let (findings, suppression_sources): (Vec<_>, Vec<_>) = paired.into_iter().unzip();
    advisories.sort();
    advisories.dedup();
    PreparedRuleFindings {
        findings,
        suppression_sources,
        advisories,
    }
}
