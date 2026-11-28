include!("comment_parser/core.rs");
include!("comment_parser/comment_parser_html.rs");
include!("comment_parser/comment_parser_modes.rs");

pub fn rule_part_matches(rule_part: &str, rule_id: &str) -> bool {
    rule_part.strip_prefix(rule_id).is_some_and(|suffix| {
        suffix.is_empty() || suffix.starts_with(':') || suffix.starts_with(char::is_whitespace)
    })
}
