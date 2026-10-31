use crate::{Error, Result};
use serde_json::Value;

// region:    --- Types

/// Internal HTTP transport, a thin wrapper over `reqwest::Client`.
#[derive(Debug, Clone)]
pub(crate) struct Webc {
	inner: reqwest::Client,
}

// endregion: --- Types

/// Constructors
impl Webc {
	pub(crate) fn from_reqwest(inner: reqwest::Client) -> Self {
		Self { inner }
	}
}

/// Execution
impl Webc {
	pub(crate) async fn post_json(
		&self,
		url: &str,
		api_key: &str,
		extra_headers: &[(String, String)],
		payload: &Value,
	) -> Result<Value> {
		let mut builder = self
			.inner
			.post(url)
			.header(reqwest::header::CONTENT_TYPE, "application/json")
			.bearer_auth(api_key);

		for (name, value) in extra_headers {
			builder = builder.header(name.as_str(), value.as_str());
		}

		let res = builder.json(payload).send().await?;

		let status = res.status();
		if !status.is_success() {
			let body = res.text().await.unwrap_or_default();
			return Err(Error::ResponseError { status, body });
		}

		let body = res.json::<Value>().await?;

		Ok(body)
	}
}

// region:    --- Default

impl Default for Webc {
	fn default() -> Self {
		Self {
			inner: reqwest::Client::new(),
		}
	}
}

// endregion: --- Default
