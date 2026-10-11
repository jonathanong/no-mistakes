use super::*;

#[test]
fn nullish_short_circuit_checks_the_fallback_and_accepts_all_safe_versions() {
    let bad = check("postgres-no-offset", "nullish-and-fallback-bad");
    let findings = bad["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 1, "{bad}");
    assert_eq!(findings[0]["target"], "offset", "{bad}");
    assert_eq!(findings[0]["line"], 6, "{bad}");
    let good = check("postgres-no-offset", "nullish-and-fallback-good");
    assert!(good["rules"].as_array().unwrap().is_empty(), "{good}");
}
