use super::*;

#[derive(serde::Deserialize)]
struct Selection {
    entries: Vec<String>,
    at_budget_groups: Vec<u32>,
    extra_group: u32,
}

#[test]
fn caps_selected_paths_before_building_objects() {
    let fixture =
        crate::codebase::postgres::embedded::tests::read_fixture("frozen-selection-budget.json");
    let fixture: Selection = serde_json::from_str(&fixture).expect("selection budget fixture");
    let entries: Vec<_> = fixture
        .entries
        .into_iter()
        .map(|key| FrozenEntry {
            key,
            fields: Vec::new(),
        })
        .collect();
    let mut keys = Vec::new();
    for group in fixture.at_budget_groups {
        append_key_arms(&mut keys, &entries, group);
    }
    let at_budget = choose(keys.clone(), &entries).expect("selection at budget");
    assert_eq!(at_budget.len(), super::MAX_EMBEDDED_SQL_VARIANTS);

    append_key_arms(&mut keys, &entries, fixture.extra_group);
    assert!(choose(keys, &entries).is_none());
}

fn append_key_arms(keys: &mut Vec<Arm>, entries: &[FrozenEntry], group: u32) {
    for entry in entries {
        keys.push(Arm {
            node: Node::Text(entry.key.clone()),
            choices: vec![(1 << 62, group), (1 << 63, keys.len() as u32)],
            root: None,
            provenance: None,
            type_annotation: false,
        });
    }
}
