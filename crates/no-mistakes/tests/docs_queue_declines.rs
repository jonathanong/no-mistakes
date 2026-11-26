use std::fs;
use std::path::PathBuf;

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(rel);
    fs::read_to_string(&path).unwrap_or_else(|err| panic!("{}: {err}", path.display()))
}

fn normalized(rel: &str) -> String {
    read(rel).split_whitespace().collect::<Vec<_>>().join(" ")
}

#[test]
fn declined_queue_languages_match_the_status_table() {
    let parity = read("docs/feature-parity.md");
    let phrase = "declined: no documented static producer or worker";
    for row in [
        "| Swift",
        "| .NET / C#",
        "| Java, Spring",
        "| Kotlin, Spring",
        "| Elixir, Phoenix",
        "| Dart / Flutter",
    ] {
        let line = parity
            .lines()
            .find(|line| line.starts_with(row))
            .unwrap_or_else(|| panic!("missing status row {row}"));
        assert!(line.contains(phrase), "{row}: {line}");
    }
    let rust = parity
        .lines()
        .find(|line| line.starts_with("| Rust"))
        .expect("rust status row");
    assert!(rust.contains("declined: no Rust job facts"), "{rust}");
    assert!(!parity.contains("configured enqueue/worker globs"));
    assert!(parity.contains("Kafka literals outside TypeScript and Python are out of scope."));
    let shared = "Rust facts record no jobs, so configured globs publish no Rust job edges.";
    for path in [
        "docs/feature-parity.md",
        "docs/cli/queues.md",
        "docs/node-api.md",
        "docs/configuration/queues.md",
        "plugins/no-mistakes/skills/no-mistakes/references/limits-and-fallbacks.md",
    ] {
        assert!(normalized(path).contains(shared), "{path}");
    }
}
