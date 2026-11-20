use super::take_pattern_line;
use std::collections::{BTreeMap, BTreeSet};

#[test]
fn absent_pattern_uses_line_one() {
    let mut seen = BTreeMap::new();
    let mut used = BTreeSet::new();
    let line = take_pattern_line("id: widgets\n", "src/missing.ts", &mut seen, &mut used);
    assert_eq!(line, 1);
    assert!(used.contains(&1));
    let empty = take_pattern_line("ignores: []\n", "", &mut seen, &mut used);
    assert_eq!(empty, 1);
}
