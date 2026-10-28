use super::syntax;
use oxc_ast::ast::Program;

pub(crate) struct PreparedLoopHeads(pub(super) syntax::Syntax);

/// Negative source filtering skips the visitor entirely for ordinary max files.
/// Positive matches are confirmed by the existing heads visitor, then reused.
pub(crate) fn prepare_loop_heads(
    program: &Program<'_>,
    source: &str,
    append: bool,
    helpers: bool,
) -> Option<PreparedLoopHeads> {
    if !has_loop_word(source) {
        return None;
    }
    let syntax = syntax::collect_with_helpers(program, append, helpers, false);
    (!syntax.loop_candidates.is_empty() || append || helpers).then_some(PreparedLoopHeads(syntax))
}

fn has_loop_word(source: &str) -> bool {
    fn identifier(byte: u8) -> bool {
        byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$'
    }
    let bytes = source.as_bytes();
    ["for", "while"].into_iter().any(|word| {
        source.match_indices(word).any(|(start, _)| {
            (start == 0 || !identifier(bytes[start - 1]))
                && (start + word.len() == bytes.len() || !identifier(bytes[start + word.len()]))
        })
    })
}
