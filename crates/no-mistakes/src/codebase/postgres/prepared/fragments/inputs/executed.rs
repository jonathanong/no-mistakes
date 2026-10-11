use super::Input;
use crate::codebase::postgres::{EmbeddedSqlFileFacts, EmbeddedSqlKind, EmbeddedSqlVariant};
use crate::fx::{FxHashMap, FxHashSet};

type Key<'a> = (&'a str, &'a [(u32, u32)]);

pub(super) struct Executed<'a> {
    legacy: FxHashSet<Key<'a>>,
    variants: FxHashMap<Key<'a>, Vec<&'a EmbeddedSqlVariant>>,
}

impl<'a> Executed<'a> {
    pub(super) fn collect(file: &'a EmbeddedSqlFileFacts) -> Self {
        let mut legacy = FxHashSet::default();
        let mut variants: FxHashMap<_, Vec<_>> = FxHashMap::default();
        for call in &file.calls {
            if call.variants.is_empty() && call.kind != EmbeddedSqlKind::Dynamic {
                if let Some(sql) = &call.sql_text {
                    legacy.insert((
                        sql.as_str(),
                        call.recovered_placeholder_positions.as_slice(),
                    ));
                }
            }
            for variant in &call.variants {
                variants
                    .entry((
                        variant.sql_text.as_str(),
                        variant.recovered_placeholder_positions.as_slice(),
                    ))
                    .or_default()
                    .push(variant);
            }
        }
        Self { legacy, variants }
    }

    pub(super) fn contains(&self, fragment: &Input<'_>) -> bool {
        let key = (fragment.sql, fragment.binds);
        // Preserve the pre-variant text/bind projection for legacy inputs.
        if !fragment.enumerated && self.legacy.contains(&key) {
            return true;
        }
        self.variants.get(&key).is_some_and(|versions| {
            versions.iter().any(|version| {
                fragment
                    .append_site
                    .is_some_and(|site| version.append_sites.contains(&site))
                    && version.sql_source_offsets.as_slice() == fragment.origins
            })
        })
    }
}
