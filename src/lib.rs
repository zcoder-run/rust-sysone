// region:    --- Modules

mod answer;
mod client;
mod client_builder;
mod error;
pub mod pricer;
mod qkey;
pub mod question;
mod request;
mod response;

pub use answer::{Answer, ChoiceAnswer, NoulAnswer, ScoreAnswer};
pub use client::*;
pub use client_builder::*;
pub use error::{Error, Result};
pub use pricer::{PRICE_PER_MILLION_TOKENS, cost, price_per_million_tokens};
pub use qkey::QKey;
pub use question::{ChoiceQuestion, NoulQuestion, Question, ScoreQuestion};
pub use request::*;
pub use response::Response;

// endregion: --- Modules
