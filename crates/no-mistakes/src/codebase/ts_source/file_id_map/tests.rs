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

#[test]
fn into_entries_yields_only_occupied_slots_and_overflow() {
    let inventoried = path("inventoried.ts");
    let occupied = path("occupied.ts");
    let overflow = path("overflow.ts");
    let inventory = Arc::new(FileInventory::from_lookup_paths([
        inventoried,
        occupied.clone(),
    ]));
    let mut map = FileIdMap::with_inventory(inventory);
    map.insert(occupied.clone(), 1u32);
    map.insert(overflow.clone(), 2u32);

    let mut entries: Vec<_> = map.into_entries().collect();
    entries.sort_by(|left, right| left.0.cmp(&right.0));
    assert_eq!(entries, vec![(occupied, 1), (overflow, 2)]);
}

#[test]
fn borrowed_iter_skips_empty_slots_and_visits_overflow() {
    let inventoried = path("inventoried.ts");
    let occupied = path("occupied.ts");
    let overflow = path("overflow.ts");
    let inventory = Arc::new(FileInventory::from_lookup_paths([
        inventoried,
        occupied.clone(),
    ]));
    let mut map = FileIdMap::with_inventory(inventory);
    map.insert(occupied.clone(), 1u32);
    map.insert(overflow.clone(), 2u32);

    let mut entries: Vec<_> = (&map)
        .into_iter()
        .map(|(got, value)| (got.clone(), *value))
        .collect();
    entries.sort_by(|left, right| left.0.cmp(&right.0));
    assert_eq!(entries, vec![(occupied, 1), (overflow, 2)]);
}

#[test]
fn borrowed_file_id_map_into_iter_stays_lazy() {
    let source = include_str!("../file_id_map_iter.rs");
    assert!(
        source.contains("type IntoIter = FileIdMapIter<'a, T>;"),
        "borrowed FileIdMap iteration must stay a lazy occupied-slot iterator"
    );
    assert!(
        !source
            .split("impl<'a, T> IntoIterator for &'a FileIdMap<T>")
            .nth(1)
            .expect("borrowed IntoIterator impl")
            .split("impl<T> std::iter::FromIterator")
            .next()
            .expect("borrowed impl body")
            .contains("collect::<Vec<_>"),
        "borrowed FileIdMap into_iter must not materialize a Vec"
    );
}

#[test]
fn consuming_file_id_map_into_iter_reuses_into_entries() {
    let source = include_str!("../file_id_map_iter.rs");
    let body = source
        .split("impl<T> IntoIterator for FileIdMap<T>")
        .nth(1)
        .expect("owned IntoIterator impl")
        .split("impl<'a, T> IntoIterator for &'a FileIdMap<T>")
        .next()
        .expect("owned impl body");
    assert!(
        body.contains("self.into_entries()"),
        "owned FileIdMap into_iter must return the into_entries Vec iterator"
    );
    assert!(
        !body.contains("collect::<Vec<_>"),
        "owned FileIdMap into_iter must not collect into_entries a second time"
    );
}
