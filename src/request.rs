use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};
use serde_json::Value;

use crate::question::Question;
use crate::QKey;

// region:    --- Types

/// The execution request, holding the current `state` and the `questions` to be answered.
#[derive(Debug, Clone, Default, Serialize)]
pub struct Request {
	pub state: Value,
	#[serde(serialize_with = "serialize_questions")]
	pub questions: Vec<(QKey, Question)>,
}

// endregion: --- Types

/// Constructors
impl Request {
	/// Create a new request for the given state, with no question yet.
	///
	/// Note: This is an alias of `Request::from_state`.
	pub fn new(state: impl Into<Value>) -> Self {
		Self::from_state(state)
	}

	/// Create a new request for the given state, with no question yet.
	pub fn from_state(state: impl Into<Value>) -> Self {
		Self {
			state: state.into(),
			questions: Vec::new(),
		}
	}

	/// Create a new request with the given state and questions.
	pub fn from_state_questions<K, Q>(state: impl Into<Value>, questions: impl IntoIterator<Item = (K, Q)>) -> Self
	where
		K: Into<QKey>,
		Q: Into<Question>,
	{
		Self::from_state(state).extend_questions(questions)
	}
}

/// Accessors
impl Request {
	pub fn state(&self) -> &Value {
		&self.state
	}

	pub fn questions(&self) -> &[(QKey, Question)] {
		&self.questions
	}

	/// Look up a question by key, returning `None` when no question matches.
	pub fn question(&self, key: impl Into<QKey>) -> Option<&Question> {
		let key = key.into();
		self.questions
			.iter()
			.find_map(|(k, q)| (k == &key || k.wire() == key.wire()).then_some(q))
	}
}

/// Fluid Setters
impl Request {
	/// Set (replace) the state.
	pub fn with_state(mut self, state: impl Into<Value>) -> Self {
		self.state = state.into();
		self
	}

	/// Set (replace) all of the questions.
	pub fn with_questions<K, Q>(mut self, questions: impl IntoIterator<Item = (K, Q)>) -> Self
	where
		K: Into<QKey>,
		Q: Into<Question>,
	{
		self.questions.clear();
		self.extend_questions(questions)
	}

	/// Add (or replace) one question, keyed by `key` (name or integer index).
	///
	/// Note: An existing question with the same `key` gets overridden.
	pub fn append_question(mut self, key: impl Into<QKey>, question: impl Into<Question>) -> Self {
		let key = key.into();
		let question = question.into();
		if let Some(entry) = self.questions.iter_mut().find(|(k, _)| *k == key || k.wire() == key.wire()) {
			entry.0 = key;
			entry.1 = question;
		} else {
			self.questions.push((key, question));
		}
		self
	}

	/// Add (or replace) multiple questions, each one keyed by its own key (name or integer index).
	pub fn extend_questions<K, Q>(mut self, questions: impl IntoIterator<Item = (K, Q)>) -> Self
	where
		K: Into<QKey>,
		Q: Into<Question>,
	{
		for (key, question) in questions {
			self = self.append_question(key, question);
		}
		self
	}
}

// region:    --- Support

fn serialize_questions<S>(questions: &[(QKey, Question)], serializer: S) -> core::result::Result<S::Ok, S::Error>
where
	S: Serializer,
{
	let mut map = serializer.serialize_map(Some(questions.len()))?;
	for (key, question) in questions {
		map.serialize_entry(&key.wire(), question)?;
	}
	map.end()
}

// endregion: --- Support

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

	use super::*;

	#[test]
	fn test_request_append_question_named_and_indexed() -> Result<()> {
		// -- Setup & Fixtures
		let req = Request::from_state("state")
			.append_question("intent", Question::noul("Check intent"))
			.append_question(0, Question::choice("Pick option"))
			.extend_questions([
				(QKey::from(1), Question::score("Rate score")),
				(QKey::from("summary"), Question::noul("Summarize")),
			]);

		// -- Exec
		let val = serde_json::to_value(&req)?;

		// -- Check
		assert_eq!(val["state"], "state");
		let questions = val.get("questions").and_then(Value::as_object).ok_or("expected questions object")?;
		assert!(questions.contains_key("intent"));
		assert!(questions.contains_key("q0"));
		assert!(questions.contains_key("q1"));
		assert!(questions.contains_key("summary"));
		assert_eq!(questions["intent"]["type"], "noul");
		assert_eq!(questions["q0"]["type"], "choice");
		assert_eq!(questions["q1"]["type"], "score");
		assert_eq!(questions["summary"]["type"], "noul");

		assert_eq!(req.questions().len(), 4);
		assert!(matches!(req.question("intent"), Some(Question::Noul(_))));
		assert!(matches!(req.question(0), Some(Question::Choice(_))));
		assert!(matches!(req.question(1), Some(Question::Score(_))));
		assert!(matches!(req.question("summary"), Some(Question::Noul(_))));
		assert!(req.question("missing").is_none());

		Ok(())
	}

	#[test]
	fn test_request_append_question_override() -> Result<()> {
		// -- Setup & Fixtures
		let req = Request::from_state("state")
			.append_question("q0", Question::noul("Original"))
			.append_question("q0", Question::noul("Updated"));

		// -- Check
		assert_eq!(req.questions().len(), 1);
		let q = req.question("q0").ok_or("expected question")?;
		if let Question::Noul(noul) = q {
			assert_eq!(noul.instructions(), "Updated");
		} else {
			return Err("expected noul question".into());
		}

		Ok(())
	}

	#[test]
	fn test_request_from_state_questions_and_with_questions() -> Result<()> {
		// -- Setup & Fixtures
		let choice_q = crate::ChoiceQuestion::new("Choose team").append_criteria("ops", "Operations");
		let noul_q = crate::NoulQuestion::new("Is urgent?").with_true("Urgent action required");
		let score_q = crate::ScoreQuestion::new("Severity").append_level("Low").append_level("High");

		let req = Request::from_state_questions("test_state", [
			("choice", Question::from(choice_q)),
			("noul", Question::from(noul_q)),
		]);

		assert_eq!(req.questions().len(), 2);
		assert!(matches!(req.question("choice"), Some(Question::Choice(_))));
		assert!(matches!(req.question("noul"), Some(Question::Noul(_))));

		let req2 = req.with_questions([(0, score_q)]);
		assert_eq!(req2.questions().len(), 1);
		assert!(matches!(req2.question(0), Some(Question::Score(_))));

		Ok(())
	}
}

// endregion: --- Tests
