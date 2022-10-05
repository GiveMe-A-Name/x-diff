// the attribute is crate-level attribute.
// so must be in root module
#![feature(iterator_try_collect)]
pub mod cli;
pub mod config;
pub mod request;
pub mod response;
pub mod utils;

pub use cli::{Action, Args, RunArgs};
pub use config::{DiffConfig, DiffProfile, ResponseProfile};
pub use request::RequestProfile;
pub use response::ResponseExt;
pub use utils::{diff_text, highlight_text};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExtraArgs {
    pub headers: Vec<(String, String)>,
    pub query: Vec<(String, String)>,
    pub body: Vec<(String, String)>,
}
