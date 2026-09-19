//! Client-side token pricing for System One models.
//!
//! The price is a fixed constant for now, applied to models whose name starts
//! with `jev`. Other models have no known price and return `None`.

// region:    --- Types

/// The price in USD per one million tokens.
pub const PRICE_PER_MILLION_TOKENS: f64 = 0.042;

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
	price_per_million_tokens(model)
		.map(|price| price * input_tokens as f64 / 1_000_000.0)
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
}

// endregion: --- Tests
