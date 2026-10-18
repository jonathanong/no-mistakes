//! Alternative arms restore input state and join effects conservatively.
//! Imported modules keep one request-local initialization identity.
pub(super) mod arena;
mod bindings;
mod callback_seen;
mod delta;
#[cfg(test)]
mod extras;
#[cfg(test)]
mod final_prune;
pub(super) mod frames;
mod generation;
mod merge;
pub(super) mod modules;
mod prune;
mod reachable;
mod run;
#[cfg(test)]
mod values;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod deletion_tests;

#[cfg(test)]
mod prefix_tests;

#[cfg(test)]
mod shared_module_tests;

#[cfg(test)]
mod callback_tests;

#[cfg(test)]
mod dense_fixture_tests;
