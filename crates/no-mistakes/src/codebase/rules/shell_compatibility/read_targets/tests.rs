use super::super::lex::{Lexer, Shell};
use super::{decode_scalar_target, decode_target, targets, Targets};

fn parse_targets(source: &str) -> Targets {
    let mut parsed = None;
    Lexer::new(source, Shell::Bash).commands(|words, _, _| {
        parsed = targets(&words);
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
    assert_eq!(decode_target("'gate[0]'"), None);
    assert_eq!(decode_scalar_target("'gate[0]'").as_deref(), Some("gate"));
}

#[test]
fn read_array_mode_and_mapfile_callbacks() {
    let parsed = parse_targets("read -a arr gate < input");
    assert_eq!(parsed.names, vec!["arr"]);
    assert!(parsed.scalars.is_empty());

    let parsed = parse_targets("read -a 'gate[0]' extra < input");
    assert!(parsed.names.is_empty());
    assert!(parsed.scalars.is_empty());

    let parsed = parse_targets("mapfile -c1 -C wipe lines <<< x");
    assert_eq!(parsed.names, vec!["lines"]);
    assert_eq!(parsed.callback, Some(Some("wipe".to_owned())));

    let parsed = parse_targets("mapfile -c1 -C 'items=()' lines <<< x");
    assert_eq!(parsed.callback, Some(None));

    let parsed = parse_targets("mapfile -c1 -C '' lines <<< x");
    assert_eq!(parsed.callback, None);
}
