use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use super::{decoded_parts, SchemaCatalog};

impl SchemaCatalog {
    pub(crate) fn scalar_function_names(&self) -> Arc<BTreeSet<Vec<String>>> {
        Arc::clone(
            self.projections()
                .scalar_function_names
                .get_or_init(|| Arc::new(self.collect_scalar_function_names())),
        )
    }

    fn collect_scalar_function_names(&self) -> BTreeSet<Vec<String>> {
        let Some(schema) = self.schema.as_ref() else {
            return BTreeSet::new();
        };
        let mut overloads = BTreeMap::<Vec<String>, bool>::new();
        for function in self.functions.values() {
            let parts = decoded_parts(&function.name);
            let qualified = match parts.as_slice() {
                [name] => vec![schema.clone(), name.clone()],
                [_, _] => parts,
                _ => continue,
            };
            overloads
                .entry(qualified)
                .and_modify(|scalar| *scalar &= function.returns_set == Some(false))
                .or_insert(function.returns_set == Some(false));
        }
        let mut names = BTreeSet::new();
        for (name, scalar) in overloads {
            if scalar {
                if name[0] == *schema {
                    names.insert(vec![name[1].clone()]);
                }
                names.insert(name);
            }
        }
        names
    }
}
