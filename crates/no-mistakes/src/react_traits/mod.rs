pub(crate) mod analyze;
pub(crate) mod pipeline;
pub(crate) mod report;
pub(crate) mod traits;

pub use pipeline::check::check_enabled;
pub use pipeline::check::run_check_with_facts;
pub use pipeline::run_analyze;
pub use pipeline::run_check;
pub use pipeline::{run_usages, UsagesInclude};
pub use report::text::{
    print_results, print_results_md, print_usages, print_usages_md, print_violations,
    print_violations_md,
};
pub use report::types::{AggregatedFacts, Callsite, ComponentFacts, UsagesReport, Violation};
