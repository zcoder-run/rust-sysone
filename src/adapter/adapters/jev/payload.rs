use crate::adapter::support::parse_q_key;
use crate::{Answer, QKey, Request, Response, Result};
use serde_json::Value;

/// Build the jev wire payload by serializing the request and inserting the `model`.
pub(crate) fn build_payload(model: &str, request: &Request) -> Result<Value> {
	let mut payload = serde_json::to_value(request)?;
	if let Value::Object(map) = &mut payload {
		map.insert("model".to_string(), Value::String(model.to_string()));
	}

	Ok(payload)
}

/// Parse a raw jev response `Value` into a typed `Response`, leaving `cost` as `None`.
///
/// The parsing is tolerant: a missing `model` defaults to an empty string, a missing or
/// non-object `answers` yields no answers, and a missing token usage defaults to `0`. The
/// token usage is read from `usage.input_tokens` and `usage.output_tokens`, falling back to
/// the top-level `input_tokens` and `output_tokens` when absent.
pub(crate) fn parse_response(value: Value) -> Result<Response> {
	let model = value.get("model").and_then(Value::as_str).unwrap_or_default().to_string();

	let answers = parse_answers(value.get("answers"))?;

	let input_tokens = parse_u64(value.get("usage"), "input_tokens")
		.or_else(|| value.get("input_tokens").and_then(Value::as_u64))
		.unwrap_or(0);

	let output_tokens = parse_u64(value.get("usage"), "output_tokens")
		.or_else(|| value.get("output_tokens").and_then(Value::as_u64))
		.unwrap_or(0);

	Ok(Response {
		model,
		answers,
		input_tokens,
		output_tokens,
		cost: None,
	})
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

/// Read a `u64` field from an optional object.
fn parse_u64(parent: Option<&Value>, key: &str) -> Option<u64> {
	parent.and_then(|value| value.get(key)).and_then(Value::as_u64)
}

// endregion: --- Support

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

	use super::*;
	use crate::Question;
	use serde_json::json;

	#[test]
	fn test_adapter_jev_payload_build_payload() -> Result<()> {
		// -- Setup & Fixtures
		let request = Request::from_state("state")
			.append_question(0, Question::noul("Check"))
			.append_question("intent", Question::choice("Pick"));

		// -- Exec
		let payload = build_payload("jev1", &request)?;

		// -- Check
		assert_eq!(payload["model"], "jev1");
		assert_eq!(payload["state"], "state");
		let questions = payload
			.get("questions")
			.and_then(Value::as_object)
			.ok_or("expected questions object")?;
		assert!(questions.contains_key("q0"));
		assert!(questions.contains_key("intent"));

		Ok(())
	}

	#[test]
	fn test_adapter_jev_payload_parse_response_full() -> Result<()> {
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
		let response = parse_response(value)?;

		// -- Check
		assert_eq!(response.model, "jev1");
		assert_eq!(response.answers.len(), 2);
		assert_eq!(response.input_tokens, 2_000_000);
		assert_eq!(response.output_tokens, 100);
		assert_eq!(response.cost, None);

		Ok(())
	}

	#[test]
	fn test_adapter_jev_payload_parse_response_answer_lookup_by_idx_and_name() -> Result<()> {
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
		let response = parse_response(value)?;

		// -- Check
		assert!(matches!(response.answer(0), Some(Answer::Noul(_))));
		assert!(matches!(response.answer("intent"), Some(Answer::Choice(_))));
		assert!(response.answer("missing").is_none());

		Ok(())
	}

	#[test]
	fn test_adapter_jev_payload_parse_response_token_fallback_to_top_level() -> Result<()> {
		// -- Setup & Fixtures
		let value = json!({
			"model": "jev1",
			"input_tokens": 1000000,
			"output_tokens": 7
		});

		// -- Exec
		let response = parse_response(value)?;

		// -- Check
		assert_eq!(response.input_tokens, 1_000_000);
		assert_eq!(response.output_tokens, 7);
		assert_eq!(response.cost, None);

		Ok(())
	}

	#[test]
	fn test_adapter_jev_payload_parse_response_tolerates_missing_fields() -> Result<()> {
		// -- Setup & Fixtures
		let value = json!({ "model": "other" });

		// -- Exec
		let response = parse_response(value)?;

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
