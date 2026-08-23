use super::char_scan::{
    embedded_expressions, has_further_access, is_access_boundary, previous_non_whitespace,
    quoted_end, static_access,
};

fn chars(value: &str) -> Vec<char> {
    value.chars().collect()
}

#[test]
fn static_access_accepts_whitespace_around_dot_and_bracket_forms() {
    let dotted = chars(" .  job_id");
    assert_eq!(
        static_access(&dotted, 0).map(|(name, _)| name),
        Some("job_id".to_string())
    );
    assert!(static_access(&chars(" ."), 0).is_none());

    let bracket = chars(" [  'deploy-job'  ]");
    assert_eq!(
        static_access(&bracket, 0).map(|(name, _)| name),
        Some("deploy-job".to_string())
    );
    assert!(static_access(&chars(" [ 42 ]"), 0).is_none());
    assert!(static_access(&chars(" ['open"), 0).is_none());
    assert!(static_access(&chars(" ['x'"), 0).is_none());
}

#[test]
fn access_boundaries_quoted_spans_and_embedded_expressions_cover_scan_edges() {
    assert!(is_access_boundary(None));
    assert!(is_access_boundary(Some(' ')));
    assert!(!is_access_boundary(Some('a')));
    assert!(!is_access_boundary(Some('.')));

    let padded = chars("  x");
    assert_eq!(previous_non_whitespace(&padded, 1), None);
    assert_eq!(previous_non_whitespace(&padded, 2), Some('x'));

    let quoted = chars("'it''s'");
    assert_eq!(quoted_end(&quoted, 0, '\''), quoted.len());
    assert_eq!(quoted_end(&chars("'open"), 0, '\''), 5);

    let expressions =
        embedded_expressions(&chars("prefix ${{ needs.a }} ${{ '}}' }} ${{ unterminated"));
    assert_eq!(expressions.len(), 2);
    assert!(has_further_access(&chars(" .outputs"), 0));
    assert!(!has_further_access(&chars("  done"), 0));
}
