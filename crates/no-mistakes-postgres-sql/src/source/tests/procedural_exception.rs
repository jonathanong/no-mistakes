use super::procedural_occurrences::{block, kinds};

#[test]
fn exception_section_without_a_when_arm_is_incomplete() {
    // No WHEN arm must not become an empty recognized control-flow occurrence.
    let (_, parsed) = block("DO $$ BEGIN CREATE TABLE t(id integer); EXCEPTION END $$;");
    assert_eq!(kinds(&parsed.occurrences), ["Utility", "Unknown"]);
    assert!(!parsed.complete, "{:?}", parsed.diagnostics);
    assert!(parsed.diagnostics.iter().any(|diagnostic| diagnostic
        .message
        .contains("Unsupported procedural occurrence")));
}

#[test]
fn exception_when_arm_stays_control_flow() {
    let (_, parsed) = block("DO $$ BEGIN EXCEPTION WHEN others THEN NULL; END $$;");
    assert!(
        kinds(&parsed.occurrences)
            .iter()
            .any(|kind| kind.starts_with("ControlFlow")),
        "{:?}",
        kinds(&parsed.occurrences)
    );
}

#[test]
fn exception_when_arm_without_then_stays_unknown() {
    let (_, parsed) = block(
        "DO $$ BEGIN INSERT INTO t(id) VALUES (1); EXCEPTION WHEN unique_violation DELETE FROM t; END $$;",
    );
    assert!(kinds(&parsed.occurrences)
        .iter()
        .any(|kind| kind.contains("Unknown")));
}
