pub mod branch;
pub mod commit;
pub mod context;
pub mod install;
pub mod list;
pub mod new;
pub mod reset;
pub mod sync;
pub mod uninstall;

pub const DEFAULT_TEMPLATE: &str = include_str!("../../templates/default.json");