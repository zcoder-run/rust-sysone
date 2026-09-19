use crate::{ClientBuilder, Result};
use serde::Serialize;
use serde_json::Value;

// region:    --- Types

#[derive(Debug, Clone)]
pub struct Client {
	pub(crate) reqwest_client: reqwest::Client,
	pub(crate) endpoint: String,
	pub(crate) model: String,
	pub(crate) api_key: Option<String>,
}

// endregion: --- Types

/// Constructors
impl Client {
	pub fn builder() -> ClientBuilder {
		ClientBuilder::default()
	}

	pub fn from_endpoint(endpoint: impl Into<String>) -> Result<Self> {
		Client::builder().with_endpoint(endpoint).build()
	}
}

/// Accessors
impl Client {
	pub fn endpoint(&self) -> &str {
		&self.endpoint
	}

	pub fn model(&self) -> &str {
		&self.model
	}
}

/// Execution
impl Client {
	pub async fn exec(
		&self,
		state: impl Serialize,
		questions: impl Serialize,
	) -> Result<Value> {
		self.exec_with_model(&self.model, state, questions).await
	}

	pub async fn exec_with_model(
		&self,
		model: impl Into<String>,
		state: impl Serialize,
		questions: impl Serialize,
	) -> Result<Value> {
		let api_key = match &self.api_key {
			Some(key) => key.clone(),
			None => std::env::var("TYPESAFE_API_KEY")?,
		};

		let payload = serde_json::json!({
			"state": state,
			"model": model.into(),
			"questions": questions,
		});

		let res = self
			.reqwest_client
			.post(&self.endpoint)
			.header(reqwest::header::CONTENT_TYPE, "application/json")
			.bearer_auth(api_key)
			.json(&payload)
			.send()
			.await?
			.error_for_status()?;

		let body = res.json::<Value>().await?;

		Ok(body)
	}
}

// region:    --- Default

impl Default for Client {
	fn default() -> Self {
		Self {
			reqwest_client: reqwest::Client::new(),
			endpoint: "https://api.typesafe.ai/v1/systemone".to_string(),
			model: "jev-latest".to_string(),
			api_key: None,
		}
	}
}

// endregion: --- Default
