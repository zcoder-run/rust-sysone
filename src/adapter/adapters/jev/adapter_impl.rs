use crate::adapter::{Adapter, AdapterCtx};
use crate::{Request, Response, Result};

use super::{build_payload, parse_response};

/// The stateless, zero sized Jev (TypeSafe System One) adapter.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct JevAdapter;

impl Adapter for JevAdapter {
	#[allow(clippy::manual_async_fn)] // for now
	fn exec(&self, ctx: AdapterCtx<'_>, request: &Request) -> impl Future<Output = Result<Response>> + Send {
		async move {
			let payload = build_payload(ctx.model, request)?;
			let body = ctx
				.webc
				.post_json(ctx.endpoint, ctx.api_key, ctx.extra_headers, &payload)
				.await?;

			let mut response = parse_response(body)?;
			response.cost = self.cost(&response.model, response.input_tokens, response.output_tokens);

			Ok(response)
		}
	}

	fn cost(&self, model: &str, input_tokens: u64, _output_tokens: u64) -> Option<f64> {
		crate::pricer::cost(model, input_tokens)
	}
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	use super::*;
	use crate::PRICE_PER_MILLION_TOKENS;

	#[test]
	fn test_adapter_jev_adapter_cost_priced_model() {
		// -- Setup & Fixtures
		let adapter = JevAdapter;
		let input_tokens = 2_000_000;

		// -- Exec
		let cost = adapter.cost("jev1", input_tokens, 0);

		// -- Check
		assert_eq!(cost, Some(PRICE_PER_MILLION_TOKENS * 2.0));
	}

	#[test]
	fn test_adapter_jev_adapter_cost_other_model_is_none() {
		// -- Setup & Fixtures
		let adapter = JevAdapter;

		// -- Exec
		let cost = adapter.cost("other-model", 1_000_000, 0);

		// -- Check
		assert_eq!(cost, None);
	}
}

// endregion: --- Tests
