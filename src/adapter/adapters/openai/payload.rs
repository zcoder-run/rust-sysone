use crate::adapter::support::parse_q_key;
use crate::{Answer, QKey, Question, Request, Response, Result};
use serde_json::{Value, json};
use value_ext::JsonValueExt;

/// Build the OpenAI decisions payload from the canonical request.
///
/// The `state` becomes the shared `input`, and each question is rendered into the decisions
/// `questions` array under its `QKey::wire()` name, with its `criteria` mapped to `choices`
/// (choice questions) or `levels` (score questions).
pub(crate) fn build_payload(model: &str, request: &Request) -> Result<Value> {
	let input = build_input(&request.state);

	let mut questions = Vec::with_capacity(request.questions.len());
	for (q_key, question) in &request.questions {
		questions.push(build_question(&q_key.wire(), question)?);
	}

	let payload = json!({
		"model": model,
		"input": input,
		"questions": questions,
	});

	Ok(payload)
}

/// Parse a raw OpenAI decisions response `Value` into a typed `Response`, leaving `cost` as `None`.
///
/// The parsing is tolerant: a missing `model` defaults to an empty string, a missing or
/// non-array `answers` yields no answers, and a missing token usage defaults to `0`. The
/// token usage is read from `usage.input_tokens` and `usage.output_tokens`, falling back to
/// the top-level `input_tokens` and `output_tokens` when absent.
pub(crate) fn parse_response(value: Value) -> Result<Response> {
	let model = value.x_get_str("model").unwrap_or_default().to_string();

	let answers = parse_answers(&value)?;

	let input_tokens = parse_tokens(&value, "input_tokens");
	let output_tokens = parse_tokens(&value, "output_tokens");

	Ok(Response {
		model,
		answers,
		input_tokens,
		output_tokens,
		cost: None,
	})
}

// region:    --- Support

/// Build the shared `input` messages from the canonical `state`.
///
/// A string state becomes one `input_text` part, while any other state becomes an `input_json`
/// part, since the decisions API carries structured evidence only in a user message.
fn build_input(state: &Value) -> Value {
	let part = match state {
		Value::String(text) => json!({ "type": "input_text", "text": text }),
		other => json!({ "type": "input_json", "data": other }),
	};

	json!([{ "role": "user", "content": [part] }])
}

/// Render one canonical question into a decisions question, named by its wire key.
///
/// The `instructions` are forwarded as-is, because the decisions API accepts either a string
/// or a content parts array.
fn build_question(wire_key: &str, question: &Question) -> Result<Value> {
	let question_value = serde_json::to_value(question)?;
	let instructions = question_value.get("instructions").cloned().unwrap_or_default();

	let rendered = match question {
		Question::Noul(_) => json!({
			"type": "predicate",
			"name": wire_key,
			"instructions": instructions,
		}),
		Question::Choice(_) => json!({
			"type": "choice",
			"name": wire_key,
			"instructions": instructions,
			"choices": build_choices(&question_value),
		}),
		Question::Score(_) => json!({
			"type": "score",
			"name": wire_key,
			"instructions": instructions,
			"levels": build_levels(&question_value),
		}),
	};

	Ok(rendered)
}

/// Map the canonical choice `criteria` map to decisions `choices`.
fn build_choices(question_value: &Value) -> Value {
	let Ok(criteria) = question_value.x_get_object("criteria") else {
		return Value::Array(Vec::new());
	};

	let choices = criteria
		.iter()
		.map(|(value, description)| json!({ "value": value, "description": description }))
		.collect::<Vec<_>>();

	Value::Array(choices)
}

/// Map the canonical score `criteria` levels to decisions `levels`, lowest to highest.
fn build_levels(question_value: &Value) -> Value {
	let Ok(levels) = question_value.x_get_as::<&Vec<Value>>("criteria") else {
		return Value::Array(Vec::new());
	};

	let levels = levels
		.iter()
		.map(|level| json!({ "label": level }))
		.collect::<Vec<_>>();

	Value::Array(levels)
}

/// Parse the `answers` array into an ordered list of `(QKey, Answer)`.
///
/// A missing or non-array `answers` yields an empty list. Each key is mapped back into a `QKey`
/// (see `parse_q_key`), and each decisions answer is normalized into the canonical `Answer` shape.
fn parse_answers(value: &Value) -> Result<Vec<(QKey, Answer)>> {
	let Ok(answers) = value.x_get_as::<&Vec<Value>>("answers") else {
		return Ok(Vec::new());
	};

	let mut parsed = Vec::with_capacity(answers.len());

	for answer in answers {
		if let Some(parsed_answer) = parse_answer(answer)? {
			parsed.push(parsed_answer);
		}
	}

	Ok(parsed)
}

