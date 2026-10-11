use crate::codebase::postgres::EmbeddedSqlFileFacts;
mod executed;

pub(super) struct Input<'a> {
    pub line: u32,
    pub original_line: u32,
    pub append_site: Option<u32>,
    pub sql: &'a str,
    pub binds: &'a [(u32, u32)],
    pub origins: &'a [u32],
    pub enumerated: bool,
}

pub(super) fn collect(file: &EmbeddedSqlFileFacts) -> Vec<Input<'_>> {
    let executed = executed::Executed::collect(file);
    let located = file.calls.iter().any(|call| !call.variants.is_empty());
    let mut out = Vec::new();
    for (index, fragment) in file.fragments.iter().enumerate() {
        let variants = file
            .fragment_variants
            .get(index)
            .map(Vec::as_slice)
            .unwrap_or_default();
        if let Some(sql) = &fragment.sql_text {
            let origins = located
                .then(|| {
                    variants.iter().find(|variant| {
                        variant.sql_text == *sql
                            && variant.recovered_placeholder_positions
                                == fragment.recovered_placeholder_positions
                    })
                })
                .flatten()
                .map(|variant| variant.sql_source_offsets.as_slice())
                .unwrap_or_default();
            out.push(Input {
                line: fragment.line,
                original_line: fragment.line,
                append_site: file.fragment_sites.get(index).copied().flatten(),
                sql,
                binds: &fragment.recovered_placeholder_positions,
                origins,
                enumerated: false,
            });
        } else {
            out.extend(variants.iter().map(|variant| Input {
                line: variant.line,
                original_line: fragment.line,
                append_site: file.fragment_sites.get(index).copied().flatten(),
                sql: &variant.sql_text,
                binds: &variant.recovered_placeholder_positions,
                origins: &variant.sql_source_offsets,
                enumerated: true,
            }));
        }
    }
    out.retain(|fragment| !executed.contains(fragment));
    out
}
