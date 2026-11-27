use super::super::PostgresSqlStatementKind;
use super::procedural_occurrences::{block, kinds};

#[test]
fn literal_execute_select_is_utility_and_dynamic_select_stays_closed() {
    let (_, parsed) = block("DO $$ BEGIN EXECUTE 'SELECT 1'; END $$;");
    assert_eq!(kinds(&parsed.occurrences), ["Utility"]);
    let rendered = format!("{:?}", parsed.occurrences);
    assert!(!rendered.contains("Unknown"));
    assert!(!rendered.contains("DynamicExecute"));
    assert!(matches!(
        parsed.statements[0].facts,
        PostgresSqlStatementKind::LiteralExecute { .. }
    ));

    let (_, parsed) = block("DO $$ BEGIN EXECUTE format('SELECT 1'); END $$;");
    assert_eq!(kinds(&parsed.occurrences), ["DynamicExecute"]);
    assert!(!parsed.complete);

    let (_, parsed) = block("DO $$ BEGIN EXECUTE 'INSERT INTO t(id) VALUES (1)'; END $$;");
    assert_eq!(kinds(&parsed.occurrences), ["Dml"]);

    let (_, parsed) = block("DO $$ BEGIN EXECUTE 'CREATE TABLE a(id int)'; END $$;");
    assert_eq!(kinds(&parsed.occurrences), ["Utility"]);

    // SELECT must not outrank a sibling DML or utility command inside one literal.
    let (_, parsed) =
        block("DO $$ BEGIN EXECUTE 'SELECT 1; INSERT INTO t(id) VALUES (1)'; END $$;");
    assert_eq!(kinds(&parsed.occurrences), ["Dml"]);
    let (_, parsed) = block("DO $$ BEGIN EXECUTE 'SELECT 1; CREATE TABLE a(id int)'; END $$;");
    assert_eq!(kinds(&parsed.occurrences), ["Utility"]);

    // A non-SELECT literal that is not known utility or DML stays fail-closed.
    let (_, parsed) = block("DO $$ BEGIN EXECUTE 'VALUES (1)'; END $$;");
    assert_eq!(kinds(&parsed.occurrences), ["Unknown"]);

    // Bare SELECT stays unknown. Classifying it as utility would change LOOP completeness.
    let (_, parsed) = block("DO $$ BEGIN SELECT 1; END $$;");
    assert_eq!(kinds(&parsed.occurrences), ["Unknown"]);
    let (_, parsed) = block("DO $$ BEGIN LOOP SELECT 1; END LOOP; END $$;");
    assert_eq!(kinds(&parsed.occurrences), ["ControlFlow[\"Unknown\"]"]);
    assert!(!parsed.complete);

    // A procedural child that starts with SELECT is not a SQL statement.
    let (_, parsed) = block("DO $$ BEGIN EXECUTE 'IF true THEN SELECT 1; END IF'; END $$;");
    assert_eq!(kinds(&parsed.occurrences), ["Unknown"]);
    assert!(!parsed.complete, "{:?}", parsed.diagnostics);
    let (_, parsed) =
        block("DO $$ BEGIN LOOP EXECUTE 'IF true THEN SELECT 1; END IF'; END LOOP; END $$;");
    assert_eq!(kinds(&parsed.occurrences), ["ControlFlow[\"Unknown\"]"]);
    assert!(!parsed.complete, "{:?}", parsed.diagnostics);

    // The first token is not proof of a statement. LOOP would otherwise complete.
    let (_, parsed) = block("DO $$ BEGIN EXECUTE 'SELECT * FROM'; END $$;");
    assert_eq!(kinds(&parsed.occurrences), ["Unknown"]);
    assert!(!parsed.complete, "{:?}", parsed.diagnostics);
    let (_, parsed) = block("DO $$ BEGIN LOOP EXECUTE 'SELECT * FROM'; END LOOP; END $$;");
    assert_eq!(kinds(&parsed.occurrences), ["ControlFlow[\"Unknown\"]"]);
    assert!(!parsed.complete, "{:?}", parsed.diagnostics);

    let (_, parsed) = block("DO $$ BEGIN LOOP EXECUTE 'SELECT 1'; END LOOP; END $$;");
    assert_eq!(kinds(&parsed.occurrences), ["ControlFlow[\"Utility\"]"]);

    // An interior semicolon is not a terminator. Stripping it would parse as SELECT (1).
    let (_, parsed) = block("DO $$ BEGIN EXECUTE 'SELECT (1;);'; END $$;");
    assert_eq!(kinds(&parsed.occurrences), ["Unknown"]);
    assert!(!parsed.complete, "{:?}", parsed.diagnostics);
    let (_, parsed) = block("DO $$ BEGIN LOOP EXECUTE 'SELECT (1;);'; END LOOP; END $$;");
    assert_eq!(kinds(&parsed.occurrences), ["ControlFlow[\"Unknown\"]"]);
    assert!(!parsed.complete, "{:?}", parsed.diagnostics);

    // skip_label drops <<mark>> before the span, so the slice is only SELECT 1.
    let (_, parsed) = block("DO $$ BEGIN EXECUTE '<<mark>> SELECT 1'; END $$;");
    assert_eq!(kinds(&parsed.occurrences), ["Unknown"]);
    assert!(!parsed.complete, "{:?}", parsed.diagnostics);
    let (_, parsed) = block("DO $$ BEGIN LOOP EXECUTE '<<mark>> SELECT 1'; END LOOP; END $$;");
    assert_eq!(kinds(&parsed.occurrences), ["ControlFlow[\"Unknown\"]"]);
    assert!(!parsed.complete, "{:?}", parsed.diagnostics);

    // A procedural sibling must not fold into recognized control flow.
    let (_, parsed) = block("DO $$ BEGIN EXECUTE 'SELECT 1; RAISE NOTICE ''x'';'; END $$;");
    assert_eq!(kinds(&parsed.occurrences), ["Unknown"]);
    assert!(!parsed.complete, "{:?}", parsed.diagnostics);
    let (_, parsed) =
        block("DO $$ BEGIN LOOP EXECUTE 'SELECT 1; RAISE NOTICE ''x'';'; END LOOP; END $$;");
    assert_eq!(kinds(&parsed.occurrences), ["ControlFlow[\"Unknown\"]"]);
    assert!(!parsed.complete, "{:?}", parsed.diagnostics);
}
