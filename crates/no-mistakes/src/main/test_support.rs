#[path = "../test_support/git.rs"]
mod git;
#[path = "../test_support/gitignore_fixture.rs"]
mod gitignore_fixture;

pub(crate) use git::{git_add_all, git_init};
pub(crate) use gitignore_fixture::{materialize_gitignore_fixture, materialize_saved_fixture};
