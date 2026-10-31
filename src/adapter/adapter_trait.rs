use crate::adapter::AdapterCtx;
use crate::{Request, Response, Result};

/// The contract every provider adapter implements, executed via static dispatch.
pub(crate) trait Adapter: Send + Sync {
	/// Execute one request against the provider and return the typed response.
	fn exec(&self, ctx: AdapterCtx<'_>, request: &Request) -> impl Future<Output = Result<Response>> + Send;

	/// Compute the USD cost for the given model and token usage.
	fn cost(&self, model: &str, input_tokens: u64, output_tokens: u64) -> Option<f64>;
}
