use super::*;

/// A component-level React diagnostic covers every local fetch. Preserve its
/// single stable public finding unless all of those fetches are suppressed.
struct ReactSuppressionFinding {
    finding: react_traits::Violation,
    line: Option<usize>,
    source_location: Option<(String, usize)>,
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
                source_location: None,
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
                    // Same-file fetches are the public diagnostic location.
                    // For inherited fetches, keep the parent as the target
                    // and use the child fetch only as directive provenance.
                    finding: finding.clone(),
                    line: (target.file == finding.file).then_some(target.line),
                    source_location: (target.file != finding.file)
                        .then(|| (target.file.clone(), target.line)),
                    identity: identity.clone(),
                })
                .collect()
        } else {
            vec![ReactSuppressionFinding {
                finding: finding.clone(),
                line: None,
                source_location: None,
                identity,
            }]
        };
        let target_suppressions = suppress_domain_findings_with_source_locations(
            root,
            &mut locations,
            sources,
            react_target,
            |location| {
                location
                    .source_location
                    .as_ref()
                    .map(|(file, line)| (file.as_str(), Some(*line)))
            },
        );
        if locations.is_empty() {
            // A React violation is one component-level diagnostic even when
            // several fetch locations contribute to it. Keep one deterministic
            // directive record for the first contributing location only after
            // every location is hidden.
            let first_target = targets
                .iter()
                .find(|target| target.file == finding.file)
                .or_else(|| targets.first())
                .expect("non-empty suppression targets have a first target");
            let first_suppression = target_suppressions.iter().find(|suppression| {
                suppression.file == finding.file
                    && suppression.source_file == first_target.file
                    && if first_target.file == finding.file {
                        suppression.line == Some(first_target.line)
                    } else {
                        suppression.line.is_none()
                    }
            });
            suppressed.extend(first_suppression.or(target_suppressions.first()).cloned());
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
        location.source_location.as_ref().map_or_else(
            || {
                location.finding.file == first_target.file
                    && location.line == Some(first_target.line)
            },
            |(file, line)| file == &first_target.file && *line == first_target.line,
        )
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
