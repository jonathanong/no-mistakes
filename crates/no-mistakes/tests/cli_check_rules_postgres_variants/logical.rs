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

#[test]
fn unproved_falsy_nullishness_uses_the_dynamic_policy_and_visits_fallback_effects() {
    let failed = check("postgres-no-offset", "opaque-and-nullish");
    let findings = failed["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 1, "{failed}");
    assert_eq!(findings[0]["target"], "unanalyzable", "{failed}");
    assert_eq!(findings[0]["line"], 2, "{failed}");
    let ignored = check("postgres-no-offset", "opaque-and-nullish-ignore");
    assert!(ignored["rules"].as_array().unwrap().is_empty(), "{ignored}");
    let mutation = check("postgres-no-offset", "opaque-and-nullish-mutation");
    let findings = mutation["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 1, "{mutation}");
    assert_eq!(findings[0]["target"], "offset", "{mutation}");
    assert_eq!(findings[0]["line"], 4, "{mutation}");
}
