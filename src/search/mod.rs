pub mod info;
pub mod move_picker;
pub mod pv;
#[allow(clippy::module_inception)]
pub mod search;
pub mod searcher;
pub mod time;

pub use info::*;
pub use move_picker::*;
pub use pv::*;
pub use search::*;
pub use searcher::*;
pub use time::*;

pub const MAX_PLY: usize = 256;
pub const MAX_DEPTH: u8 = 255;
