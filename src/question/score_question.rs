use crate::QKey;
use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

// region:    --- Types

/// The `score` question kind, placing a value on a scale or ordinal levels.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ScoreQuestion {
	/// Instructions for answering the question, either a string or structured value.
	pub instructions: Value,

	/// The level criteria, keyed by score level index or band name.
	#[serde(default, serialize_with = "serialize_criteria", deserialize_with = "deserialize_criteria")]
	pub criteria: Vec<(QKey, Value)>,
}

// endregion: --- Types

/// Constructors
impl ScoreQuestion {
	pub fn new(instructions: impl Into<Value>) -> Self {
		Self {
			instructions: instructions.into(),
			criteria: Vec::new(),
		}
	}
}

/// Chainable setters
impl ScoreQuestion {
	pub fn with_instructions(mut self, instructions: impl Into<Value>) -> Self {
		self.instructions = instructions.into();
		self
	}

	pub fn append_criteria(mut self, key: impl Into<QKey>, criterion: impl Into<Value>) -> Self {
		let key = key.into();
		let criterion = criterion.into();
		if let Some(entry) = self.criteria.iter_mut().find(|(k, _)| *k == key) {
			entry.1 = criterion;
		} else {
			self.criteria.push((key, criterion));
		}
		self
	}

	pub fn extend_criteria<K, V>(mut self, iter: impl IntoIterator<Item = (K, V)>) -> Self
	where
		K: Into<QKey>,
		V: Into<Value>,
	{
		for (k, v) in iter {
			self = self.append_criteria(k, v);
		}
		self
	}

	pub fn append_level(self, criterion: impl Into<Value>) -> Self {
		let idx = self.criteria.len();
		self.append_criteria(QKey::Idx(idx), criterion)
	}

	pub fn extend_levels<V>(mut self, iter: impl IntoIterator<Item = V>) -> Self
	where
		V: Into<Value>,
	{
		for criterion in iter {
			self = self.append_level(criterion);
		}
		self
	}
}

/// Accessors
impl ScoreQuestion {
	pub fn instructions(&self) -> &Value {
		&self.instructions
	}

	pub fn criteria(&self) -> &[(QKey, Value)] {
		&self.criteria
	}
}

// region:    --- Support

fn serialize_criteria<S>(criteria: &[(QKey, Value)], serializer: S) -> core::result::Result<S::Ok, S::Error>
where
	S: Serializer,
{
	let all_idx = criteria.iter().all(|(k, _)| matches!(k, QKey::Idx(_)));
	if all_idx {
		let mut seq = serializer.serialize_seq(Some(criteria.len()))?;
		for (_, v) in criteria {
			seq.serialize_element(v)?;
		}
		seq.end()
	} else {
		let mut map = serializer.serialize_map(Some(criteria.len()))?;
		for (k, v) in criteria {
			map.serialize_entry(&k.wire(), v)?;
		}
		map.end()
	}
}

fn deserialize_criteria<'de, D>(deserializer: D) -> core::result::Result<Vec<(QKey, Value)>, D::Error>
where
	D: Deserializer<'de>,
{
	let opt: Option<Value> = Option::deserialize(deserializer)?;
	match opt {
		Some(Value::Array(arr)) => {
			Ok(arr.into_iter().enumerate().map(|(i, v)| (QKey::Idx(i), v)).collect())
		}
		Some(Value::Object(map)) => {
			Ok(map.into_iter().map(|(k, v)| (QKey::from(k), v)).collect())
		}
		Some(Value::Null) | None => Ok(Vec::new()),
		Some(other) => Err(serde::de::Error::custom(format!("expected criteria array or object, got {other:?}"))),
	}
}

// endregion: --- Support

// region:    --- Tests

#[cfg(test)]
mod tests {
	use super::*;
	use serde_json::json;

	#[test]
	fn test_score_question_serialize_array() -> crate::Result<()> {
		// -- Setup & Fixtures
		let question = ScoreQuestion::new("Rate severity")
			.append_level("Low")
			.append_level("Medium")
			.append_level("High");

		// -- Exec
		let val = serde_json::to_value(&question)?;

		// -- Check
		assert_eq!(val["instructions"], "Rate severity");
		assert_eq!(val["criteria"], json!(["Low", "Medium", "High"]));

		Ok(())
	}

	#[test]
	fn test_score_question_deserialize_array() -> crate::Result<()> {
		// -- Setup & Fixtures
		let json = json!({
			"instructions": "How large is the amount?",
			"criteria": [
				"Under $1,000",
				"$1,000 to $10,000",
				"Over $10,000"
			]
		});

		// -- Exec
		let question: ScoreQuestion = serde_json::from_value(json)?;

		// -- Check
		assert_eq!(question.instructions(), "How large is the amount?");
		assert_eq!(question.criteria().len(), 3);
		assert_eq!(question.criteria()[0].0, QKey::Idx(0));
		assert_eq!(question.criteria()[0].1, "Under $1,000");

		Ok(())
	}
}

// endregion: --- Tests
