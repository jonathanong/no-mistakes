use super::{bound_comments, BoundComment, BoundCommentError};

fn comments(source: &str) -> Vec<BoundComment> {
    bound_comments(source, "postgres-bounded-statements")
}

#[test]
fn parses_sql_typescript_and_hash_comments() {
    let source = "\
-- no-mistakes-bound postgres-bounded-statements rows<=100: caller passes one page of ids
// no-mistakes-bound postgres-bounded-statements rows<=25:page of ids
# no-mistakes-bound postgres-bounded-statements rows<=0: empty page
";
    assert_eq!(
        comments(source),
        vec![
            BoundComment::Valid {
                line: 1,
                rows: 100,
                justification: "caller passes one page of ids".to_string(),
            },
            BoundComment::Valid {
                line: 2,
                rows: 25,
                justification: "page of ids".to_string(),
            },
            BoundComment::Valid {
                line: 3,
                rows: 0,
                justification: "empty page".to_string(),
            },
        ]
    );
}

#[test]
fn rejects_a_missing_bound_or_justification() {
    let source = "\
-- no-mistakes-bound postgres-bounded-statements: caller passes one page of ids
-- no-mistakes-bound postgres-bounded-statements rows<=
-- no-mistakes-bound postgres-bounded-statements rows<=100
-- no-mistakes-bound postgres-bounded-statements rows<=100:
-- no-mistakes-bound postgres-bounded-statements rows<=100 extra: reason
-- no-mistakes-bound postgres-bounded-statements rows<=18446744073709551616: too big
";
    assert_eq!(
        comments(source),
        vec![
            BoundComment::Invalid {
                line: 1,
                error: BoundCommentError::MissingBound,
            },
            BoundComment::Invalid {
                line: 2,
                error: BoundCommentError::MissingBound,
            },
            BoundComment::Invalid {
                line: 3,
                error: BoundCommentError::MissingJustification,
            },
            BoundComment::Invalid {
                line: 4,
                error: BoundCommentError::MissingJustification,
            },
            BoundComment::Invalid {
                line: 5,
                error: BoundCommentError::MissingJustification,
            },
            BoundComment::Invalid {
                line: 6,
                error: BoundCommentError::MissingBound,
            },
        ]
    );
}

#[test]
fn reads_a_directive_inside_a_sql_template() {
    let source = "\
const hidden = \"-- no-mistakes-bound postgres-bounded-statements rows<=1: hidden\";
return query(`SELECT '-- no-mistakes-bound postgres-bounded-statements rows<=2: quoted'`);
return query(`
  -- no-mistakes-bound postgres-bounded-statements rows<=25: page of ids
  SELECT id FROM candidates WHERE id = ANY($1::uuid[])
`);
";
    assert_eq!(
        comments(source),
        vec![BoundComment::Valid {
            line: 4,
            rows: 25,
            justification: "page of ids".to_string(),
        }]
    );
}

#[test]
fn ignores_other_rules_strings_and_lookalike_words() {
    let source = "\
-- no-mistakes-bound postgres-explicit-columns rows<=10: not this rule
-- no-mistakes-boundary postgres-bounded-statements rows<=10: not a directive
const hidden = \"-- no-mistakes-bound postgres-bounded-statements rows<=1: hidden\";
-- no-mistakes-bound postgres-bounded-statements rows<=4: visible
";
    assert_eq!(
        comments(source),
        vec![BoundComment::Valid {
            line: 4,
            rows: 4,
            justification: "visible".to_string(),
        }]
    );
}
