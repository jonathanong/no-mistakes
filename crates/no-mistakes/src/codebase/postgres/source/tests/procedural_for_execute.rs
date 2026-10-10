use super::procedural_occurrences::{block, kinds};

#[test]
fn literal_for_header_execute_exposes_static_dml() {
    let (_, parsed) = block(
        "DO $$ BEGIN FOR r IN EXECUTE 'DELETE FROM t RETURNING id' LOOP NULL; END LOOP; END $$;",
    );
    assert_eq!(
        kinds(&parsed.occurrences),
        ["ControlFlow[\"Dml\", \"Unknown\"]"]
    );
    assert!(!format!("{:?}", parsed.occurrences).contains("DynamicExecute"));

    for sql in [
        "INSERT INTO t VALUES (1)",
        "UPDATE t SET id = 1",
        "MERGE INTO t USING s ON true WHEN MATCHED THEN DO NOTHING",
    ] {
        let (_, parsed) = block(&format!(
            "DO $$ BEGIN FOR r IN EXECUTE '{sql}' LOOP NULL; END LOOP; END $$;"
        ));
        assert_eq!(
            kinds(&parsed.occurrences),
            ["ControlFlow[\"Dml\", \"Unknown\"]"],
            "{sql}"
        );
        assert!(!format!("{:?}", parsed.occurrences).contains("DynamicExecute"));
    }
}

#[test]
fn dynamic_for_header_execute_stays_incomplete() {
    let (_, parsed) = block(
        "DO $$ BEGIN FOR r IN EXECUTE format('DELETE FROM t RETURNING id') LOOP NULL; END LOOP; END $$;",
    );
    assert_eq!(
        kinds(&parsed.occurrences),
        ["ControlFlow[\"DynamicExecute\", \"Unknown\"]"]
    );
    assert!(!parsed.complete);
}

#[test]
fn literal_for_header_execute_create_table_is_utility() {
    let (_, parsed) =
        block("DO $$ BEGIN FOR r IN EXECUTE 'CREATE TABLE a(id int)' LOOP NULL; END LOOP; END $$;");
    assert_eq!(
        kinds(&parsed.occurrences),
        ["ControlFlow[\"Utility\", \"Unknown\"]"]
    );
    assert!(!format!("{:?}", parsed.occurrences).contains("DynamicExecute"));
}

#[test]
fn for_header_execute_stops_before_using_into_and_the_loop_keyword() {
    // Those words stay outside the operand, or a literal command would look dynamic.
    for sql in [
        "DO $$ BEGIN FOR r IN EXECUTE 'DELETE FROM t RETURNING id' USING extra LOOP NULL; END LOOP; END $$;",
        "DO $$ BEGIN FOR r IN EXECUTE 'DELETE FROM t RETURNING id' INTO target LOOP NULL; END LOOP; END $$;",
        "DO $$ BEGIN FOR r IN EXECUTE 'DELETE FROM t RETURNING id'; LOOP NULL; END LOOP; END $$;",
        "DO $$ BEGIN FOR r IN EXECUTE ('DELETE FROM t RETURNING id') LOOP NULL; END LOOP; END $$;",
        "DO $$ BEGIN FOR r IN EXECUTE 'DELETE FROM t ' || 'RETURNING id' LOOP NULL; END LOOP; END $$;",
    ] {
        let (_, parsed) = block(sql);
        assert_eq!(
            kinds(&parsed.occurrences),
            ["ControlFlow[\"Dml\", \"Unknown\"]"],
            "{sql}"
        );
        assert!(!format!("{:?}", parsed.occurrences).contains("DynamicExecute"));
    }
}

#[test]
fn literal_for_header_execute_select_is_utility() {
    // The header uses the same literal SELECT promotion as statement position.
    let (_, parsed) =
        block("DO $$ BEGIN FOR r IN EXECUTE 'SELECT * FROM t' LOOP NULL; END LOOP; END $$;");
    assert_eq!(
        kinds(&parsed.occurrences),
        ["ControlFlow[\"Utility\", \"Unknown\"]"]
    );

    let (_, parsed) =
        block("DO $$ BEGIN FOR r IN EXECUTE 'SELECT * FROM' LOOP NULL; END LOOP; END $$;");
    assert_eq!(
        kinds(&parsed.occurrences),
        ["ControlFlow[\"Unknown\", \"Unknown\"]"]
    );
    assert!(!format!("{:?}", parsed.occurrences).contains("DynamicExecute"));
}

#[test]
fn qualified_execute_in_a_for_query_is_not_a_command() {
    // `public.execute(...)` is a function call. It stays the fail-closed header
    // mark instead of a static DML command parsed from the argument.
    let (_, parsed) = block(
        "DO $$ BEGIN FOR r IN SELECT * FROM public.execute('DELETE FROM t') LOOP NULL; END LOOP; END $$;",
    );
    let rendered = format!("{:?}", parsed.occurrences);
    assert!(rendered.contains("DynamicExecute"), "{rendered}");
    assert!(!rendered.contains("Dml"), "{rendered}");
}

#[test]
fn statement_position_execute_is_unchanged() {
    let (_, parsed) = block("DO $$ BEGIN EXECUTE 'DELETE FROM t RETURNING id'; END $$;");
    assert_eq!(kinds(&parsed.occurrences), ["Dml"]);
    assert!(!format!("{:?}", parsed.occurrences).contains("DynamicExecute"));

    let (_, parsed) = block("DO $$ BEGIN EXECUTE 'CREATE TABLE a(id int)'; END $$;");
    assert_eq!(kinds(&parsed.occurrences), ["Utility"]);

    let (_, parsed) = block("DO $$ BEGIN EXECUTE format('DELETE FROM t RETURNING id'); END $$;");
    assert_eq!(kinds(&parsed.occurrences), ["DynamicExecute"]);
    assert!(!parsed.complete);
}
