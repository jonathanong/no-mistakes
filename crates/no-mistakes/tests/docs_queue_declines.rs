use std::fs;
use std::path::PathBuf;

fn read(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(rel);
    fs::read_to_string(&path).unwrap_or_else(|err| panic!("{}: {err}", path.display()))
}

#[test]
fn declined_queue_languages_match_the_status_table() {
    let parity = read("docs/feature-parity.md");
    for row in [
        "| Swift",
        "| .NET / C#",
        "| Rust",
        "| Java, Spring",
        "| Kotlin, Spring",
        "| Elixir, Phoenix",
        "| Dart / Flutter",
    ] {
        let queue_status = parity
            .lines()
            .find(|line| line.starts_with(row))
            .and_then(|line| line.split('|').nth(5))
            .unwrap_or_else(|| panic!("missing queue status for {row}"));
        assert!(
            queue_status.trim_start().starts_with("declined:"),
            "{row}: {queue_status}"
        );
    }
}
