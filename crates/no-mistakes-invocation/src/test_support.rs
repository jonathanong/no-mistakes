#[expect(dead_code, reason = "materialize is used by integration tests")]
#[path = "../../no-mistakes/tests/common/saved_fixture.rs"]
mod saved_fixture;

pub(crate) use saved_fixture::materialize_path as materialize_saved_fixture;
