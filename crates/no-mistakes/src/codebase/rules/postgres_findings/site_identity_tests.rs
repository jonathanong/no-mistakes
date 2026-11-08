use super::SiteIdentities;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Deserialize)]
struct Case {
    groups: usize,
    projections: Vec<Vec<(u32, usize)>>,
    #[serde(default)]
    probe_limit: Option<usize>,
}

#[test]
fn only_identities_that_coexist_require_distinct_groups() {
    let source = std::fs::read_to_string(
        crate::test_support::rule_fixture_root("postgres-site-identity")
            .join("site-identities.json"),
    )
    .unwrap();
    let cases: BTreeMap<String, Case> = serde_json::from_str(&source).unwrap();
    for (name, case) in cases {
        let mut identities = SiteIdentities::default();
        for (projection, sites) in case.projections.iter().enumerate() {
            for &(site, copy) in sites {
                identities.record((), projection, (vec![site], copy));
            }
        }
        let mut probes = 0;
        let colors = identities.finish_with(|| probes += 1);
        if let Some(limit) = case.probe_limit {
            assert!(
                probes <= limit,
                "{name}: {probes} candidate probes exceeds {limit}"
            );
        }
        let colors = &colors[&()];
        assert_eq!(
            colors.values().copied().collect::<BTreeSet<_>>().len(),
            case.groups,
            "{name}"
        );
        for sites in case.projections {
            let assigned: BTreeSet<_> = sites
                .iter()
                .map(|&(site, copy)| colors[&(vec![site], copy)])
                .collect();
            assert_eq!(assigned.len(), sites.len(), "{name}");
        }
    }
}
