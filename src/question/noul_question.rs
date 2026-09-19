use crate::QKey;
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

// region:    --- Types

/// The `noul` question kind, verifying a boolean condition or hypothesis.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct NoulQuestion {
	/// Instructions for answering the question, either a string or structured value.
	pub instructions: Value,

	/// Binary criteria describing true and false conditions.
	#[serde(
		default,
		skip_serializing_if = "Vec::is_empty",
		serialize_with = "serialize_criteria",
		deserialize_with = "deserialize_criteria"
	)]
	pub criteria: Vec<(QKey, Value)>,
}

// endregion: --- Types

/// Constructors
impl NoulQuestion {
	pub fn new(instructions: impl Into<Value>) -> Self {
		Self {
			instructions: instructions.into(),
			criteria: Vec::new(),
		}
	}
}

/// Chainable setters
impl NoulQuestion {
	pub fn with_instructions(mut self, instructions: impl Into<Value>) -> Self {
		self.instructions = instructions.into();
		self
	}

	pub fn with_true(self, criteria: impl Into<Value>) -> Self {
		self.append_criteria(QKey::from("true"), criteria)
	}

	pub fn with_false(self, criteria: impl Into<Value>) -> Self {
		self.append_criteria(QKey::from("false"), criteria)
	}

	pub fn with_true_false(self, true_criteria: impl Into<Value>, false_criteria: impl Into<Value>) -> Self {
		self.with_true(true_criteria).with_false(false_criteria)
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
impl NoulQuestion {
	pub fn instructions(&self) -> &Value {
		&self.instructions
	}

	pub fn criteria(&self) -> &[(QKey, Value)] {
		&self.criteria
	}

	pub fn true_criteria(&self) -> Option<&Value> {
		self.criteria.iter().find(|(k, _)| k.wire() == "true").map(|(_, v)| v)
	}

	pub fn false_criteria(&self) -> Option<&Value> {
		self.criteria.iter().find(|(k, _)| k.wire() == "false").map(|(_, v)| v)
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
		Some(other) => Err(serde::de::Error::custom(format!("expected criteria object, got {other:?}"))),
	}
}

// endregion: --- Support

// region:    --- Tests

#[cfg(test)]
mod tests {
	use super::*;
	use serde_json::json;

	#[test]
	fn test_noul_question_serialize_without_criteria() -> crate::Result<()> {
		// -- Setup & Fixtures
		let question = NoulQuestion::new("Is the invoice paid?");

		// -- Exec
		let val = serde_json::to_value(&question)?;

		// -- Check
		assert_eq!(val["instructions"], "Is the invoice paid?");
		assert!(val.get("criteria").is_none());

		Ok(())
	}

	#[test]
	fn test_noul_question_serialize_with_true_false() -> crate::Result<()> {
		// -- Setup & Fixtures
		let question = NoulQuestion::new("Does message ask for credentials?")
			.with_true_false("Asks for password or PIN", "No sensitive credential requested");

		// -- Exec
		let val = serde_json::to_value(&question)?;

		// -- Check
		assert_eq!(val["instructions"], "Does message ask for credentials?");
		assert_eq!(val["criteria"]["true"], "Asks for password or PIN");
		assert_eq!(val["criteria"]["false"], "No sensitive credential requested");
		assert_eq!(question.true_criteria().map(|v| v.as_str()), Some(Some("Asks for password or PIN")));
		assert_eq!(question.false_criteria().map(|v| v.as_str()), Some(Some("No sensitive credential requested")));

		Ok(())
	}

	#[test]
	fn test_noul_question_deserialize() -> crate::Result<()> {
		// -- Setup & Fixtures
		let json = json!({
			"instructions": "Is this an anomaly?",
			"criteria": {
				"true": "Anomalous reading",
				"false": "Normal reading"
			}
		});

		// -- Exec
		let question: NoulQuestion = serde_json::from_value(json)?;

		// -- Check
		assert_eq!(question.instructions(), "Is this an anomaly?");
		assert_eq!(question.true_criteria().map(|v| v.as_str()), Some(Some("Anomalous reading")));
		assert_eq!(question.false_criteria().map(|v| v.as_str()), Some(Some("Normal reading")));

		Ok(())
	}
}

// endregion: --- Tests
