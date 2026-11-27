use super::comparison::{check_manifest, CheckContext};
use super::manifests::{collect, dependency_fields, Manifest};
use super::{Options, RULE_ID};
use no_mistakes_rule_kernel::finding::RuleFinding;
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub(super) fn check_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &SourceStore,
) -> Result<Vec<RuleFinding>> {
    super::super::selected_files::run_parallel_rule_applications(config, RULE_ID, &|rule| {
        scan(root, &rule.try_rule_options()?, all_files, sources)
    })
}

fn scan(
    root: &Path,
    opts: &Options,
    files: &[PathBuf],
    sources: &SourceStore,
) -> Result<Vec<RuleFinding>> {
    if opts.roots.is_empty() || opts.dependency_name_prefixes.is_empty() {
        return Ok(Vec::new());
    }
    let manifests = collect(root, files, sources);
    let roots = root_globs(&opts.roots)?;
    let roots: Vec<_> = manifests
        .iter()
        .filter(|manifest| roots.is_match(relative_slash_path(root, &manifest.dir)))
        .cloned()
        .collect();
    if roots.is_empty() {
        return Ok(Vec::new());
    }
    let by_name = manifests
        .iter()
        .filter_map(|manifest| manifest.name.as_ref().map(|name| (name.clone(), manifest)))
        .fold(
            BTreeMap::<String, Vec<&Manifest>>::new(),
            |mut map, (name, manifest)| {
                map.entry(name).or_default().push(manifest);
                map
            },
        );
    let fields = dependency_fields(&opts.dependency_fields);
    let context = CheckContext {
        root,
        opts,
        manifests: &manifests,
        by_name: &by_name,
        fields: &fields,
        sources,
    };
    let mut findings = Vec::new();
    for manifest in roots {
        check_manifest(&context, &manifest, &mut findings)?;
    }
    super::super::sort_findings::sort_by_file_and_message(&mut findings);
    Ok(findings)
}

fn root_globs(roots: &[String]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for root in roots {
        builder.add(
            GlobBuilder::new(&crate::codebase::glob_normalize::normalize(root))
                .literal_separator(true)
                .build()?,
        );
    }
    Ok(builder.build()?)
}
