use crate::adapter::{Adapter, AdapterCtx};
use crate::pricer::openai_cost;
use crate::{Request, Response, Result};

use super::{build_payload, parse_response};

/// The stateless, zero sized OpenAI decisions adapter.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct OpenAiAdapter;

impl Adapter for OpenAiAdapter {
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

	fn cost(&self, model: &str, input_tokens: u64, output_tokens: u64) -> Option<f64> {
		// The decisions rate table bills uncached input tokens; output tokens are not charged.
		openai_cost(model, input_tokens, output_tokens)
	}
}
