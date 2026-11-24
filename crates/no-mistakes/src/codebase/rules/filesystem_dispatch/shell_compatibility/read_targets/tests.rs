use super::super::lex::{Lexer, Shell};
use super::{decode_target, targets, Targets};

fn parse_targets(source: &str) -> Targets {
    let mut parsed = None;
    Lexer::new(source, Shell::Bash).commands(|words, _| {
        parsed = targets(words);
    });
    parsed.expect("read/mapfile command")
}

#[test]
fn targets_handle_attached_option_arguments() {
    let parsed = parse_targets("read -raitems <<< ''");
    assert_eq!(parsed.names, vec!["items"]);
    assert!(!parsed.keep);

    let parsed = parse_targets("mapfile -O1 lines < input");
    assert_eq!(parsed.names, vec!["lines"]);
    assert!(parsed.keep);
}

#[test]
fn decode_target_matches_bash_quoting_and_escapes() {
    assert_eq!(decode_target("items").as_deref(), Some("items"));
    assert_eq!(decode_target("\"items\"").as_deref(), Some("items"));
    assert_eq!(decode_target("'items'").as_deref(), Some("items"));
    assert_eq!(decode_target("es\\caped").as_deref(), Some("escaped"));
    assert_eq!(decode_target("a\\\nb").as_deref(), Some("ab"));
    assert_eq!(decode_target("\"a\\\nb\"").as_deref(), Some("ab"));
    assert_eq!(decode_target("\"ke\\pt\""), None);
    assert_eq!(decode_target("\"a\\$b\""), None);
    assert_eq!(decode_target("a\\"), None);
    assert_eq!(decode_target("\"a\\"), None);
    assert_eq!(decode_target("\"unclosed"), None);
}
