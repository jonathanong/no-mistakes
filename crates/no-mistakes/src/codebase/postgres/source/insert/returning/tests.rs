use super::cte_column_sources;
use crate::codebase::postgres::source::{locations::Locations, types::*};

#[test]
fn delegated_source_variant_is_not_used_for_an_unsupported_cte_source() {
    let locations = Locations::new("INSERT INTO t(id) VALUES (1) RETURNING id");
    let sources = cte_column_sources(
        &[],
        None,
        &PostgresSqlCteInsertSource::Unsupported {
            reason: "INSERT source".into(),
        },
        &locations,
    );
    assert!(matches!(
        sources,
        PostgresSqlInsertColumnSources::Unsupported {
            reason: PostgresSqlInsertColumnSourcesReason::ColumnsOmitted,
            ..
        }
    ));
    let defaults = cte_column_sources(
        &[],
        None,
        &PostgresSqlCteInsertSource::DefaultValues,
        &locations,
    );
    assert!(matches!(
        defaults,
        PostgresSqlInsertColumnSources::Unsupported {
            reason: PostgresSqlInsertColumnSourcesReason::ColumnsOmitted,
            ..
        }
    ));
}
