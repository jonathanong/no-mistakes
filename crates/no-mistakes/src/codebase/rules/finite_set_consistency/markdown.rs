use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use std::collections::BTreeSet;

pub(super) fn extract_markdown_table_code_cells(source: &str) -> BTreeSet<String> {
    let mut values = BTreeSet::new();
    let mut in_table = false;
    let mut in_table_head = false;
    for (event, _) in Parser::new_ext(source, Options::all()).into_offset_iter() {
        match event {
            Event::Start(Tag::Table(_)) => in_table = true,
            Event::Start(Tag::TableHead) => in_table_head = true,
            Event::End(TagEnd::TableHead) => in_table_head = false,
            Event::End(TagEnd::Table) => in_table = false,
            Event::Code(value) if in_table && !in_table_head => {
                values.insert(value.to_string());
            }
            _ => {}
        }
    }
    values
}
