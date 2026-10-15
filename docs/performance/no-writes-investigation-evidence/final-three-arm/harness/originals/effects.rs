//! Configured public consuming API; fresh request state on every iteration.
use no_mistakes::{
    diagnostics::{with_observer, InvocationObserver},
    effects_query,
};
use std::{hint::black_box, path::PathBuf};

fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(args.len(), 6, "MODE ROOT KIND ENTRY ITERATIONS");
    let root = PathBuf::from(&args[2]).canonicalize().unwrap();
    let entry = PathBuf::from(&args[4]);
    let iterations: usize = args[5].parse().unwrap();
    assert!(iterations > 0);
    assert!(matches!(args[1].as_str(), "off" | "observer" | "verify"));
    let mut expected = None;
    for _ in 0..iterations {
        let observer = (args[1] == "observer").then(|| InvocationObserver::new(true));
        let report = with_observer(observer, || {
            effects_query::run(&root, None, None, &args[3], &entry, &[], None).unwrap()
        });
        let count = report.call_sites.len();
        if let Some(prior) = expected {
            assert_eq!(prior, count);
        }
        expected = Some(count);
        if args[1] == "verify" {
            // Full reports are untimed semantic evidence, never timing output.
            assert_eq!(iterations, 1);
            println!("{report:?}");
        }
        black_box(report);
    }
    if args[1] != "verify" {
        println!("{}", expected.unwrap());
    }
}
