use super::*;

pub(in crate::tests::configured_plan) fn merge_lockfile_seed_candidates(
    root: &Path,
    seeds: &[SelectedTest],
    candidates: &mut Vec<SelectedTest>,
    used: &HashSet<String>,
    selected: &mut BTreeMap<PathBuf, SelectedTest>,
) {
    for seed in seeds {
        if used.contains(&seed.test_file) {
            if let Some(existing) = selected.get_mut(&root.join(&seed.test_file)) {
                merge_selected(existing, seed);
                existing.targets.clear();
            }
            continue;
        }
        if let Some(existing) = candidates
            .iter_mut()
            .find(|candidate| candidate.test_file == seed.test_file)
        {
            merge_selected(existing, seed);
            existing.targets.clear();
        } else {
            candidates.push(seed.clone());
        }
    }
}
