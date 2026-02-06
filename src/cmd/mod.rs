pub mod branch;
pub mod commit;
pub mod context;
pub mod help;
pub mod install;
pub mod list;
pub mod new;
pub mod reset;
pub mod update;
pub mod uninstall;

pub const DEFAULT_TEMPLATE: &str = include_str!("../../templates/default.json");