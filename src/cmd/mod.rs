pub mod add;
pub mod commit;
pub mod list;
pub mod new;
pub mod push;
pub mod tree;

pub const DEFAULT_TEMPLATE: &str = include_str!("../../templates/default.json");