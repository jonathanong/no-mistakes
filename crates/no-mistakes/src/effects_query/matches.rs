impl EffectCallSite {
    fn add_match(&mut self, matched: EffectCallAlternative) {
        if self.callee == matched.callee && self.category == matched.category {
            return;
        }
        self.alternatives
            .get_or_insert_with(|| {
                vec![EffectCallAlternative {
                    callee: self.callee.clone(),
                    category: self.category.clone(),
                }]
            })
            .push(matched);
    }

    fn normalize_matches(&mut self) {
        if let Some(alternatives) = &mut self.alternatives {
            alternatives.sort();
            alternatives.dedup();
            let first = &alternatives[0];
            self.callee.clone_from(&first.callee);
            self.category.clone_from(&first.category);
        }
    }

    fn count_categories(&self, counts: &mut BTreeMap<String, usize>) {
        if let Some(alternatives) = &self.alternatives {
            let categories = alternatives
                .iter()
                .map(|matched| matched.category.as_deref().unwrap_or("uncategorized"))
                .collect::<std::collections::BTreeSet<_>>();
            for category in categories {
                *counts.entry(category.to_string()).or_insert(0) += 1;
            }
        } else {
            *counts
                .entry(
                    self.category
                        .clone()
                        .unwrap_or_else(|| "uncategorized".into()),
                )
                .or_insert(0) += 1;
        }
    }
}

fn needs_resolved_effects(selection: &EffectsSelection) -> bool {
    !selection.targets.is_empty()
        || !selection.returned.is_empty()
        || !selection.provenance.is_empty()
        || !selection.names.is_empty()
}

fn visit_effect_matches(
    selection: &EffectsSelection,
    site: &crate::codebase::dependencies::graph::ResolvedCallSite,
    target_nodes: &crate::fx::FxHashMap<NodeId, &crate::config::v2::schema::EffectTargetConfig>,
    record: &mut impl FnMut(EffectCallAlternative),
) {
    use crate::codebase::dependencies::graph::KnownCallTargetRef;
    let multiple = has_effect_alternatives(site);
    for candidate in site.targets() {
        let matched = match candidate.target {
            KnownCallTargetRef::ModuleExport {
                specifier,
                export_path,
                ..
            } => selection
                .names
                .get(export_path)
                .filter(|_| export_path.contains('.'))
                .map(|category| EffectCallAlternative {
                    callee: export_path.to_string(),
                    category: category.clone(),
                })
                .or_else(|| {
                    selection
                        .targets
                        .get(specifier)
                        .and_then(|exports| exports.get(export_path))
                        .map(effect_alternative)
                })
                // Explicit module categories keep precedence over a plain
                // name admitted only by the alternatives projection.
                .or_else(|| {
                    selection.names.get(export_path).filter(|_| multiple).map(
                        |category| EffectCallAlternative {
                            callee: export_path.to_string(),
                            category: category.clone(),
                        },
                    )
                }),
            KnownCallTargetRef::RepositoryFunction { .. } => candidate
                .node
                .and_then(|node| target_nodes.get(node).copied())
                .map(effect_alternative),
            KnownCallTargetRef::Global { name } if multiple => {
                selection
                    .names
                    .get(name)
                    .map(|category| EffectCallAlternative {
                        callee: name.to_string(),
                        category: category.clone(),
                    })
            }
            _ => None,
        };
        if let Some(matched) = matched {
            record(matched);
            if !multiple {
                return;
            }
        }
    }
    for target in site
        .returned_from
        .iter()
        .filter_map(|(module, export)| {
            selection
                .returned
                .get(module)
                .and_then(|exports| exports.get(export))
        })
        .chain(selection.provenance.matching_sites(site))
    {
        record(effect_alternative(target));
        if !multiple {
            return;
        }
    }
}

fn effect_alternative(
    target: &crate::config::v2::schema::EffectTargetConfig,
) -> EffectCallAlternative {
    EffectCallAlternative {
        callee: target.export.clone(),
        category: target.category.clone(),
    }
}

fn has_effect_alternatives(site: &crate::codebase::dependencies::graph::ResolvedCallSite) -> bool {
    let known = site.targets().len();
    // A wrapper containing one known identity alone is still a singleton.
    // Unknown possibilities add no effects, but retain distinct trusted metadata.
    known > 1 || (known > 0 && site.has_unknown_target())
}
