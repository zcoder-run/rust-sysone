use std::fmt;
use std::str::FromStr;

use crate::{Error, Result};

// region:    --- Types

/// The provider adapter identity, and the single place holding the per provider defaults.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum AdapterKind {
	/// TypeSafe System One (jev).
	#[default]
	Jev,
}

// endregion: --- Types

/// Accessors
impl AdapterKind {
	/// The stable lowercase name of this adapter kind.
	pub fn as_str(self) -> &'static str {
		match self {
			AdapterKind::Jev => "jev",
		}
	}

	/// The default endpoint used when the client builder does not set one.
	pub fn default_endpoint(self) -> &'static str {
		match self {
			AdapterKind::Jev => "https://api.typesafe.ai/v1/systemone",
		}
	}

	/// The default model used when the client builder does not set one.
	pub fn default_model(self) -> &'static str {
		match self {
			AdapterKind::Jev => "jev-latest",
		}
	}

	/// The environment variable read for the API key.
	pub fn api_key_env_var(self) -> &'static str {
		match self {
			AdapterKind::Jev => "TYPESAFE_API_KEY",
		}
	}
}

impl fmt::Display for AdapterKind {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(self.as_str())
	}
}

impl FromStr for AdapterKind {
	type Err = Error;

	fn from_str(input: &str) -> Result<Self> {
		match input.trim().to_lowercase().as_str() {
			"jev" => Ok(AdapterKind::Jev),
			_ => Err(Error::UnknownAdapterKind(input.to_string())),
		}
	}
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_adapter_kind_jev_values() {
		// -- Exec
		let kind = AdapterKind::Jev;

		// -- Check
		assert_eq!(kind.as_str(), "jev");
		assert_eq!(kind.default_endpoint(), "https://api.typesafe.ai/v1/systemone");
		assert_eq!(kind.default_model(), "jev-latest");
		assert_eq!(kind.api_key_env_var(), "TYPESAFE_API_KEY");
		assert_eq!(AdapterKind::default(), AdapterKind::Jev);
	}

	#[test]
	fn test_adapter_kind_display() {
		// -- Exec
		let display = AdapterKind::Jev.to_string();

		// -- Check
		assert_eq!(display, "jev");
	}

	#[test]
	fn test_adapter_kind_from_str() -> Result<()> {
		// -- Exec & Check
		assert_eq!("jev".parse::<AdapterKind>()?, AdapterKind::Jev);
		assert_eq!(" JEV ".parse::<AdapterKind>()?, AdapterKind::Jev);

		let unknown_err = "openai".parse::<AdapterKind>().err();
		assert!(matches!(unknown_err, Some(Error::UnknownAdapterKind(name)) if name == "openai"));

		Ok(())
	}
}

// endregion: --- Tests
