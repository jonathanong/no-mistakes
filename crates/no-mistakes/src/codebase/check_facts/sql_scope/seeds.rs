use super::*;

pub(super) fn select(
    request: &BatchCheckFactRequest,
    id: usize,
    demands: &BTreeMap<PathBuf, Vec<FactDemand>>,
) -> Option<BTreeSet<PathBuf>> {
    let eligible = |variants: &Vec<FactDemand>| {
        variants
            .iter()
            .any(|variant| variant.request == id && variant.plan.embedded_sql)
    };
    let selected = |path: &PathBuf| selected(&request.root, &request.plan, path);
    let mut files = demands.iter().filter(|(_, variants)| eligible(variants));
    let excluded = files.find(|(path, _)| !selected(path))?.0;
    Some(
        demands
            .range(..excluded.clone())
            .filter(|(_, variants)| eligible(variants))
            .map(|(path, _)| path.clone())
            .chain(
                files
                    .filter(|(path, _)| selected(path))
                    .map(|(path, _)| path.clone()),
            )
            .collect(),
    )
}

fn selected(root: &std::path::Path, plan: &CheckFactPlan, path: &std::path::Path) -> bool {
    record_seed_check!();
    let relative = crate::codebase::ts_source::relative_slash_path(root, path);
    plan.embedded_sql_rule_ids
        .values()
        .flatten()
        .any(|scan| scan.paths.includes(&relative))
}

pub(in crate::codebase::check_facts) fn single(
    root: &std::path::Path,
    files: &[PathBuf],
    plan: &CheckFactPlan,
) -> Option<BTreeSet<PathBuf>> {
    let eligible = |path: &&PathBuf| {
        crate::codebase::dependencies::extract::is_indexable(path)
            || plan.production_import_paths.contains(*path)
            || plan.storybook && super::super::is_mdx_file(path)
    };
    let mut seeds = None;
    let mut previous = None;
    for (index, path) in files.iter().enumerate().filter(|(_, path)| eligible(path)) {
        let path = crate::codebase::ts_resolver::normalize_path(path);
        if previous.as_ref().is_some_and(|previous| previous >= &path) {
            return None;
        }
        previous = Some(path.clone());
        if selected(root, plan, &path) {
            if let Some(seeds) = &mut seeds {
                BTreeSet::insert(seeds, path);
            }
        } else if seeds.is_none() {
            seeds = Some(
                files[..index]
                    .iter()
                    .filter(eligible)
                    .map(|path| crate::codebase::ts_resolver::normalize_path(path))
                    .collect(),
            );
        }
    }
    if seeds.is_some() {
        record_sql_delegation!();
    }
    seeds
}
