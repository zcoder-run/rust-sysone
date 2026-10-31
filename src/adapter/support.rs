use crate::QKey;

/// Map a wire key back into a `QKey`, mapping `q<i>` to `QKey::Idx(i)` and anything else to `QKey::Name`.
pub(crate) fn parse_q_key(key: &str) -> QKey {
	key.strip_prefix('q')
		.and_then(|idx| idx.parse::<usize>().ok())
		.map(QKey::Idx)
		.unwrap_or_else(|| QKey::Name(key.to_string()))
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

	use super::*;

	#[test]
	fn test_adapter_support_parse_q_key_idx() -> Result<()> {
		// -- Exec
		let key = parse_q_key("q3");

		// -- Check
		assert_eq!(key, QKey::Idx(3));

		Ok(())
	}

	#[test]
	fn test_adapter_support_parse_q_key_name() -> Result<()> {
		// -- Exec
		let key = parse_q_key("intent");

		// -- Check
		assert_eq!(key, QKey::Name("intent".to_string()));

		Ok(())
	}

	#[test]
	fn test_adapter_support_parse_q_key_prefixed_name() -> Result<()> {
		// -- Exec
		let key = parse_q_key("qx");

		// -- Check
		assert_eq!(key, QKey::Name("qx".to_string()));

		Ok(())
	}
}

// endregion: --- Tests
