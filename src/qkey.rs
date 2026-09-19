use serde::{Deserialize, Deserializer, Serialize, Serializer};

// region:    --- Types

/// The key identifying a question on the request side and its answer on the response side.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum QKey {
	/// An integer-indexed question, wire-encoded as `q<i>`.
	Idx(usize),

	/// A named question, wire-encoded as the name itself.
	Name(String),
}

// endregion: --- Types

/// Accessors
impl QKey {
	/// The string used as the JSON key in the request and response.
	pub fn wire(&self) -> String {
		match self {
			QKey::Idx(i) => format!("q{i}"),
			QKey::Name(s) => s.clone(),
		}
	}
}

// region:    --- Froms

impl From<usize> for QKey {
	fn from(value: usize) -> Self {
		QKey::Idx(value)
	}
}

impl From<&str> for QKey {
	fn from(value: &str) -> Self {
		QKey::Name(value.to_string())
	}
}

impl From<String> for QKey {
	fn from(value: String) -> Self {
		QKey::Name(value)
	}
}

// endregion: --- Froms

// region:    --- Trait Implementations

impl Serialize for QKey {
	fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		serializer.serialize_str(&self.wire())
	}
}

impl<'de> Deserialize<'de> for QKey {
	fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		let s = String::deserialize(deserializer)?;
		if let Some(rest) = s.strip_prefix('q')
			&& let Ok(idx) = rest.parse::<usize>()
		{
			Ok(QKey::Idx(idx))
		} else {
			Ok(QKey::Name(s))
		}
	}
}

// endregion: --- Trait Implementations

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

	use super::*;

	#[test]
	fn test_q_key_wire_idx() -> Result<()> {
		// -- Setup & Fixtures
		let key = QKey::from(12);

		// -- Exec
		let wire = key.wire();

		// -- Check
		assert_eq!(wire, "q12");

		Ok(())
	}

	#[test]
	fn test_q_key_wire_name() -> Result<()> {
		// -- Setup & Fixtures
		let key = QKey::from("intent");

		// -- Exec
		let wire = key.wire();

		// -- Check
		assert_eq!(wire, "intent");

		Ok(())
	}

	#[test]
	fn test_q_key_from_string() -> Result<()> {
		// -- Setup & Fixtures
		let key = QKey::from(String::from("improvements"));

		// -- Check
		assert_eq!(key, QKey::Name("improvements".to_string()));

		Ok(())
	}

	#[test]
	fn test_q_key_serde() -> Result<()> {
		// -- Setup & Fixtures
		let key_idx = QKey::from(5);
		let key_name = QKey::from("intent");

		// -- Exec
		let json_idx = serde_json::to_string(&key_idx)?;
		let json_name = serde_json::to_string(&key_name)?;

		// -- Check
		assert_eq!(json_idx, "\"q5\"");
		assert_eq!(json_name, "\"intent\"");

		let back_idx: QKey = serde_json::from_str(&json_idx)?;
		let back_name: QKey = serde_json::from_str(&json_name)?;
		assert_eq!(back_idx, key_idx);
		assert_eq!(back_name, key_name);

		Ok(())
	}
}

// endregion: --- Tests
