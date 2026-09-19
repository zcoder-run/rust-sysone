use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// The `choice` answer kind, holding the selected choice, a confidence, and the option probabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChoiceAnswer {
	/// The selected choice label.
	pub choice: String,

	/// The confidence in the selected choice.
	pub confidence: f64,

	/// The probability for each option.
	pub probabilities: HashMap<String, f64>,
}
