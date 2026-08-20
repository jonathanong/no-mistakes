use sqlparser::ast::Expr;

pub(super) fn predicate_key(expr: &Expr) -> String {
    normalize_sql_key(&unwrap_expr(expr).to_string())
}

pub(super) fn not_null_predicate_column(expr: &Expr) -> Option<String> {
    match unwrap_expr(expr) {
        Expr::IsNotNull(inner) => expr_column_name(inner),
        _ => None,
    }
}

fn unwrap_expr(expr: &Expr) -> &Expr {
    match expr {
        Expr::Nested(inner) => unwrap_expr(inner),
        other => other,
    }
}

fn expr_column_name(expr: &Expr) -> Option<String> {
    match unwrap_expr(expr) {
        Expr::Identifier(ident) => Some(ident.value.clone()),
        Expr::CompoundIdentifier(parts) => parts.last().map(|ident| ident.value.clone()),
        _ => None,
    }
}

fn normalize_sql_key(sql: &str) -> String {
    let bytes = sql.as_bytes();
    let mut index = 0usize;
    let mut out = Vec::new();
    while index < bytes.len() {
        let next = bytes[index];
        if next.is_ascii_whitespace() {
            index += 1;
            continue;
        }
        if next == b'\'' || next == b'"' {
            let start = index;
            index = skip_quoted(bytes, index, next);
            out.push(sql[start..index].to_string());
            continue;
        }
        if next.is_ascii_alphabetic() || next == b'_' {
            let start = index;
            index += 1;
            while index < bytes.len()
                && (bytes[index].is_ascii_alphanumeric() || bytes[index] == b'_')
            {
                index += 1;
            }
            out.push(sql[start..index].to_ascii_lowercase());
            continue;
        }
        out.push(sql[index..index + 1].to_string());
        index += 1;
    }
    out.join(" ")
}

fn skip_quoted(bytes: &[u8], mut index: usize, quote: u8) -> usize {
    index += 1;
    while index < bytes.len() {
        if bytes[index] == quote {
            if quote == b'\'' && bytes.get(index + 1) == Some(&b'\'') {
                index += 2;
                continue;
            }
            return index + 1;
        }
        index += 1;
    }
    index
}

#[cfg(test)]
mod tests;
