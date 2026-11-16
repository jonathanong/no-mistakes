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

#[test]
fn postfix_division_and_nested_templates_keep_the_next_bound() {
    let source = concat!(
        "const label = `page ${count++ / 2}`;\n",
        "const bang = `page ${count! / 2}`;\n",
        "const minus = `page ${count--/2}`;\n",
        "const spaced = `page ${count-- / 2}`;\n",
        "const plus = +/'/;\n",
        "const nested = `${`it's`}`;\n",
        "const inner = `${`${'x'}`}`;\n",
        "const q = `\n",
        "  -- no-mistakes-bound postgres-bounded-statements rows<=4: after\n",
        "`;\n",
    );
    assert_eq!(
        comments(source),
        vec![BoundComment::Valid {
            line: 9,
            rows: 4,
            justification: "after".to_string(),
        }]
    );
}

#[test]
fn quote_escapes_and_nested_braces_keep_the_next_bound() {
    let source = concat!(
        "const quoted = 'it\\'s';\n",
        "const nested = `${`a\\``}`;\n",
        "const block = `${`a${b({c:1})}`}`;\n",
        "const obj = { n: 1 };\n",
        "const q = `\n",
        "  -- no-mistakes-bound postgres-bounded-statements rows<=4: after\n",
        "`;\n",
    );
    assert_eq!(
        comments(source),
        vec![BoundComment::Valid {
            line: 6,
            rows: 4,
            justification: "after".to_string(),
        }]
    );
}

#[test]
fn control_regex_identifier_of_and_a_trailing_dash_keep_the_bound() {
    let source = concat!(
        "if (ready) /`-- no-mistakes-bound postgres-bounded-statements rows<=1: regex/.test(text);\n",
        "const of = 8;\n",
        "const label = `page ${of / 2}`;\n",
        "for (const x of /`-- no-mistakes-bound postgres-bounded-statements rows<=1: for of/) {}\n",
        "const q = `SELECT 1 -\n",
        "  1\n",
        "  -- no-mistakes-bound postgres-bounded-statements rows<=1: bounded\n",
        "`;\n",
    );
    assert_eq!(
        comments(source),
        vec![BoundComment::Valid {
            line: 7,
            rows: 1,
            justification: "bounded".to_string(),
        }]
    );
}

#[test]
fn for_initializer_of_stays_division_and_for_of_regex_stays_closed() {
    let source = concat!(
        "const of = 8;\n",
        "const label = `page ${(() => { for (let i = of / 2; i < 5; i++) {} return 1; })()}`;\n",
        "const plain = `page ${(() => { for (of / 2; i < 3; i++) {} return 1; })()}`;\n",
        "const comma = `page ${(() => { for (let i = 1, of / 2; i < 3; i++) {} return 1; })()}`;\n",
        "const clause = `page ${(() => { for (let i = 0; of / 2; i++) {} return 1; })()}`;\n",
        "const call = `page ${(() => { for (of foo(1) / 2; i < 3; i++) {} return 1; })()}`;\n",
        "const index = `page ${(() => { for (of a[0] / 2; i < 3; i++) {} return 1; })()}`;\n",
        "const block = `page ${(() => { for (of a{1} / 2; i < 3; i++) {} return 1; })()}`;\n",
        "const brackets = `n ${for (const x of /keep/) {} [of / 2]}`;\n",
        "const braces = `n ${for (const x of /keep/) {} {of / 2}}`;\n",
        "const called = `page ${foo(bar) / 2}`;\n",
        "const awaited = `page ${await (of / 2)}`;\n",
        "for (const x of /a;`-- no-mistakes-bound postgres-bounded-statements rows<=1: semi/) {}\n",
        "for (const {a = 1} of /`-- no-mistakes-bound postgres-bounded-statements rows<=1: def/) {}\n",
        "for (const [a, b] of /`-- no-mistakes-bound postgres-bounded-statements rows<=1: pair/) {}\n",
        "for await (const x of /`-- no-mistakes-bound postgres-bounded-statements rows<=1: awaited/) {}\n",
        "for (const x of /[a/;]`-- no-mistakes-bound postgres-bounded-statements rows<=1: class/) {}\n",
        "for (const x of /a\\/;`-- no-mistakes-bound postgres-bounded-statements rows<=1: esc/) {}\n",
        "const control = `page ${if (ready) / -- no-mistakes-bound postgres-bounded-statements rows<=1: regex/}`;\n",
        "const q = `\n",
        "  -- no-mistakes-bound postgres-bounded-statements rows<=1: bounded\n",
        "`;\n",
    );
    assert_eq!(
        comments(source),
        vec![BoundComment::Valid {
            line: 21,
            rows: 1,
            justification: "bounded".to_string(),
        }]
    );
}
