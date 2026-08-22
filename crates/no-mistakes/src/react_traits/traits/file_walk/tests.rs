use super::collect_file_trait_hits;
use crate::ast;
use crate::react_traits::analyze::components::extract_components;
use crate::react_traits::analyze::import_table::build_import_table;
use std::collections::HashSet;
use std::path::Path;

fn hits_for(source: &str) -> super::FileTraitHits {
    let path = Path::new("test.tsx");
    ast::with_program(path, source, |program, _| {
        let defs = extract_components(program);
        let dynamic_names = vec![HashSet::new(); defs.len()];
        let import_table = build_import_table(path, program);
        collect_file_trait_hits(program, &defs, &dynamic_names, &import_table, path)
    })
    .expect("source should parse")
}

#[test]
fn file_walk_records_this_state_inside_class_component() {
    let hits =
        hits_for("export default class App extends Component { render() { return this.state; } }");
    assert_eq!(hits.has_state, vec![true]);
}

#[test]
fn file_walk_records_set_state_member_inside_class_component() {
    let hits =
        hits_for("export default class App extends Component { bump() { this.setState({}); } }");
    assert_eq!(hits.has_state, vec![true]);
}

#[test]
fn file_walk_ignores_this_state_outside_component_spans() {
    let hits = hits_for(
        "class Helper { value() { return this.state; } }\nexport default function App() { return null; }",
    );
    assert_eq!(hits.has_state, vec![false]);
}
