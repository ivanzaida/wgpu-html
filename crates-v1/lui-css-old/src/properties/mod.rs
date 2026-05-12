pub mod groups;
mod id;
mod meta;

pub use groups::PropertyGroup;
pub use id::PropertyId;
pub use meta::{is_inherited, is_shorthand, shorthand_longhands};
