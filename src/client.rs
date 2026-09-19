use crate::{ClientBuilder, Error, Request, Response, Result};
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

	pub fn api_key(&self) -> Option<&str> {
		self.api_key.as_deref()
	}
}

/// Auth / Resolution
impl Client {
	fn resolve_api_key(&self) -> Result<String> {
		if let Some(key) = &self.api_key
			&& !key.is_empty()
		{
			return Ok(key.clone());
		}

		if let Ok(key) = std::env::var("TYPESAFE_API_KEY")
			&& !key.is_empty()
		{
			return Ok(key);
		}

		Err(Error::AuthNotPresent)
	}
}

/// Execution
impl Client {
	pub async fn exec(&self, request: Request) -> Result<Response> {
		self.exec_with_model(&self.model, request).await
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

// region:    --- Support

impl Client {
	async fn exec_with_model(&self, model: impl Into<String>, request: Request) -> Result<Response> {
		let api_key = self.resolve_api_key()?;
		let Request { state, questions } = request;

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
			.await?;

		let status = res.status();
		if !status.is_success() {
			let body = res.text().await.unwrap_or_default();
			return Err(Error::ResponseError { status, body });
		}

		let body = res.json::<Value>().await?;
		let response = Response::from_value(body)?;

		Ok(response)
	}
}

// endregion: --- Support
