//! Typed response returned by the client.

use crate::{Answer, QKey};

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
