//! Typed question variants for the System One request.

use serde::{Deserialize, Serialize};
use serde_json::Value;

// region:    --- Modules

mod choice_question;
mod noul_question;
mod score_question;

pub use choice_question::ChoiceQuestion;
pub use noul_question::NoulQuestion;
pub use score_question::ScoreQuestion;

// endregion: --- Modules

// region:    --- Types

/// The typed question for a request.
///
/// Serialized and deserialized with an internal `type` tag (`noul`, `choice`, or `score`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Question {
	/// The `noul` question kind, verifying a boolean condition or hypothesis.
	Noul(NoulQuestion),

	/// The `choice` question kind, selecting one option from candidates.
	Choice(ChoiceQuestion),

	/// The `score` question kind, placing a value on a scale or ordinal levels.
	Score(ScoreQuestion),
}

// endregion: --- Types

/// Constructors
impl Question {
	pub fn noul(instructions: impl Into<Value>) -> Self {
		Question::Noul(NoulQuestion::new(instructions))
	}

	pub fn choice(instructions: impl Into<Value>) -> Self {
		Question::Choice(ChoiceQuestion::new(instructions))
	}

	pub fn score(instructions: impl Into<Value>) -> Self {
		Question::Score(ScoreQuestion::new(instructions))
	}
}

// region:    --- Froms

impl From<ChoiceQuestion> for Question {
	fn from(value: ChoiceQuestion) -> Self {
		Question::Choice(value)
	}
}

impl From<ScoreQuestion> for Question {
	fn from(value: ScoreQuestion) -> Self {
		Question::Score(value)
	}
}

impl From<NoulQuestion> for Question {
	fn from(value: NoulQuestion) -> Self {
		Question::Noul(value)
	}
}

impl From<Value> for Question {
	fn from(value: Value) -> Self {
		match serde_json::from_value(value.clone()) {
			Ok(question) => question,
			Err(_) => Question::noul(value),
		}
	}
}

// endregion: --- Froms

// region:    --- Tests

#[cfg(test)]
mod tests {
	use super::*;
	use serde_json::json;

	#[test]
	fn test_question_noul_serde() -> crate::Result<()> {
		// -- Setup & Fixtures
		let question = Question::noul("Is the document valid?");

		// -- Exec
		let val = serde_json::to_value(&question)?;

		// -- Check
		assert_eq!(val["type"], "noul");
		assert_eq!(val["instructions"], "Is the document valid?");

		let de: Question = serde_json::from_value(val)?;
		assert_eq!(de, question);

		Ok(())
	}

	#[test]
	fn test_question_choice_serde() -> crate::Result<()> {
		// -- Setup & Fixtures
		let question = Question::choice("Select category")
			.into_choice()
			.map(|c| c.append_criteria("A", "Alpha").append_criteria("B", "Beta"))
			.map(Question::Choice)
			.unwrap_or_else(|| Question::choice("fallback"));

		// -- Exec
		let val = serde_json::to_value(&question)?;

		// -- Check
		assert_eq!(val["type"], "choice");
		assert_eq!(val["instructions"], "Select category");
		assert_eq!(val["criteria"]["A"], "Alpha");

		let de: Question = serde_json::from_value(val)?;
		assert_eq!(de, question);

		Ok(())
	}

	#[test]
	fn test_question_score_serde() -> crate::Result<()> {
		// -- Setup & Fixtures
		let score_q = ScoreQuestion::new("Rate priority").append_level("P0").append_level("P1");
		let question = Question::from(score_q);

		// -- Exec
		let val = serde_json::to_value(&question)?;

		// -- Check
		assert_eq!(val["type"], "score");
		assert_eq!(val["criteria"], json!(["P0", "P1"]));

		let de: Question = serde_json::from_value(val)?;
		assert_eq!(de, question);

		Ok(())
	}

	#[test]
	fn test_question_from_value() -> crate::Result<()> {
		// -- Exec & Check (noul json)
		let q_noul: Question = json!({
			"type": "noul",
			"instructions": "Is it active?"
		})
		.into();
		assert!(matches!(q_noul, Question::Noul(_)));

		// -- Exec & Check (fallback)
		let q_fallback: Question = json!("Just a text instruction").into();
		assert!(matches!(q_fallback, Question::Noul(_)));

		Ok(())
	}

	impl Question {
		fn into_choice(self) -> Option<ChoiceQuestion> {
			match self {
				Question::Choice(c) => Some(c),
				_ => None,
			}
		}
	}
}

// endregion: --- Tests
