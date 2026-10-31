// region:    --- Modules

mod adapter_impl;
mod payload;

pub(crate) use adapter_impl::OpenAiAdapter;
pub(crate) use payload::{build_payload, parse_response};

// endregion: --- Modules
