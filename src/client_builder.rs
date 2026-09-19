use crate::{Client, Result};

#[derive(Default)]
pub struct ClientBuilder {
	endpoint: Option<String>,
	model: Option<String>,
	api_key: Option<String>,
	reqwest_client: Option<reqwest::Client>,
}

/// Chainable setters
impl ClientBuilder {
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

	pub fn with_reqwest_client(mut self, client: reqwest::Client) -> Self {
		self.reqwest_client = Some(client);
		self
	}

	pub fn build(self) -> Result<Client> {
		let reqwest_client = match self.reqwest_client {
			Some(client) => client,
			None => reqwest::Client::builder().build()?,
		};
		let endpoint = self
			.endpoint
			.unwrap_or_else(|| "https://api.typesafe.ai/v1/systemone".to_string());
		let model = self
			.model
			.unwrap_or_else(|| "jev-latest".to_string());

		Ok(Client {
			reqwest_client,
			endpoint,
			model,
			api_key: self.api_key,
		})
	}
}
