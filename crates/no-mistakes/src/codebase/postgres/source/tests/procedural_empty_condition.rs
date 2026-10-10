use super::procedural_occurrences::{block, kinds};

const MISSING: &str = "Procedural condition is missing; add a condition before THEN or LOOP";
const UNKNOWN: &str = "Unsupported procedural occurrence; no execution is inferred";

fn assert_flow(sql: &str, complete: bool, expected: &[&str]) {
    let (_, parsed) = block(sql);
    assert_eq!(parsed.complete, complete, "{sql} {:?}", parsed.diagnostics);
    assert_eq!(kinds(&parsed.occurrences), expected, "{sql}");
    assert!(
        parsed
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.message != MISSING),
        "{sql} {:?}",
        parsed.diagnostics
    );
}

fn assert_missing(sql: &str, expected: &[&str], headers: &[(&str, &str)]) {
    let (owned, parsed) = block(sql);
    assert!(!parsed.complete, "{sql} {:?}", parsed.diagnostics);
    assert_eq!(kinds(&parsed.occurrences), expected, "{sql}");
    assert!(
        parsed
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message == UNKNOWN),
        "{sql} {:?}",
        parsed.diagnostics
    );
    let found: Vec<String> = parsed
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.message == MISSING)
        .map(|diagnostic| {
            let span = diagnostic.span.as_ref().expect("missing condition span");
            owned[span.start.offset..span.end.offset].to_string()
        })
        .collect();
    assert_eq!(found.len(), headers.len(), "{sql} {found:?}");
    for (text, (keyword, stop)) in found.iter().zip(headers) {
        assert!(text.contains(keyword), "{text:?} missing {keyword}");
        assert!(text.contains(stop), "{text:?} missing {stop}");
        assert!(!text.contains("RAISE"), "{text:?} covers RAISE");
    }
}

#[test]
fn empty_if_header_is_not_complete() {
    let expected = ["Unknown[\"ControlFlow\"]"];
    for sql in [
        "DO $$ BEGIN IF THEN RAISE NOTICE 'x'; END IF; END $$;",
        "DO $$ BEGIN IF   THEN RAISE NOTICE 'x'; END IF; END $$;",
        "DO $$ BEGIN IF\nTHEN RAISE NOTICE 'x'; END IF; END $$;",
    ] {
        assert_missing(sql, &expected, &[("IF", "THEN")]);
    }
}

#[test]
fn conditioned_if_stays_complete_control_flow() {
    assert_flow(
        "DO $$ BEGIN IF true THEN RAISE NOTICE 'x'; END IF; END $$;",
        true,
        &["ControlFlow[\"ControlFlow\"]"],
    );
    assert_flow(
        "DO $$ BEGIN IF (1) THEN RAISE NOTICE 'x'; END IF; END $$;",
        true,
        &["ControlFlow[\"ControlFlow\"]"],
    );
}

#[test]
fn empty_elsif_keeps_the_if_incomplete() {
    let expected = ["Unknown[\"ControlFlow\", \"ControlFlow\"]"];
    assert_missing(
        "DO $$ BEGIN IF true THEN RAISE NOTICE 'x'; ELSIF THEN RAISE NOTICE 'y'; END IF; END $$;",
        &expected,
        &[("ELSIF", "THEN")],
    );
    assert_missing(
        "DO $$ BEGIN IF true THEN RAISE NOTICE 'x'; ELSEIF THEN RAISE NOTICE 'y'; END IF; END $$;",
        &expected,
        &[("ELSEIF", "THEN")],
    );
    assert_missing(
        "DO $$ BEGIN IF THEN RAISE NOTICE 'x'; ELSIF THEN RAISE NOTICE 'y'; END IF; END $$;",
        &expected,
        &[("IF", "THEN"), ("ELSIF", "THEN")],
    );
    assert_missing(
        "DO $$ BEGIN IF THEN RAISE NOTICE 'x'; ELSIF false THEN RAISE NOTICE 'y'; END IF; END $$;",
        &expected,
        &[("IF", "THEN")],
    );
    assert_flow(
        "DO $$ BEGIN IF true THEN RAISE NOTICE 'x'; ELSIF false THEN RAISE NOTICE 'y'; END IF; END $$;",
        true,
        &["ControlFlow[\"ControlFlow\", \"ControlFlow\"]"],
    );
}

