// region:    --- Modules

mod adapter_ctx;
mod adapter_dispatch;
mod adapter_kind;
mod adapter_trait;
pub(crate) mod adapters;
pub(crate) mod support;

pub use adapter_kind::AdapterKind;
pub(crate) use adapter_ctx::AdapterCtx;
pub(crate) use adapter_dispatch::exec;
pub(crate) use adapter_trait::Adapter;

// endregion: --- Modules