/// Parse one decisions answer into its canonical `(QKey, Answer)` pair.
///
/// Returns `None` for answers that have no canonical variant yet, such as `refusal`.
fn parse_answer(answer: &Value) -> Result<Option<(QKey, Answer)>> {
	let Some(name) = answer.x_get_str("name").ok() else {
		return Ok(None);
	};

	let canonical = match answer.x_get_str("type").unwrap_or_default() {
		"predicate" => {
			let probability = answer.x_get_f64("probability").unwrap_or_default();
			json!({ "type": "noul", "noul": probability })
		}
		"choice" => {
			let choice = answer.x_get_str("choice").unwrap_or_default();
			let confidence = answer.x_get_f64("confidence").unwrap_or_default();
			json!({
				"type": "choice",
				"choice": choice,
				"confidence": confidence,
				"probabilities": collect_probabilities(answer, "value"),
			})
		}
		"score" => {
			let score = answer.x_get_f64("score").unwrap_or_default();
			let confidence = answer.x_get_f64("confidence").unwrap_or_default();
			json!({
				"type": "score",
				"score": score,
				"confidence": confidence,
				"probabilities": collect_probabilities(answer, "label"),
				"legend": {},
			})
		}
		_ => return Ok(None),
	};

	let typed: Answer = serde_json::from_value(canonical)?;

	Ok(Some((parse_q_key(name), typed)))
}

/// Collect a decisions `probabilities` array into a `{ key: probability }` object.
///
/// The `key_field` selects what identifies each entry: `value` for choice answers, and `label`
/// for score answers, where the entries are ordered level indices.
fn collect_probabilities(answer: &Value, key_field: &str) -> Value {
	let Ok(items) = answer.x_get_as::<&Vec<Value>>("probabilities") else {
		return Value::Object(serde_json::Map::new());
	};

	let mut probabilities = serde_json::Map::with_capacity(items.len());
	for item in items {
		let Some(key) = item.get(key_field) else {
			continue;
		};
		let key = key.as_str().map(str::to_string).unwrap_or_else(|| key.to_string());
		let probability = item.x_get_f64("probability").unwrap_or_default();
		probabilities.insert(key, json!(probability));
	}

	Value::Object(probabilities)
}

/// Read a `u64` token count from `usage.<name>`, falling back to the top-level `<name>`.
fn parse_tokens(value: &Value, name: &str) -> u64 {
	value
		.x_get::<u64>(&format!("/usage/{name}"))
		.or_else(|_| value.x_get::<u64>(name))
		.unwrap_or(0)
}

