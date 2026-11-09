use super::*;

type SharedSites = BTreeMap<SiteKey, (BTreeSet<String>, Reasons)>;

pub(super) fn finish(root: &Path, exports: ExportEvidence) -> AdvisoryGroups {
    let mut shared = SharedSites::new();
    for (name, sites) in exports.values() {
        for (site, reasons) in sites {
            let (names, all_reasons) = shared.entry(site.clone()).or_default();
            names.insert(name.clone());
            all_reasons.extend(reasons.iter().copied());
        }
    }
    let mut compressed = Vec::new();
    for (name, sites) in exports.values() {
        let individual: Vec<_> = sites
            .iter()
            .filter(|(site, _)| shared[*site].0.len() == 1)
            .collect();
        if !individual.is_empty() {
            compressed.push(row(root, &[name], &individual));
        }
    }
    for (site, (names, reasons)) in &shared {
        if names.len() > 1 {
            let names: Vec<_> = names.iter().collect();
            compressed.push(row(root, &names, &[(site, reasons)]));
        }
    }
    let baseline: Vec<_> = exports
        .values()
        .map(|(name, sites)| row(root, &[name], &sites.iter().collect::<Vec<_>>()))
        .collect();
    AdvisoryGroups {
        baseline,
        compressed,
    }
}

fn row(root: &Path, names: &[&String], sites: &[(&SiteKey, &Reasons)]) -> (PathBuf, usize, String) {
    let (first, _) = sites.first().expect("uncertain exports have site evidence");
    let single = names.len() == 1;
    let names = names
        .iter()
        .map(|name| format!("`{name}`"))
        .collect::<Vec<_>>()
        .join(", ");
    let details = sites
        .iter()
        .map(|((file, line, specifier), reasons)| {
            let path = relative_slash_path(root, file);
            let reasons = reasons.iter().copied().collect::<Vec<_>>().join(", ");
            match specifier {
                Some(specifier) => format!("{path}:{line} `{specifier}` ({reasons})"),
                None => format!("{path}: {reasons}"),
            }
        })
        .collect::<Vec<_>>()
        .join(", ");
    (
        first.0.clone(),
        first.1,
        if single {
            format!("symbol consumers of {names} include an unknown import selector: {details}")
        } else {
            format!("production export consumers for {names} include unknown import selectors: {details}")
        },
    )
}