#[test]
fn else_has_no_condition() {
    assert_flow(
        "DO $$ BEGIN IF true THEN RAISE NOTICE 'x'; ELSE RAISE NOTICE 'y'; END IF; END $$;",
        true,
        &["ControlFlow[\"ControlFlow\", \"ControlFlow\"]"],
    );
    // The empty IF arm is unknown, but ELSE is still walked and is not itself rejected.
    assert_missing(
        "DO $$ BEGIN IF THEN RAISE NOTICE 'x'; ELSE RAISE NOTICE 'y'; END IF; END $$;",
        &["Unknown[\"ControlFlow\", \"ControlFlow\"]"],
        &[("IF", "THEN")],
    );
}

#[test]
fn empty_conditional_loops_are_not_complete() {
    let expected = ["Unknown[\"ControlFlow\"]"];
    assert_missing(
        "DO $$ BEGIN WHILE LOOP RAISE NOTICE 'x'; END LOOP; END $$;",
        &expected,
        &[("WHILE", "LOOP")],
    );
    assert_missing(
        "DO $$ BEGIN FOR LOOP RAISE NOTICE 'x'; END LOOP; END $$;",
        &expected,
        &[("FOR", "LOOP")],
    );
    assert_missing(
        "DO $$ BEGIN FOREACH LOOP RAISE NOTICE 'x'; END LOOP; END $$;",
        &expected,
        &[("FOREACH", "LOOP")],
    );
}

#[test]
fn conditioned_and_bare_loops_stay_complete() {
    let expected = ["ControlFlow[\"ControlFlow\"]"];
    for sql in [
        "DO $$ BEGIN WHILE true LOOP RAISE NOTICE 'x'; END LOOP; END $$;",
        "DO $$ BEGIN FOR r IN SELECT 1 LOOP RAISE NOTICE 'x'; END LOOP; END $$;",
        "DO $$ BEGIN FOREACH item IN ARRAY items LOOP RAISE NOTICE 'x'; END LOOP; END $$;",
        // Bare LOOP is not a conditional form.
        "DO $$ BEGIN LOOP RAISE NOTICE 'x'; END LOOP; END $$;",
    ] {
        assert_flow(sql, true, &expected);
    }
}

#[test]
fn empty_header_still_walks_the_following_statement() {
    // Returning before the body would swallow the next statement into the header.
    assert_missing(
        "DO $$ BEGIN IF THEN RAISE NOTICE 'x'; END IF; RAISE NOTICE 'y'; END $$;",
        &["Unknown[\"ControlFlow\"]", "ControlFlow"],
        &[("IF", "THEN")],
    );
    assert_missing(
        "DO $$ BEGIN WHILE LOOP RAISE NOTICE 'x'; END LOOP; RAISE NOTICE 'y'; END $$;",
        &["Unknown[\"ControlFlow\"]", "ControlFlow"],
        &[("WHILE", "LOOP")],
    );
}

#[test]
fn empty_header_beside_sql_keeps_the_header_diagnostic() {
    // SQL takes the parse path, which replaces walker diagnostics.
    assert_missing(
        "DO $$ BEGIN IF THEN RAISE NOTICE 'x'; END IF; CREATE TABLE a(id int); END $$;",
        &["Unknown[\"ControlFlow\"]", "Utility"],
        &[("IF", "THEN")],
    );
    assert_missing(
        "DO $$ BEGIN WHILE LOOP RAISE NOTICE 'x'; END LOOP; CREATE TABLE a(id int); END $$;",
        &["Unknown[\"ControlFlow\"]", "Utility"],
        &[("WHILE", "LOOP")],
    );
}