// endregion: --- Support

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

	use super::*;
	use crate::{ChoiceQuestion, ScoreQuestion};

	#[test]
	fn test_adapter_openai_payload_build_payload() -> Result<()> {
		// -- Setup & Fixtures
		let request = Request::from_state(json!({ "code": "fn main() {}" }))
			.append_question(0, Question::noul("Check the code"))
			.append_question(
				"intent",
				ChoiceQuestion::new("Pick a department")
					.append_criteria("billing", "Billing questions")
					.append_criteria("technical", "Technical questions"),
			);

		// -- Exec
		let payload = build_payload("gpt-6-luna", &request)?;

		// -- Check
		assert_eq!(payload["model"], "gpt-6-luna");

		let content = payload["input"][0]["content"].as_array().ok_or("should have input parts")?;
		assert_eq!(content[0]["type"], "input_json");
		assert_eq!(content[0]["data"]["code"], "fn main() {}");

		let questions = payload["questions"].as_array().ok_or("should have questions")?;
		assert_eq!(questions.len(), 2);

		let q0 = questions
			.iter()
			.find(|question| question["name"] == "q0")
			.ok_or("should have the indexed question")?;
		assert_eq!(q0["type"], "predicate");
		assert_eq!(q0["instructions"], "Check the code");

		let intent = questions
			.iter()
			.find(|question| question["name"] == "intent")
			.ok_or("should have the named question")?;
		assert_eq!(intent["type"], "choice");
		assert_eq!(intent["instructions"], "Pick a department");

		let choices = intent["choices"].as_array().ok_or("should have choices")?;
		let billing = choices
			.iter()
			.find(|choice| choice["value"] == "billing")
			.ok_or("should have the billing choice")?;
		assert_eq!(billing["description"], "Billing questions");

		Ok(())
	}

	#[test]
	fn test_adapter_openai_payload_build_payload_string_state() -> Result<()> {
		// -- Setup & Fixtures
		let request = Request::from_state("I was charged twice for my order.")
			.append_question("department", Question::noul("Is this a billing complaint?"));

		// -- Exec
		let payload = build_payload("gpt-6-luna", &request)?;

		// -- Check
		assert_eq!(payload["input"][0]["role"], "user");
		assert_eq!(payload["input"][0]["content"][0]["type"], "input_text");
		assert_eq!(payload["input"][0]["content"][0]["text"], "I was charged twice for my order.");
		assert_eq!(payload["questions"][0]["type"], "predicate");
		assert_eq!(payload["questions"][0]["name"], "department");

		Ok(())
	}

	#[test]
	fn test_adapter_openai_payload_build_payload_score_levels() -> Result<()> {
		// -- Setup & Fixtures
		let request = Request::from_state("Export fails in Safari.").append_question(
			"severity",
			ScoreQuestion::new("How severe is this issue?")
				.append_level("Cosmetic")
				.append_level("Fully blocked"),
		);

		// -- Exec
		let payload = build_payload("gpt-6-luna", &request)?;

		// -- Check
		let question = &payload["questions"][0];
		assert_eq!(question["type"], "score");

		let levels = question["levels"].as_array().ok_or("should have levels")?;
		assert_eq!(levels.len(), 2);
		assert_eq!(levels[0]["label"], "Cosmetic");
		assert_eq!(levels[1]["label"], "Fully blocked");

		Ok(())
	}

	#[test]
	fn test_adapter_openai_payload_parse_response_full() -> Result<()> {
		// -- Setup & Fixtures
		let value = json!({
			"model": "gpt-6-luna",
			"answers": [
				{ "type": "predicate", "name": "q0", "probability": 0.92 },
				{
					"type": "choice",
					"name": "intent",
					"choice": "billing",
					"confidence": 0.93,
					"probabilities": [
						{ "value": "billing", "probability": 0.95 },
						{ "value": "technical", "probability": 0.05 }
					]
				},
				{
					"type": "score",
					"name": "severity",
					"score": 1.1,
					"confidence": 0.55,
					"probabilities": [
						{ "value": 0, "label": "Cosmetic", "probability": 0.1 },
						{ "value": 1, "label": "Workaround available", "probability": 0.7 },
						{ "value": 2, "label": "Fully blocked", "probability": 0.2 }
					]
				}
			],
			"usage": { "input_tokens": 2000000 }
		});

		// -- Exec
		let response = parse_response(value)?;

		// -- Check
		assert_eq!(response.model, "gpt-6-luna");
		assert_eq!(response.answers.len(), 3);
		assert_eq!(response.input_tokens, 2_000_000);
		assert_eq!(response.output_tokens, 0);
		assert_eq!(response.cost, None);

		let noul = response.answer(0).ok_or("should have the q0 answer")?;
		assert!(matches!(noul, Answer::Noul(noul) if noul.noul == 0.92));

		let choice = response.answer("intent").ok_or("should have the intent answer")?;
		if let Answer::Choice(choice) = choice {
			assert_eq!(choice.choice, "billing");
			assert_eq!(choice.confidence, 0.93);
			assert_eq!(choice.probabilities.get("billing"), Some(&0.95));
		} else {
			return Err("should be a choice answer".into());
		}

		let score = response.answer("severity").ok_or("should have the severity answer")?;
		if let Answer::Score(score) = score {
			assert_eq!(score.score, 1.1);
			assert_eq!(score.probabilities.get("Workaround available"), Some(&0.7));
		} else {
			return Err("should be a score answer".into());
		}

		Ok(())
	}

	#[test]
	fn test_adapter_openai_payload_parse_response_token_fallback_to_top_level() -> Result<()> {
		// -- Setup & Fixtures
		let value = json!({
			"model": "gpt-6-luna",
			"input_tokens": 1000000,
			"output_tokens": 7
		});

		// -- Exec
		let response = parse_response(value)?;

		// -- Check
		assert_eq!(response.input_tokens, 1_000_000);
		assert_eq!(response.output_tokens, 7);

		Ok(())
	}

	#[test]
	fn test_adapter_openai_payload_parse_response_tolerates_missing_fields() -> Result<()> {
		// -- Setup & Fixtures
		let value = json!({ "model": "gpt-6-luna" });

		// -- Exec
		let response = parse_response(value)?;

		// -- Check
		assert_eq!(response.model, "gpt-6-luna");
		assert!(response.answers.is_empty());
		assert_eq!(response.input_tokens, 0);
		assert_eq!(response.output_tokens, 0);
		assert_eq!(response.cost, None);

		Ok(())
	}

	#[test]
	fn test_adapter_openai_payload_parse_response_skips_refusal() -> Result<()> {
		// -- Setup & Fixtures
		let value = json!({
			"model": "gpt-6-luna",
			"answers": [
				{ "type": "refusal", "name": "intent" },
				{ "type": "predicate", "name": "q0", "probability": 0.3 }
			]
		});

		// -- Exec
		let response = parse_response(value)?;

		// -- Check
		assert_eq!(response.answers.len(), 1);
		assert!(response.answer("intent").is_none());
		assert!(matches!(response.answer(0), Some(Answer::Noul(_))));

		Ok(())
	}
}

// endregion: --- Tests
