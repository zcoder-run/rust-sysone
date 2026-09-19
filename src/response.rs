//! Typed response returned by the client.

use serde_json::Value;

use crate::pricer::cost;
use crate::{Answer, QKey, Result};

// region:    --- Types

/// The typed response returned by the client, with the client-side cost.
#[derive(Debug, Clone)]
pub struct Response {
	/// The model used for the request.
	pub model: String,

	/// The answers, each one keyed by its question key.
	pub answers: Vec<(QKey, Answer)>,

	/// The number of input tokens used.
	pub input_tokens: u64,

	/// The number of output tokens used.
	pub output_tokens: u64,

	/// The USD cost, computed client-side from the input tokens. `None` when the model has no known price.
	pub cost: Option<f64>,
}

// endregion: --- Types

/// Constructors
impl Response {
	/// Parse a raw response `Value` into a typed `Response`.
	///
	/// The parsing is tolerant: a missing `model` defaults to an empty string, a missing or
	/// non-object `answers` yields no answers, and a missing token usage defaults to `0`. The
	/// token usage is read from `usage.input_tokens` and `usage.output_tokens`, falling back to
	/// the top-level `input_tokens` and `output_tokens` when absent. The `cost` is computed
	/// client-side from the input tokens.
	pub fn from_value(value: Value) -> Result<Self> {
		let model = value
			.get("model")
			.and_then(Value::as_str)
			.unwrap_or_default()
			.to_string();

		let answers = parse_answers(value.get("answers"))?;

		let input_tokens = parse_u64(value.get("usage"), "input_tokens")
			.or_else(|| value.get("input_tokens").and_then(Value::as_u64))
			.unwrap_or(0);

		let output_tokens = parse_u64(value.get("usage"), "output_tokens")
			.or_else(|| value.get("output_tokens").and_then(Value::as_u64))
			.unwrap_or(0);

		let cost = cost(&model, input_tokens);

		Ok(Self {
			model,
			answers,
			input_tokens,
			output_tokens,
			cost,
		})
	}
}

/// Accessors
impl Response {
	/// Look up an answer by key, returning `None` when no answer matches.
	pub fn answer(&self, key: impl Into<QKey>) -> Option<&Answer> {
		let key = key.into();
		self.answers
			.iter()
			.find_map(|(q_key, answer)| (q_key == &key).then_some(answer))
	}
}

// region:    --- Support

/// Parse the `answers` object into an ordered list of `(QKey, Answer)`.
///
/// A missing or non-object `answers` yields an empty list. Each key is mapped back into a `QKey`
/// (see `parse_q_key`), and each value is deserialized into an `Answer`.
fn parse_answers(answers: Option<&Value>) -> Result<Vec<(QKey, Answer)>> {
	let mut parsed = Vec::new();

	if let Some(Value::Object(answers)) = answers {
		for (key, value) in answers {
			let q_key = parse_q_key(key);
			let answer: Answer = serde_json::from_value(value.clone())?;
			parsed.push((q_key, answer));
		}
	}

	Ok(parsed)
}

/// Map a wire key back into a `QKey`, mapping `q<i>` to `QKey::Idx(i)` and anything else to `QKey::Name`.
fn parse_q_key(key: &str) -> QKey {
	key.strip_prefix('q')
		.and_then(|idx| idx.parse::<usize>().ok())
		.map(QKey::Idx)
		.unwrap_or_else(|| QKey::Name(key.to_string()))
}

/// Read a `u64` field from an optional object.
fn parse_u64(parent: Option<&Value>, key: &str) -> Option<u64> {
	parent
		.and_then(|value| value.get(key))
		.and_then(Value::as_u64)
}

// endregion: --- Support

// region:    --- Tests

#[cfg(test)]
mod tests {
	use super::*;
	use serde_json::json;

	use crate::pricer::PRICE_PER_MILLION_TOKENS;

	#[test]
	fn test_response_from_value_parses_full_response() -> Result<()> {
		// -- Setup & Fixtures
		let value = json!({
			"model": "jev1",
			"answers": {
				"q0": { "type": "noul", "noul": 0.42 },
				"intent": {
					"type": "choice",
					"choice": "a",
					"confidence": 0.9,
					"probabilities": { "a": 0.9, "b": 0.1 }
				}
			},
			"usage": { "input_tokens": 2000000, "output_tokens": 100 }
		});

		// -- Exec
		let response = Response::from_value(value)?;

		// -- Check
		assert_eq!(response.model, "jev1");
		assert_eq!(response.answers.len(), 2);
		assert_eq!(response.input_tokens, 2_000_000);
		assert_eq!(response.output_tokens, 100);

		let cost = response.cost.ok_or("cost should be some for jev model")?;
		assert!((cost - PRICE_PER_MILLION_TOKENS * 2.0).abs() < 1e-12);

		Ok(())
	}

	#[test]
	fn test_response_from_value_answer_lookup_by_idx_and_name() -> Result<()> {
		// -- Setup & Fixtures
		let value = json!({
			"model": "jev1",
			"answers": {
				"q0": { "type": "noul", "noul": 0.42 },
				"intent": {
					"type": "choice",
					"choice": "a",
					"confidence": 0.9,
					"probabilities": { "a": 0.9, "b": 0.1 }
				}
			}
		});

		// -- Exec
		let response = Response::from_value(value)?;

		// -- Check
		assert!(matches!(response.answer(0), Some(Answer::Noul(_))));
		assert!(matches!(response.answer("intent"), Some(Answer::Choice(_))));
		assert!(response.answer("missing").is_none());

		Ok(())
	}

	#[test]
	fn test_response_from_value_token_fallback_to_top_level() -> Result<()> {
		// -- Setup & Fixtures
		let value = json!({
			"model": "jev1",
			"input_tokens": 1000000,
			"output_tokens": 7
		});

		// -- Exec
		let response = Response::from_value(value)?;

		// -- Check
		assert_eq!(response.input_tokens, 1_000_000);
		assert_eq!(response.output_tokens, 7);

		let cost = response.cost.ok_or("cost should be some for jev model")?;
		assert!((cost - PRICE_PER_MILLION_TOKENS).abs() < 1e-12);

		Ok(())
	}

	#[test]
	fn test_response_from_value_tolerates_missing_fields() -> Result<()> {
		// -- Setup & Fixtures
		let value = json!({ "model": "other" });

		// -- Exec
		let response = Response::from_value(value)?;

		// -- Check
		assert_eq!(response.model, "other");
		assert!(response.answers.is_empty());
		assert_eq!(response.input_tokens, 0);
		assert_eq!(response.output_tokens, 0);
		assert_eq!(response.cost, None);

		Ok(())
	}
}

// endregion: --- Tests
