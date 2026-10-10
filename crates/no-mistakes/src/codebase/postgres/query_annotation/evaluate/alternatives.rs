//! Alternative arms restore input state and join effects conservatively.
//! Imported modules keep one request-local initialization identity.
mod arena;
mod bindings;
mod callback_seen;
mod extras;
pub(super) mod frames;
mod freshness;
mod merge;
pub(super) mod modules;
mod prune;
mod reachable;
mod run;
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
