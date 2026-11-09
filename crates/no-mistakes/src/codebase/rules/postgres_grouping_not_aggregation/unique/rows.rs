use super::*;
pub(super) fn proven(
    relations: &[walk::Relation],
    keys: &[&Expr],
    select: &Select,
    catalog: &super::super::scope::CatalogScope<'_>,
    counted: Option<&Expr>,
) -> bool {
    for driver in relations {
        if !key_covers(
            driver,
            keys,
            select,
            catalog,
            &BTreeSet::new(),
            (relations.len() == 1, counted),
        ) {
            continue;
        }
        let mut known = BTreeSet::from([driver.alias.clone()]);
        loop {
            let before = known.len();
            for relation in relations {
                if !known.contains(&relation.alias)
                    && key_covers(relation, &[], select, catalog, &known, (false, None))
                {
                    known.insert(relation.alias.clone());
                }
            }
            if known.len() == relations.len() {
                return true;
            }
            if known.len() == before {
                break;
            }
        }
    }
    false
}
