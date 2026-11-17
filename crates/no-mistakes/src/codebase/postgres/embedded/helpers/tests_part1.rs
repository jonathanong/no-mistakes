#[test]
fn sentinel_markers_round_trip_and_ignore_malformed_text() {
    let text = format!("a{}b{}c", sentinel::encode(3), sentinel::encode(12));
    let sites = sentinel::find_all(&text)
        .into_iter()
        .map(|(_, site)| site)
        .collect::<Vec<_>>();
    assert_eq!(sites, [3, 12]);
    assert_eq!(sentinel::prefix(&text), "a");
    assert_eq!(sentinel::prefix("no hole"), "no hole");
    assert!(sentinel::contains(&text));
    assert_eq!(
        restore_placeholders(&text),
        "asql_placeholder_1bsql_placeholder_2c"
    );
    let glued = format!("tmp_{}", sentinel::encode(1));
    assert!(splits_identifier(&glued));
    assert!(!has_identifier_slot_hole(&glued));
    let relation = format!("SELECT id FROM tmp_{}", sentinel::encode(1));
    assert!(has_identifier_slot_hole(&relation));
    assert!(has_identifier_slot_hole(&format!(
        "SELECT {}",
        sentinel::encode(1)
    )));
    assert!(has_identifier_slot_hole(&format!(
        "SELECT id FROM {}",
        sentinel::encode(1)
    )));
    let value = format!("SELECT id FROM items WHERE id = {}", sentinel::encode(1));
    assert!(!has_identifier_slot_hole(&value));
    assert!(has_value_operand_hole(&value));
    assert!(keeps_placeholder(&value));
    assert!(!has_value_operand_hole(&format!(
        "SELECT 1 WHERE {}",
        sentinel::encode(1)
    )));
    assert!(!splits_identifier(&format!("FROM {}", sentinel::encode(1))));
    assert_eq!(restore_placeholders(&glued), "tmp_sql_placeholder_1");
    for malformed in [
        format!("bare {MARK} marker"),
        format!("{MARK}12 never closed"),
        format!("{MARK}{MARK}"),
        format!("{MARK}99999999999999999999{MARK}"),
    ] {
        assert!(!sentinel::contains(&malformed), "{malformed:?}");
    }
}

#[test]
fn unpublish_restores_only_the_recorded_markers() {
    let internal =
        format!("a {INTERNAL_PLACEHOLDER}1 sql_placeholder_9 \n{INTERNAL_PLACEHOLDER}2 z");
    let (published, positions) = publish_placeholders_with_positions(internal.clone());
    // The user-authored spelling has no recorded position, so it stays as written.
    assert_eq!(
        unpublish(&published, &positions).as_deref(),
        Some(internal.as_str())
    );
    assert_eq!(unpublish("plain text", &[]).as_deref(), Some("plain text"));
    assert!(unpublish(&published, &[(1, 2)]).is_none());
    assert!(unpublish("short", &[(1, 9)]).is_none());
    assert!(unpublish("sql_placeholder_", &[(1, 1)]).is_none());
}

#[test]
fn split_and_compose_keep_every_physical_line() {
    let hole = sentinel::encode(0);
    let sql = format!("SELECT 1\nWHERE {hole} ORDER BY z");
    let mut joined = caller(sql, 10);
    // The second SQL line was authored far from the first.
    joined.positions = [(1, 10), (2, 50)]
        .map(
            |(sql_line, source_line)| crate::codebase::postgres::EmbeddedSqlSourcePosition {
                sql_line,
                sql_column: 1,
                source_line,
                source_path: None,
            },
        )
        .to_vec();
    let markers = sentinel::find_all(&joined.sql);
    let ranges = markers
        .iter()
        .map(|(range, _)| range.clone())
        .collect::<Vec<_>>();
    let (pieces, origins) = split(&joined, &ranges);
    assert_eq!(pieces.len(), 2);
    assert_eq!(pieces[0].sql, "SELECT 1\nWHERE ");
    assert_eq!(pieces[1].sql, " ORDER BY z");
    // The marker sits on the caller's second line.
    assert_eq!((origins[0].line, origins[0].path.clone()), (50, None));
    let helper = plain(
        "EXISTS (SELECT 1\nFROM t)".to_string(),
        Marker {
            line: 40,
            path: Some(PathBuf::from("/helper.ts")),
            origin: 7,
        },
    );
    let [first, second]: [RecoveredSql; 2] = pieces.try_into().ok().expect("two pieces");
    let whole = compose([first, helper, second]).expect("composed");
    assert_eq!(
        whole.sql,
        "SELECT 1\nWHERE EXISTS (SELECT 1\nFROM t) ORDER BY z"
    );
    let line_of = |needle: &str| {
        let offset = whole.sql.find(needle).expect(needle);
        origin_at(&whole.positions, whole.line, coordinate(&whole.sql, offset))
    };
    let helper_path = Some(PathBuf::from("/helper.ts"));
    assert_eq!(line_of("SELECT 1\nWHERE"), (10, None));
    assert_eq!(line_of("WHERE"), (50, None));
    assert_eq!(line_of("EXISTS"), (40, helper_path.clone()));
    assert_eq!(line_of("FROM t"), (41, helper_path));
    // Text after the helper returns to the caller's own lines and module.
    assert_eq!(line_of("ORDER"), (50, None));
}

#[test]
fn empty_pieces_and_empty_compositions_contribute_nothing() {
    assert!(compose([]).is_none());
    assert!(compose([caller(String::new(), 1)]).is_none());
    let only = compose([caller(String::new(), 1), caller("x".into(), 5)]).expect("one part");
    assert_eq!((only.sql.as_str(), only.line), ("x", 5));
    assert_eq!(only.positions.len(), 1);
}

#[test]
fn site_arguments_decide_which_summary_shapes_may_fill_a_hole() {
    let owner = Path::new("/helpers.ts");
    let site = |args: Vec<SiteArg>| ImportedSite {
        local: "helper".into(),
        member: None,
        kind: crate::codebase::postgres::embedded::helpers::SiteKind::Sql,
        args,
        value_used: false,
        line: 1,
        constant: None,
    };
    let fixed = HelperSummary::Static(plain(
        "TRUE".into(),
        Marker {
            line: 3,
            path: None,
            origin: 0,
        },
    ));
    let literal = site(vec![SiteArg::Literal("a".into()), SiteArg::Scalar]);
    assert_eq!(
        fixed.fill(&literal, owner, owner).expect("static").sql,
        "TRUE"
    );
    // A builder argument means the helper's result includes the builder's text.
    assert!(fixed
        .fill(&site(vec![SiteArg::Builder]), owner, owner)
        .is_none());
    let filled = fixed
        .fill(&site(Vec::new()), owner, owner)
        .expect("no arguments");
    assert!(filled
        .positions
        .iter()
        .all(|position| position.source_path.as_deref() == Some(owner)));
}
