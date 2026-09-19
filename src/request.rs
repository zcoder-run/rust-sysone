use serde::Serialize;
use serde_json::Value;

// region:    --- Types

/// The execution request, holding the current `state` and the `questions` to be answered.
#[derive(Debug, Clone, Default, Serialize)]
pub struct Request {
	pub state: Value,
	pub questions: Value,
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
			questions: Value::Null,
		}
	}

	/// Create a new request with the given state and questions.
	pub fn from_state_questions(state: impl Into<Value>, questions: impl Into<Value>) -> Self {
		Self {
			state: state.into(),
			questions: questions.into(),
		}
	}
}

/// Accessors
impl Request {
	pub fn state(&self) -> &Value {
		&self.state
	}

	pub fn questions(&self) -> &Value {
		&self.questions
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
	///
	/// Note: The questions are a dictionary (JSON object), so this should be a map of
	///       question key to question body, e.g. `json!({ "intent": ".." })`.
	pub fn with_questions(mut self, questions: impl Into<Value>) -> Self {
		self.questions = questions.into();
		self
	}

	/// Add (or replace) one question, keyed by `key`.
	///
	/// Note: The questions are always kept as a dictionary (JSON object), so an existing
	///       question with the same `key` gets overridden, and a non dictionary questions
	///       value (set via `with_questions`) gets replaced by a new empty dictionary.
	pub fn append_question(mut self, key: impl Into<String>, body: impl Into<Value>) -> Self {
		let mut questions = match self.questions.take() {
			Value::Object(questions) => questions,
			_ => serde_json::Map::new(),
		};

		questions.insert(key.into(), body.into());
		self.questions = Value::Object(questions);

		self
	}

	/// Add (or replace) multiple questions, each one keyed by its own key.
	pub fn extend_questions<K, V>(mut self, questions: impl IntoIterator<Item = (K, V)>) -> Self
	where
		K: Into<String>,
		V: Into<Value>,
	{
		for (key, body) in questions {
			self = self.append_question(key, body);
		}
		self
	}
}
