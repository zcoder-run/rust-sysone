use crate::adapter::AdapterCtx;
use crate::{AdapterKind, ClientBuilder, Error, Request, Response, Result};
use crate::webc::Webc;

// region:    --- Types

#[derive(Debug, Clone)]
pub struct Client {
	pub(crate) webc: Webc,
	pub(crate) endpoint: String,
	pub(crate) model: String,
	pub(crate) api_key: Option<String>,
	pub(crate) adapter_kind: AdapterKind,
	pub(crate) extra_headers: Vec<(String, String)>,
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

	pub fn adapter_kind(&self) -> AdapterKind {
		self.adapter_kind
	}

	pub fn extra_headers(&self) -> &[(String, String)] {
		&self.extra_headers
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

		if let Ok(key) = std::env::var(self.adapter_kind.api_key_env_var())
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
		let api_key = self.resolve_api_key()?;
		let ctx = self.adapter_ctx(&api_key);
		crate::adapter::exec(self.adapter_kind, ctx, &request).await
	}
}

// region:    --- Default

impl Default for Client {
	fn default() -> Self {
		let adapter_kind = AdapterKind::default();
		Self {
			webc: Webc::default(),
			endpoint: adapter_kind.default_endpoint().to_string(),
			model: adapter_kind.default_model().to_string(),
			api_key: None,
			adapter_kind,
			extra_headers: Vec::new(),
		}
	}
}

// endregion: --- Default

// region:    --- Support

impl Client {
	fn adapter_ctx<'a>(&'a self, api_key: &'a str) -> AdapterCtx<'a> {
		AdapterCtx {
			webc: &self.webc,
			endpoint: &self.endpoint,
			model: &self.model,
			api_key,
			extra_headers: &self.extra_headers,
		}
	}
}

// endregion: --- Support
