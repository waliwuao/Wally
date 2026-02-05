pub mod add;
pub mod commit;
pub mod install;
pub mod list;
pub mod new;
pub mod push;
pub mod reset;
pub mod tree;
pub mod uninstall;

pub const DEFAULT_TEMPLATE: &str = include_str!("../../templates/default.json");