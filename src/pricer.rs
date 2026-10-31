//! Client-side token pricing for the supported providers.
//!
//! The System One (jev) price is a fixed constant, applied to models whose name
//! starts with `jev`. The OpenAI decision price bills uncached input tokens
//! only, applied to `gpt-6-luna`. Other models have no known price and return
//! `None`.

// region:    --- Types

/// The price in USD per one million tokens.
pub const PRICE_PER_MILLION_TOKENS: f64 = 0.042;

/// The OpenAI decision input price in USD per one million tokens.
///
/// Uncached input costs $0.10 per one million tokens.
pub const OPENAI_INPUT_PRICE_PER_MILLION_TOKENS: f64 = 0.10;

/// The OpenAI decision output price in USD per one million tokens.
///
/// The decisions endpoint does not charge for output tokens.
pub const OPENAI_OUTPUT_PRICE_PER_MILLION_TOKENS: f64 = 0.0;

// endregion: --- Types

// region:    --- Support

/// Returns the price in USD per one million tokens for the given model.
///
/// Only models whose name starts with `jev` are priced; other models return `None`.
pub fn price_per_million_tokens(model: &str) -> Option<f64> {
	if model.starts_with("jev") {
		Some(PRICE_PER_MILLION_TOKENS)
	} else {
		None
	}
}

/// Computes the USD cost for the given model and input token count.
///
/// Returns `None` when the model has no known price.
pub fn cost(model: &str, input_tokens: u64) -> Option<f64> {
	price_per_million_tokens(model).map(|price| price * input_tokens as f64 / 1_000_000.0)
}

/// Returns the OpenAI decision input and output prices in USD per one million tokens.
///
/// Only models whose name starts with `gpt-6-luna` are priced; other models return `None`.
pub fn openai_price_per_million_tokens(model: &str) -> Option<(f64, f64)> {
	if model.starts_with("gpt-6-luna") {
		Some((
			OPENAI_INPUT_PRICE_PER_MILLION_TOKENS,
			OPENAI_OUTPUT_PRICE_PER_MILLION_TOKENS,
		))
	} else {
		None
	}
}

/// Computes the USD cost for the given OpenAI decision model and token usage.
///
/// Input tokens are billed at the uncached input rate and output tokens at the
/// output rate. Returns `None` when the model has no known price.
pub fn openai_cost(model: &str, input_tokens: u64, output_tokens: u64) -> Option<f64> {
	let (input_price, output_price) = openai_price_per_million_tokens(model)?;
	Some(
		input_price * input_tokens as f64 / 1_000_000.0
			+ output_price * output_tokens as f64 / 1_000_000.0,
	)
}

// endregion: --- Support

// region:    --- Tests

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_pricer_price_per_million_tokens_jev_model_is_some() {
		// -- Setup & Fixtures
		let model = "jev1";

		// -- Exec
		let price = price_per_million_tokens(model);

		// -- Check
		assert_eq!(price, Some(PRICE_PER_MILLION_TOKENS));
	}

	#[test]
	fn test_pricer_price_per_million_tokens_other_model_is_none() {
		// -- Setup & Fixtures
		let model = "other-model";

		// -- Exec
		let price = price_per_million_tokens(model);

		// -- Check
		assert_eq!(price, None);
	}

	#[test]
	fn test_pricer_cost_jev_model_returns_expected_cost() -> Result<(), Box<dyn std::error::Error>> {
		// -- Setup & Fixtures
		let model = "jev1";
		let input_tokens = 2_000_000;

		// -- Exec
		let cost = cost(model, input_tokens).ok_or("cost should be some for jev model")?;

		// -- Check
		assert_eq!(cost, PRICE_PER_MILLION_TOKENS * 2.0);

		Ok(())
	}

	#[test]
	fn test_pricer_cost_other_model_is_none() {
		// -- Setup & Fixtures
		let model = "other-model";

		// -- Exec
		let cost = cost(model, 1_000_000);

		// -- Check
		assert_eq!(cost, None);
	}

	#[test]
	fn test_pricer_openai_price_per_million_tokens_gpt6luna_model_is_some() {
		// -- Setup & Fixtures
		let model = "gpt-6-luna";

		// -- Exec
		let prices = openai_price_per_million_tokens(model);

		// -- Check
		assert_eq!(
			prices,
			Some((
				OPENAI_INPUT_PRICE_PER_MILLION_TOKENS,
				OPENAI_OUTPUT_PRICE_PER_MILLION_TOKENS
			))
		);
	}

	#[test]
	fn test_pricer_openai_price_per_million_tokens_other_model_is_none() {
		// -- Setup & Fixtures
		let model = "other-model";

		// -- Exec
		let prices = openai_price_per_million_tokens(model);

		// -- Check
		assert_eq!(prices, None);
	}

	#[test]
	fn test_pricer_openai_cost_gpt6luna_model_returns_expected_cost() -> Result<(), Box<dyn std::error::Error>> {
		// -- Setup & Fixtures
		let model = "gpt-6-luna";
		let input_tokens = 1_000_000;
		let output_tokens = 1_000_000;

		// -- Exec
		let cost = openai_cost(model, input_tokens, output_tokens).ok_or("cost should be some for gpt-6-luna")?;

		// -- Check
		let expected = OPENAI_INPUT_PRICE_PER_MILLION_TOKENS + OPENAI_OUTPUT_PRICE_PER_MILLION_TOKENS;
		assert!((cost - expected).abs() < 1e-12, "cost {cost} should be near {expected}");

		Ok(())
	}

	#[test]
	fn test_pricer_openai_cost_zero_tokens_returns_zero() -> Result<(), Box<dyn std::error::Error>> {
		// -- Setup & Fixtures
		let model = "gpt-6-luna";

		// -- Exec
		let cost = openai_cost(model, 0, 0).ok_or("cost should be some for gpt-6-luna")?;

		// -- Check
		assert_eq!(cost, 0.0);

		Ok(())
	}

	#[test]
	fn test_pricer_openai_cost_other_model_is_none() {
		// -- Setup & Fixtures
		let model = "other-model";

		// -- Exec
		let cost = openai_cost(model, 1_000_000, 1_000_000);

		// -- Check
		assert_eq!(cost, None);
	}
}

// endregion: --- Tests
