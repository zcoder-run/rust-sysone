use crate::QKey;
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

// region:    --- Types

/// The `choice` question kind, selecting one option from candidates.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ChoiceQuestion {
	/// Instructions for answering the question, either a string or structured value.
	pub instructions: Value,

	/// The candidate options and their descriptions, keyed by choice identifier.
	#[serde(default, serialize_with = "serialize_criteria", deserialize_with = "deserialize_criteria")]
	pub criteria: Vec<(QKey, Value)>,
}

// endregion: --- Types

/// Constructors
impl ChoiceQuestion {
	pub fn new(instructions: impl Into<Value>) -> Self {
		Self {
			instructions: instructions.into(),
			criteria: Vec::new(),
		}
	}
}

/// Chainable setters
impl ChoiceQuestion {
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
}

/// Accessors
impl ChoiceQuestion {
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
	let mut map = serializer.serialize_map(Some(criteria.len()))?;
	for (k, v) in criteria {
		map.serialize_entry(&k.wire(), v)?;
	}
	map.end()
}

fn deserialize_criteria<'de, D>(deserializer: D) -> core::result::Result<Vec<(QKey, Value)>, D::Error>
where
	D: Deserializer<'de>,
{
	let opt: Option<Value> = Option::deserialize(deserializer)?;
	match opt {
		Some(Value::Object(map)) => Ok(map.into_iter().map(|(k, v)| (QKey::from(k), v)).collect()),
		Some(Value::Null) | None => Ok(Vec::new()),
		Some(other) => Err(serde::de::Error::custom(format!(
			"expected criteria object, got {other:?}"
		))),
	}
}

// endregion: --- Support

// region:    --- Tests

#[cfg(test)]
mod tests {
	use super::*;
	use serde_json::json;

	#[test]
	fn test_choice_question_serialize() -> crate::Result<()> {
		// -- Setup & Fixtures
		let question = ChoiceQuestion::new("Pick the department")
			.append_criteria("billing", "Billing queries")
			.append_criteria("support", "Technical support");

		// -- Exec
		let val = serde_json::to_value(&question)?;

		// -- Check
		assert_eq!(val["instructions"], "Pick the department");
		assert_eq!(val["criteria"]["billing"], "Billing queries");
		assert_eq!(val["criteria"]["support"], "Technical support");

		Ok(())
	}

	#[test]
	fn test_choice_question_deserialize() -> crate::Result<()> {
		// -- Setup & Fixtures
		let json = json!({
			"instructions": "Which team should handle this?",
			"criteria": {
				"billing": null,
				"orders": { "what": "Order tracking" }
			}
		});

		// -- Exec
		let question: ChoiceQuestion = serde_json::from_value(json)?;

		// -- Check
		assert_eq!(question.instructions(), "Which team should handle this?");
		assert_eq!(question.criteria().len(), 2);

		Ok(())
	}
}

// endregion: --- Tests
