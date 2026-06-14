use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use std::collections::BTreeSet;

pub(super) fn extract_markdown_table_code_cells(source: &str) -> BTreeSet<String> {
    let mut values = BTreeSet::new();
    let mut in_table = false;
    for (event, _) in Parser::new_ext(source, Options::all()).into_offset_iter() {
        match event {
            Event::Start(Tag::Table(_)) => in_table = true,
            Event::End(TagEnd::Table) => in_table = false,
            Event::Code(value) if in_table => {
                values.insert(value.to_string());
            }
            _ => {}
        }
    }
    values
}
