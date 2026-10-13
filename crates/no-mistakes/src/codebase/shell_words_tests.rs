use super::{split_quoted_words, Whitespace};

#[test]
fn whitespace_policy_is_preserved() {
    let input = "a\u{00a0}b";
    assert_eq!(
        split_quoted_words(input, Whitespace::Unicode),
        Some(vec!["a".into(), "b".into()])
    );
    assert_eq!(
        split_quoted_words(input, Whitespace::Ascii),
        Some(vec![input.into()])
    );
}

#[test]
fn quoted_words_preserve_escaped_characters_and_reject_unfinished_input() {
    assert_eq!(
        split_quoted_words(
            r#"one "two words" 'three words' 'mixed " quote' 'literal\ slash' four\ five"#,
            Whitespace::Unicode
        ),
        Some(vec![
            "one".into(),
            "two words".into(),
            "three words".into(),
            "mixed \" quote".into(),
            "literal\\ slash".into(),
            "four five".into()
        ])
    );
    assert_eq!(split_quoted_words("unfinished'", Whitespace::Unicode), None);
    assert_eq!(
        split_quoted_words("unfinished\\", Whitespace::Unicode),
        None
    );
}
