pub mod discover;
pub mod schema;
pub mod test_plan;
pub mod view;

pub(crate) use discover::{
    effective_v2_config_path_from_visible, load_v2_config_from_source_store,
};
pub use discover::{find_config_root, load_v2_config, load_v2_config_from_visible};
pub use schema::NoMistakesConfig;
pub use view::ConfigView;

#[cfg(test)]
mod tests;
