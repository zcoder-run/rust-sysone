use crate::{AdapterKind, Client, Error, Result};
use crate::webc::Webc;

#[derive(Default)]
pub struct ClientBuilder {
	adapter_kind: Option<AdapterKind>,
	endpoint: Option<String>,
	model: Option<String>,
	api_key: Option<String>,
	extra_headers: Vec<(String, String)>,
	reqwest_client: Option<reqwest::Client>,
}

/// Chainable setters
impl ClientBuilder {
	pub fn with_adapter_kind(mut self, kind: AdapterKind) -> Self {
		self.adapter_kind = Some(kind);
		self
	}

	pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
		self.endpoint = Some(endpoint.into());
		self
	}

	pub fn with_model(mut self, model: impl Into<String>) -> Self {
		self.model = Some(model.into());
		self
	}

	pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
		self.api_key = Some(api_key.into());
		self
	}

	pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
		self.extra_headers.push((name.into(), value.into()));
		self
	}

	pub fn with_reqwest_client(mut self, client: reqwest::Client) -> Self {
		self.reqwest_client = Some(client);
		self
	}

	pub fn build(self) -> Result<Client> {
		let adapter_kind = self.adapter_kind.unwrap_or_default();
		let reqwest_client = match self.reqwest_client {
			Some(client) => client,
			None => reqwest::Client::builder().build()?,
		};
		let endpoint = self
			.endpoint
			.unwrap_or_else(|| adapter_kind.default_endpoint().to_string());
		let model = self
			.model
			.unwrap_or_else(|| adapter_kind.default_model().to_string());

		if endpoint.trim().is_empty() || reqwest::Url::parse(&endpoint).is_err() {
			return Err(Error::InvalidEndpoint(endpoint));
		}

		Ok(Client {
			webc: Webc::from_reqwest(reqwest_client),
			endpoint,
			model,
			api_key: self.api_key,
			adapter_kind,
			extra_headers: self.extra_headers,
		})
	}
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_client_builder_build_default_jev() -> Result<()> {
		// -- Exec
		let client = Client::builder().build()?;

		// -- Check
		assert_eq!(client.adapter_kind(), AdapterKind::Jev);
		assert_eq!(client.endpoint(), AdapterKind::Jev.default_endpoint());
		assert_eq!(client.model(), AdapterKind::Jev.default_model());
		assert!(client.extra_headers().is_empty());

		Ok(())
	}

	#[test]
	fn test_client_builder_build_with_header() -> Result<()> {
		// -- Exec
		let client = Client::builder()
			.with_header("x-one", "one")
			.with_header("x-two", "two")
			.build()?;

		// -- Check
		let expected: Vec<(String, String)> = vec![
			("x-one".to_string(), "one".to_string()),
			("x-two".to_string(), "two".to_string()),
		];
		assert_eq!(client.extra_headers(), expected.as_slice());

		Ok(())
	}

	#[test]
	fn test_client_builder_build_with_endpoint_override() -> Result<()> {
		// -- Exec
		let client = Client::builder()
			.with_endpoint("https://example.test/v1")
			.with_model("custom-model")
			.build()?;

		// -- Check
		assert_eq!(client.adapter_kind(), AdapterKind::Jev);
		assert_eq!(client.endpoint(), "https://example.test/v1");
		assert_eq!(client.model(), "custom-model");

		Ok(())
	}
}

// endregion: --- Tests
