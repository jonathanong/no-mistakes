use super::config::{compile_selection_glob, from_path_parts, ChangesConfig, IdRule};
use crate::codebase::keyed_diff::KeyOptions;
use anyhow::{bail, Context, Result};
use globset::{GlobSet, GlobSetBuilder};

pub(super) struct Rules<'a> {
    pub excludes: Vec<GlobSet>,
    pub selectors: GlobSet,
    pub rules: Vec<Rule<'a>>,
    pub keyed: Vec<KeyOptions>,
}

pub(super) struct Rule<'a> {
    pub category: &'a str,
    pub exclude: Option<usize>,
    pub id: Id<'a>,
    pub emit_attribute_paths: bool,
}

impl Rule<'_> {
    pub fn is_excluded(&self, path: &str, excludes: &[GlobSet]) -> bool {
        self.exclude
            .is_some_and(|index| excludes[index].is_match(path))
    }
}

#[derive(Clone, Copy)]
pub(super) enum Id<'a> {
    Path,
    FromPath {
        prefix: &'a str,
        suffix: &'a str,
        separator: Option<&'a str>,
    },
    Keys(usize),
}

impl<'a> Rules<'a> {
    pub fn compile(config: &'a ChangesConfig) -> Result<Self> {
        let mut excludes = Vec::new();
        let mut selectors = GlobSetBuilder::new();
        let mut rules = Vec::new();
        let mut keyed = Vec::new();
        for (name, category) in &config.categories {
            if name.is_empty()
                || matches!(
                    name.as_str(),
                    "schemaVersion" | "complete" | "unclassified" | "changedFiles" | "reasons"
                )
            {
                bail!("invalid changes category name {name:?}");
            }
            let exclude = if category.exclude.is_empty() {
                None
            } else {
                let mut builder = GlobSetBuilder::new();
                for pattern in &category.exclude {
                    let glob = compile_selection_glob(pattern)
                        .with_context(|| format!("changes category {name:?} exclude"))?;
                    builder.add(glob);
                }
                let index = excludes.len();
                excludes.push(builder.build()?);
                Some(index)
            };
            let definitions = std::iter::once((&category.paths, category.id.as_ref()))
                .filter(|(paths, id)| !paths.is_empty() || id.is_some())
                .chain(
                    category
                        .rules
                        .iter()
                        .map(|rule| (&rule.paths, rule.id.as_ref())),
                );
            for (paths, id) in definitions {
                if paths.is_empty() {
                    bail!("changes category {name:?} rule requires paths");
                }
                let id = compile_id(id, &mut keyed)
                    .with_context(|| format!("changes category {name:?} ID rule"))?;
                for pattern in paths {
                    let glob = compile_selection_glob(pattern)
                        .with_context(|| format!("changes category {name:?} paths"))?;
                    selectors.add(glob);
                    rules.push(Rule {
                        category: name,
                        exclude,
                        id,
                        emit_attribute_paths: category.assert_pure_json_loader,
                    });
                }
            }
        }
        Ok(Self {
            excludes,
            selectors: selectors.build()?,
            rules,
            keyed,
        })
    }
}

fn compile_id<'a>(id: Option<&'a IdRule>, keyed: &mut Vec<KeyOptions>) -> Result<Id<'a>> {
    let Some(id) = id else {
        return Ok(Id::Path);
    };
    match (id.from_path.as_deref(), id.keys_of.as_ref()) {
        (Some(template), None) => {
            let (prefix, suffix) = from_path_parts(template)
                .context("fromPath requires exactly one {id} placeholder")?;
            Ok(Id::FromPath {
                prefix,
                suffix,
                separator: id.separator.as_deref(),
            })
        }
        (None, Some(keys)) if id.separator.is_none() => {
            let options = keys.options();
            options.validate()?;
            let index = keyed
                .iter()
                .position(|existing| existing == &options)
                .unwrap_or_else(|| {
                    let index = keyed.len();
                    keyed.push(options);
                    index
                });
            Ok(Id::Keys(index))
        }
        _ => bail!("declare either fromPath or keysOf, with separator only for fromPath"),
    }
}
