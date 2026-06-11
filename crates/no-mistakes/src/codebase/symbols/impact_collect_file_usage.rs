fn file_entry_uses_any_symbol(root: &Path, file: &str, target_symbols: &BTreeSet<String>) -> bool {
    target_symbols
        .iter()
        .any(|target_symbol| file_entry_uses_symbol(root, file, target_symbol))
}

fn has_file_level_import_edge(via: &[EdgeKind]) -> bool {
    via.contains(&EdgeKind::DynamicImport) || via.contains(&EdgeKind::Require)
}

fn file_entry_uses_symbol(root: &Path, file: &str, target_symbol: &str) -> bool {
    let path = root.join(file);
    let Ok(source) = std::fs::read_to_string(&path) else {
        return false;
    };
    let mut facts_by_file = crate::codebase::ts_source::facts::collect_ts_facts(
        std::slice::from_ref(&path),
        crate::codebase::ts_source::facts::TsFactPlan::imports_and_symbols(),
    );
    let callees: BTreeSet<String> = facts_by_file
        .remove(&path)
        .map(|facts| {
            facts
                .function_calls
                .iter()
                .chain(facts.symbol_references.iter())
                .map(|call| call.callee.clone())
                .collect()
        })
        .unwrap_or_default();
    let module_bindings = dynamic_module_bindings(&source);
    if direct_dynamic_member_use(&source, target_symbol) {
        return true;
    }
    if module_bindings.iter().any(|binding| {
        let member = format!("{binding}.{target_symbol}");
        callees.contains(&member) || source_contains_member_name(&source, &member)
    }) {
        return true;
    }
    dynamic_symbol_aliases_in_source(&source, target_symbol)
        .iter()
        .any(|alias| callees.contains(alias) || source_contains_call_name(&source, alias))
}

fn direct_dynamic_member_use(source: &str, target_symbol: &str) -> bool {
    source
        .lines()
        .filter(|line| line.contains("import(") || line.contains("require("))
        .any(|line| line.contains(&format!(").{target_symbol}")))
}

fn source_contains_member_name(source: &str, member: &str) -> bool {
    source.match_indices(member).any(|(index, _)| {
        let after = source[index + member.len()..].chars().next();
        !after.is_some_and(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '$')
    })
}

fn source_contains_call_name(source: &str, name: &str) -> bool {
    source.match_indices(name).any(|(index, _)| {
        let before = source[..index].chars().next_back();
        let mut after = source[index + name.len()..].chars();
        !before.is_some_and(is_identifier_char)
            && after.find(|ch| !ch.is_whitespace()).is_some_and(|ch| ch == '(')
    })
}

fn is_identifier_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_' || ch == '$'
}

fn dynamic_module_bindings(source: &str) -> BTreeSet<String> {
    source
        .lines()
        .filter(|line| line.contains("import(") || line.contains("require("))
        .filter_map(|line| line.split_once('='))
        .filter_map(|(before, after)| {
            if after.contains(").") {
                return None;
            }
            identifier_after_declaration(before.trim())
        })
        .collect()
}

fn dynamic_symbol_aliases_in_source(source: &str, target_symbol: &str) -> BTreeSet<String> {
    let mut aliases = BTreeSet::new();
    if target_symbol.contains('.') {
        return aliases;
    }
    for line in source
        .lines()
        .filter(|line| line.contains("import(") || line.contains("require("))
    {
        aliases.extend(destructured_symbol_aliases(line, target_symbol));
        aliases.extend(member_assignment_alias(line, target_symbol));
    }
    aliases
}

fn destructured_symbol_aliases(line: &str, target_symbol: &str) -> BTreeSet<String> {
    let mut aliases = BTreeSet::new();
    let Some(start) = line.find('{') else {
        return aliases;
    };
    let Some(end) = line[start + 1..].find('}').map(|offset| start + 1 + offset) else {
        return aliases;
    };
    for part in line[start + 1..end].split(',').map(str::trim) {
        if part == target_symbol {
            aliases.insert(target_symbol.to_string());
        } else if let Some((name, alias)) = part.split_once(':') {
            if name.trim() == target_symbol {
                aliases.insert(alias.trim().to_string());
            }
        }
    }
    aliases
}

fn member_assignment_alias(line: &str, target_symbol: &str) -> BTreeSet<String> {
    let mut aliases = BTreeSet::new();
    let destructured = format!("{target_symbol}:");
    let mut rest = line;
    while let Some(index) = rest.find(&destructured) {
        let after = &rest[index + destructured.len()..];
        let trimmed = after.trim_start();
        let end = trimmed
            .char_indices()
            .take_while(|(_, ch)| ch.is_ascii_alphanumeric() || *ch == '_' || *ch == '$')
            .map(|(index, ch)| index + ch.len_utf8())
            .last();
        if let Some(end) = end {
            aliases.insert(trimmed[..end].to_string());
        }
        rest = after;
    }
    let member = format!(".{target_symbol}");
    if line.contains(&member) {
        if let Some((before_equals, _)) = line.split_once('=') {
            if let Some(name) = identifier_after_declaration(before_equals.trim()) {
                aliases.insert(name);
            }
        }
    }
    aliases
}

fn identifier_after_declaration(value: &str) -> Option<String> {
    let name = value
        .strip_prefix("const ")
        .or_else(|| value.strip_prefix("let "))
        .or_else(|| value.strip_prefix("var "))
        .map(str::trim)?;
    if name.starts_with('{') {
        return None;
    }
    let end = name
        .char_indices()
        .take_while(|(_, ch)| ch.is_ascii_alphanumeric() || *ch == '_' || *ch == '$')
        .map(|(index, ch)| index + ch.len_utf8())
        .last()?;
    Some(name[..end].to_string())
}
