use super::FileIdMap;
use crate::codebase::ts_source::FileInventory;
use std::path::PathBuf;
use std::sync::Arc;

fn path(name: &str) -> PathBuf {
    PathBuf::from(format!("/fixture/{name}"))
}

#[test]
fn get_mut_updates_an_occupied_inventory_slot() {
    let source = path("source.ts");
    let mut map = FileIdMap::from_entries([(source.clone(), 1u32)]);

    *map.get_mut(&source).expect("inventoried path is present") = 2;

    assert_eq!(map.get(&source), Some(&2));
}

#[test]
fn mut_iter_yields_occupied_inventory_slots() {
    let source = path("source.ts");
    let mut map = FileIdMap::from_entries([(source.clone(), 1u32)]);

    let mut seen = 0;
    for (got, value) in &mut map {
        assert_eq!(got, &source);
        *value = 3;
        seen += 1;
    }

    assert_eq!(seen, 1);
    assert_eq!(map.get(&source), Some(&3));
}

#[test]
fn mut_iter_skips_empty_slots_and_visits_overflow() {
    let inventoried = path("inventoried.ts");
    let overflow = path("overflow.ts");
    let inventory = Arc::new(FileInventory::from_lookup_paths([inventoried.clone()]));
    let mut map = FileIdMap::with_inventory(inventory);
    map.insert(overflow.clone(), 7u32);

    let entries: Vec<_> = (&mut map)
        .into_iter()
        .map(|(got, value)| (got.clone(), *value))
        .collect();

    assert_eq!(entries, vec![(overflow.clone(), 7)]);
    assert!(map.get_mut(&inventoried).is_none());
    assert_eq!(map.get_mut(&overflow), Some(&mut 7));
}
