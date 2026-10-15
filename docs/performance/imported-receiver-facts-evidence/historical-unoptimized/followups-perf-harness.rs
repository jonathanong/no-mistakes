use no_mistakes::codebase::ts_source::facts::{collect_ts_facts, TsFactPlan};
use std::{hint::black_box, path::PathBuf};
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let file = PathBuf::from(&args[1]);
    let demand = args[2] == "values";
    let iterations: usize = args[3].parse().unwrap();
    let mut signature = 0;
    for _ in 0..iterations {
        let facts = collect_ts_facts(
            std::slice::from_ref(&file),
            TsFactPlan {
                imports: true,
                function_calls: true,
                value_flow: demand,
                ..Default::default()
            },
        );
        let fact = facts.get(&file).unwrap();
        assert!(fact.operational_error.is_none() && fact.parse_error.is_none());
        signature = fact.function_calls.len() + fact.imports.len() + fact.value_flow.bindings.len();
        black_box(facts);
    }
    println!("{signature}");
}
