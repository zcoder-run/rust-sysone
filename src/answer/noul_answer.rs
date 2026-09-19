use serde::{Deserialize, Serialize};

/// The `noul` answer kind, holding a single `f64` value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoulAnswer {
	/// The `noul` value.
	pub noul: f64,
}
