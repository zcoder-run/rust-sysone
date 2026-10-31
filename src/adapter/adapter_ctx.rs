use crate::webc::Webc;

/// Borrowed execution context handed to an adapter for a single request.
#[derive(Debug, Clone, Copy)]
pub(crate) struct AdapterCtx<'a> {
	pub(crate) webc: &'a Webc,
	pub(crate) endpoint: &'a str,
	pub(crate) model: &'a str,
	pub(crate) api_key: &'a str,
	pub(crate) extra_headers: &'a [(String, String)],
}
