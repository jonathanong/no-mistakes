use super::procedural_occurrences::{block, kinds};

fn assert_flow(sql: &str, complete: bool, expected: &[&str]) {
    let (_, parsed) = block(sql);
    assert_eq!(parsed.complete, complete, "{sql} {:?}", parsed.diagnostics);
    assert_eq!(kinds(&parsed.occurrences), expected, "{sql}");
}

#[test]
fn empty_if_header_is_not_complete() {
    let expected = ["Unknown[\"ControlFlow\"]"];
    for sql in [
        "DO $$ BEGIN IF THEN RAISE NOTICE 'x'; END IF; END $$;",
        "DO $$ BEGIN IF   THEN RAISE NOTICE 'x'; END IF; END $$;",
        "DO $$ BEGIN IF\nTHEN RAISE NOTICE 'x'; END IF; END $$;",
    ] {
        assert_flow(sql, false, &expected);
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
    for sql in [
        "DO $$ BEGIN IF true THEN RAISE NOTICE 'x'; ELSIF THEN RAISE NOTICE 'y'; END IF; END $$;",
        "DO $$ BEGIN IF true THEN RAISE NOTICE 'x'; ELSEIF THEN RAISE NOTICE 'y'; END IF; END $$;",
    ] {
        assert_flow(sql, false, &expected);
    }
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
    assert_flow(
        "DO $$ BEGIN IF THEN RAISE NOTICE 'x'; ELSE RAISE NOTICE 'y'; END IF; END $$;",
        false,
        &["Unknown[\"ControlFlow\", \"ControlFlow\"]"],
    );
}

#[test]
fn empty_conditional_loops_are_not_complete() {
    let expected = ["Unknown[\"ControlFlow\"]"];
    for sql in [
        "DO $$ BEGIN WHILE LOOP RAISE NOTICE 'x'; END LOOP; END $$;",
        "DO $$ BEGIN FOR LOOP RAISE NOTICE 'x'; END LOOP; END $$;",
        "DO $$ BEGIN FOREACH LOOP RAISE NOTICE 'x'; END LOOP; END $$;",
    ] {
        assert_flow(sql, false, &expected);
    }
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
    assert_flow(
        "DO $$ BEGIN IF THEN RAISE NOTICE 'x'; END IF; RAISE NOTICE 'y'; END $$;",
        false,
        &["Unknown[\"ControlFlow\"]", "ControlFlow"],
    );
    assert_flow(
        "DO $$ BEGIN WHILE LOOP RAISE NOTICE 'x'; END LOOP; RAISE NOTICE 'y'; END $$;",
        false,
        &["Unknown[\"ControlFlow\"]", "ControlFlow"],
    );
}
