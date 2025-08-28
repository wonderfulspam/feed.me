mod defaults;
mod global;
mod loader;
mod merge;
mod methods;
mod save;
mod tests;
mod types;

// Re-export main types and functions
pub use global::{get_config, init_config};
use merge::ConfigMerger;
use save::ConfigSaver;
pub use types::*;
