use super::*;

/// A component-level React diagnostic covers every local fetch. Preserve its
/// single stable public finding unless all of those fetches are suppressed.
struct ReactSuppressionFinding {
    finding: react_traits::Violation,
    line: Option<usize>,
    identity: String,
}

pub(crate) fn suppress_react(
    root: &std::path::Path,
    sources: &SourceStore,
    findings: &mut Vec<react_traits::Violation>,
    suppression_targets: &[Vec<react_traits::ReactSuppressionTarget>],
    suppressed: &mut Vec<SuppressedFinding>,
) {
    let original_findings = findings.drain(..).enumerate().collect::<Vec<_>>();
    for (index, mut finding) in original_findings {
        let identity = format!("{}@{}", finding.component, finding.file);
        let targets = suppression_targets.get(index).cloned().unwrap_or_default();
        if !targets.is_empty() {
            // An inherited fetch still belongs to the parent component. Honor a
            // parent file directive before evaluating each child fetch location.
            let mut parent_location = vec![ReactSuppressionFinding {
                finding: finding.clone(),
                line: None,
                identity: identity.clone(),
            }];
            let parent_suppressed = suppress_domain_findings_with_sources(
                root,
                &mut parent_location,
                sources,
                react_target,
            );
            if parent_location.is_empty() {
                suppressed.extend(parent_suppressed);
                continue;
            }
        }
        let mut locations = if !targets.is_empty() {
            targets
                .iter()
                .map(|target| ReactSuppressionFinding {
                    finding: react_traits::Violation {
                        file: target.file.clone(),
                        ..finding.clone()
                    },
                    line: Some(target.line),
                    identity: identity.clone(),
                })
                .collect()
        } else {
            vec![ReactSuppressionFinding {
                finding: finding.clone(),
                line: None,
                identity,
            }]
        };
        let target_suppressions =
            suppress_domain_findings_with_sources(root, &mut locations, sources, react_target);
        if locations.is_empty() {
            // A React violation is one component-level diagnostic even when
            // several fetch locations contribute to it. Keep one deterministic
            // directive record only after every contributing location is hidden.
            suppressed.extend(target_suppressions.into_iter().next());
        } else {
            clear_suppressed_first_fetch_detail(&mut finding, &targets, &locations);
            findings.push(finding);
        }
    }
}

fn clear_suppressed_first_fetch_detail(
    finding: &mut react_traits::Violation,
    targets: &[react_traits::ReactSuppressionTarget],
    locations: &[ReactSuppressionFinding],
) {
    let Some(first_target) = targets.first() else {
        return;
    };
    let first_target_retained = locations.iter().any(|location| {
        location.finding.file == first_target.file && location.line == Some(first_target.line)
    });
    if !first_target_retained {
        // The public detail belongs to the first local fetch. Once that target
        // is hidden, do not report it for another fetch.
        finding.detail = None;
    }
}

fn react_target(entry: &ReactSuppressionFinding) -> SuppressionTarget<'_> {
    let finding = &entry.finding;
    SuppressionTarget {
        domain: "react",
        rule: &finding.rule,
        file: &finding.file,
        line: entry.line,
        reason: finding
            .detail
            .as_deref()
            .unwrap_or("component fetch assertion failed"),
        identity: Some(&entry.identity),
    }
}
