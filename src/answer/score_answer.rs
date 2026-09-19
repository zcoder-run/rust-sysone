use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The `score` answer kind, holding the score, a confidence, the option probabilities, and the legend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreAnswer {
	/// The score value.
	pub score: f64,

	/// The confidence in the score.
	pub confidence: f64,

	/// The probability for each option.
	pub probabilities: HashMap<String, f64>,

	/// The legend mapping each score band to its label.
	pub legend: HashMap<String, String>,
}
