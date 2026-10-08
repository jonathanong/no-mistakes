use super::output;
use crate::codebase::postgres::source::locations::Locations;

#[test]
fn empty_token_ranges_do_not_invent_source_spans() {
    assert!(output((0, 0), &[], &Locations::new("")).is_none());
    assert!(output((1, 2), &[], &Locations::new("")).is_none());
}
