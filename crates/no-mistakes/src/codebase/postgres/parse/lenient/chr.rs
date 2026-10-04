use sqlparser::tokenizer::Token;

pub(in crate::codebase::postgres::parse) fn expand_chr_encoded_sql(sql: &str) -> Option<String> {
    if !looks_like_chr_call(sql) {
        return None;
    }
    let mut tokens = super::super::unicode::tokenize(sql);
    if tokens.is_empty() {
        return None;
    }
    super::rewrite::rewrite_chr_calls(&mut tokens);
    super::recover::concatenated_strings(&tokens)
}

pub(in crate::codebase::postgres::parse) fn rewrite_chr_tokens(tokens: &mut Vec<Token>) {
    super::rewrite::rewrite_chr_calls(tokens);
}

fn looks_like_chr_call(sql: &str) -> bool {
    sql.to_ascii_lowercase()
        .split_whitespace()
        .collect::<String>()
        .contains("chr(")
}
