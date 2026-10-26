/// Preserve the raw CLI report for parity and ledger assertions while older
/// policy checks inspect every advisory except the additive hygiene rules.
pub fn without_hygiene(report: &serde_json::Value) -> Vec<&serde_json::Value> {
    report["advisories"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| {
            !matches!(
                row["rule"].as_str(),
                Some("unused-disable-directive" | "invalid-disable-directive")
            )
        })
        .collect()
}
