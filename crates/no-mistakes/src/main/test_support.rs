#[cfg(feature = "test-instrumentation")]
#[path = "../test_support/git.rs"]
mod git;
#[path = "../test_support/gitignore_fixture.rs"]
mod gitignore_fixture;

#[cfg(feature = "test-instrumentation")]
pub(crate) use git::{git_add_all, git_init};
pub(crate) use gitignore_fixture::materialize_gitignore_fixture;
#[cfg(any(test, feature = "test-instrumentation"))]
pub(crate) use gitignore_fixture::materialize_saved_fixture;
