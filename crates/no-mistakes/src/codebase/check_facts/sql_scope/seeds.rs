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
    let selected = |path: &PathBuf| {
        #[cfg(test)]
        if let Some(observer) = crate::diagnostics::current() {
            observer.increment("sql_scope.seed_checks", 1);
        }
        let relative = crate::codebase::ts_source::relative_slash_path(&request.root, path);
        request
            .plan
            .embedded_sql_rule_ids
            .values()
            .flatten()
            .any(|scan| scan.paths.includes(&relative))
    };
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
