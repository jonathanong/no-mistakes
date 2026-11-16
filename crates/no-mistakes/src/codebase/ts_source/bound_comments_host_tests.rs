use super::{bound_comments, BoundComment};

fn comments(source: &str) -> Vec<BoundComment> {
    bound_comments(source, "postgres-bounded-statements")
}

#[test]
fn host_regex_continued_string_and_interpolation_braces_stay_out_of_sql() {
    let source = concat!(
        "const re = /`-- no-mistakes-bound postgres-bounded-statements rows<=1: in regex/;\n",
        "const quote = /['`\"]/;\n",
        "const escaped = /\\'/;\n",
        "return /`-- no-mistakes-bound postgres-bounded-statements rows<=1: keyword/;\n",
        "const divided = a/b;\n",
        "const bad = /unclosed\n",
        "const continued = '\\\n",
        "`-- no-mistakes-bound postgres-bounded-statements rows<=1: continued';\n",
        "const open = '\n",
        "`-- no-mistakes-bound postgres-bounded-statements rows<=1: open';\n",
        "const q = `SELECT ${\"{\"}\n",
        "  -- no-mistakes-bound postgres-bounded-statements rows<=9: braces\n",
        "`;\n",
        "const early = `SELECT ${\"} -- no-mistakes-bound postgres-bounded-statements rows<=1: early\"}\n",
        "`;\n",
        "const tick = `${\"`\"}`;\n",
    );
    assert_eq!(
        comments(source),
        vec![BoundComment::Valid {
            line: 12,
            rows: 9,
            justification: "braces".to_string(),
        }]
    );
}
