//! Dispatch only: every analysis loop remains in its preserved source module.
mod effects;
mod facts;
mod graph_semantics;
mod graph_timing;

fn main() {
    let args: Vec<_> = std::env::args().collect();
    match args.as_slice() {
        [_, file, demand, iterations]
            if absolute(file) && matches!(demand.as_str(), "values" | "imports") =>
        {
            positive(iterations);
            facts::main();
        }
        [_, file, demand, iterations, dump]
            if absolute(file)
                && matches!(demand.as_str(), "values" | "imports")
                && dump == "dump" =>
        {
            assert_eq!(
                positive(iterations),
                1,
                "fact verification requires one iteration"
            );
            facts::main();
        }
        [_, mode, root, iterations]
            if absolute(root)
                && matches!(mode.as_str(), "graph" | "graph-no-calls" | "graph-observer") =>
        {
            positive(iterations);
            graph_timing::main();
        }
        [_, root, iterations, observation, calls]
            if absolute(root)
                && matches!(observation.as_str(), "off" | "work")
                && matches!(calls.as_str(), "calls" | "no-calls") =>
        {
            assert_eq!(
                positive(iterations),
                1,
                "graph verification requires one iteration"
            );
            graph_semantics::main();
        }
        [_, mode, root, kind, _, iterations]
            if absolute(root)
                && matches!(mode.as_str(), "off" | "observer" | "verify")
                && kind == "valkey" =>
        {
            let count = positive(iterations);
            if mode == "verify" {
                assert_eq!(count, 1, "effects verification requires one iteration");
            }
            effects::main();
        }
        _ => panic!("unsupported probe argument shape"),
    }
}

fn positive(value: &str) -> usize {
    let count: usize = value.parse().expect("integer iteration count");
    assert!(count > 0, "positive iteration count");
    count
}

fn absolute(value: &str) -> bool {
    std::path::Path::new(value).is_absolute()
}
