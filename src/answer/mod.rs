//! Typed answer variants for the System One response.

use serde::{Deserialize, Serialize};

// region:    --- Modules

mod choice_answer;
mod noul_answer;
mod score_answer;

pub use choice_answer::ChoiceAnswer;
pub use noul_answer::NoulAnswer;
pub use score_answer::ScoreAnswer;

// endregion: --- Modules

/// The typed answer for a question.
///
/// Serialized and deserialized with an internal `type` tag (`noul`, `choice`, or `score`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Answer {
	/// The `noul` answer kind, holding a single `f64` value.
	Noul(NoulAnswer),

	/// The `choice` answer kind, holding the selected choice, a confidence, and the option probabilities.
	Choice(ChoiceAnswer),

	/// The `score` answer kind, holding the score, a confidence, the option probabilities, and the legend.
	Score(ScoreAnswer),
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	use super::*;
	use serde_json::json;

	#[test]
	fn test_answer_noul_deserialize() -> crate::Result<()> {
		// -- Setup & Fixtures
		let value = json!({ "type": "noul", "noul": 0.42 });

		// -- Exec
		let answer: Answer = serde_json::from_value(value)?;

		// -- Check
		assert!(matches!(answer, Answer::Noul(_)));
		if let Answer::Noul(noul) = &answer {
			assert_eq!(noul.noul, 0.42);
		}

		Ok(())
	}

	#[test]
	fn test_answer_choice_deserialize() -> crate::Result<()> {
		// -- Setup & Fixtures
		let value = json!({
			"type": "choice",
			"choice": "a",
			"confidence": 0.9,
			"probabilities": { "a": 0.9, "b": 0.1 }
		});

		// -- Exec
		let answer: Answer = serde_json::from_value(value)?;

		// -- Check
		assert!(matches!(answer, Answer::Choice(_)));
		if let Answer::Choice(choice) = &answer {
			assert_eq!(choice.choice, "a");
			assert_eq!(choice.probabilities.len(), 2);
		}

		Ok(())
	}

	#[test]
	fn test_answer_score_deserialize() -> crate::Result<()> {
		// -- Setup & Fixtures
		let value = json!({
			"type": "score",
			"score": 0.8,
			"confidence": 0.9,
			"probabilities": { "a": 0.8, "b": 0.2 },
			"legend": { "a": "high" }
		});

		// -- Exec
		let answer: Answer = serde_json::from_value(value)?;

		// -- Check
		assert!(matches!(answer, Answer::Score(_)));
		if let Answer::Score(score) = &answer {
			assert_eq!(score.score, 0.8);
			assert_eq!(score.legend.len(), 1);
		}

		Ok(())
	}
}

// endregion: --- Tests
