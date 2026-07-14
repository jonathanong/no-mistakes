pub mod discover;
pub mod schema;
pub mod test_plan;
pub mod view;

pub use discover::{find_config_root, load_v2_config, load_v2_config_from_visible};
pub use schema::NoMistakesConfig;
pub use view::ConfigView;

#[cfg(test)]
mod tests;
