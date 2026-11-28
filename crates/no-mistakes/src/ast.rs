pub use no_mistakes_ts_facts::ast::*;

pub fn parse_with_tokens<'a>(
    allocator: &'a oxc_allocator::Allocator,
    source: &'a str,
    source_type: oxc_span::SourceType,
) -> oxc_parser::ParserReturn<'a> {
    oxc_parser::Parser::new(allocator, source, source_type)
        .with_config(oxc_parser::config::TokensParserConfig)
        .parse()
}

#[cfg(test)]
pub(crate) mod tests;
