use super::{field::Field, paths::Ref, RuleFinding, RULE_ID};
use crate::codebase::ts_source::SourceStore;
use crate::config::source_locations::{ConfigLocations, ConfigPath, Segment};
use crate::config::v2::NoMistakesConfig;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(super) struct Context<'a> {
    config: &'a NoMistakesConfig,
    file: String,
    locations: Option<Arc<ConfigLocations>>,
}

impl<'a> Context<'a> {
    pub(super) fn new(
        root: &Path,
        config: &'a NoMistakesConfig,
        files: &[PathBuf],
        sources: &SourceStore,
    ) -> Self {
        let file = config
            .loaded_config_path
            .relative_to(root)
            .unwrap_or_else(|| super::config_rel(root, files));
        // A caller-created config may have reordered rules; its disk neighbor is
        // not evidence of where those programmatic values originated.
        let locations = config
            .loaded_config_path
            .0
            .as_deref()
            .and_then(|path| sources.config_locations(path));
        Self {
            config,
            file,
            locations,
        }
    }

    pub(super) fn origins(&self) -> bool {
        self.locations.is_some()
    }

    pub(super) fn field(&self, key: &str) -> Field {
        Field::new(key, self.origins())
    }

    pub(super) fn finding_ref(&self, reference: &Ref, detail: String) -> RuleFinding {
        self.finding(
            &Field {
                label: reference.field.clone(),
                origin: reference.origin.clone(),
                rule_index: reference.rule_index,
            },
            detail,
        )
    }

    pub(super) fn finding(&self, field: &Field, detail: String) -> RuleFinding {
        let label = match field
            .rule_index
            .and_then(|index| self.config.rules.get(index).map(|rule| (index, rule)))
        {
            Some((index, rule)) => {
                let prefix = format!("rules[{index}]");
                format!("{prefix} ({}){}", rule.rule, &field.label[prefix.len()..])
            }
            None => field.label.clone(),
        };
        let line = self
            .locations
            .as_deref()
            .zip(field.origin.as_ref())
            .and_then(|(locations, path)| location_line(locations, path, self.config))
            .unwrap_or(1);
        RuleFinding {
            rule: RULE_ID.to_string(),
            file: self.file.clone(),
            line,
            message: format!("{label}{detail}"),
            import: None,
            target: None,
        }
    }
}

fn location_line(
    locations: &ConfigLocations,
    path: &ConfigPath,
    config: &NoMistakesConfig,
) -> Option<usize> {
    let parts = path.as_slice();
    let is_plan = matches!(parts.first(), Some(Segment::Key(key)) if key == "testPlan");
    let deprecated = is_plan
        && matches!(parts.get(2), Some(Segment::Key(key)) if key == "fullSuiteTriggers")
        && super::paths::frameworks(config)
            .into_iter()
            .any(|(framework, plan)| {
                matches!(parts.get(1), Some(Segment::Key(key)) if key == framework)
                    && plan.deprecated_dependencies_key
            });
    if !deprecated {
        if let Some(line) = locations.line(path) {
            return Some(line);
        }
    }
    // These are declared loader aliases/shapes, not a search for matching text.
    if !is_plan {
        return None;
    }
    for alias in [false, true] {
        for legacy in [false, true] {
            let mut alternate = ConfigPath::default();
            for (index, part) in parts.iter().enumerate() {
                if legacy
                    && index == 3
                    && matches!(part, Segment::Key(key) if key == "triggers")
                    && matches!(parts.get(2), Some(Segment::Key(key)) if key == "fullSuiteTriggers")
                {
                    continue;
                }
                alternate = match part {
                    Segment::Key(key) => alternate.key(if alias && index == 0 {
                        "test_plan"
                    } else if deprecated && index == 2 {
                        "dependencies"
                    } else {
                        key
                    }),
                    Segment::Index(index) => alternate.index(*index),
                };
            }
            if let Some(line) = locations.line(&alternate) {
                return Some(line);
            }
        }
    }
    None
}
