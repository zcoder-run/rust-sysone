//! Ignored live smoke test for the OpenAI decisions adapter.
//!
//! It is skipped by default so `cargo test` stays offline and deterministic.
//!
//! Run it manually with a valid key in the environment:
//!
//! ```bash
//! cargo test --test openai_smoke -- --ignored
//! ```

use sysone::{AdapterKind, Answer, Client, Question, Request};

type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

#[tokio::test]
#[ignore = "live call to the OpenAI decisions endpoint"]
async fn test_openai_smoke_noul_answer() -> Result<()> {
	// -- Setup & Fixtures
	let env_var = AdapterKind::OpenAi.api_key_env_var();
	let key_present = std::env::var(env_var).map(|key| !key.trim().is_empty()).unwrap_or(false);
	if !key_present {
		println!("skipped: {env_var} is not set, so no live decision call was made");
		return Ok(());
	}

	// The key itself is resolved from the environment by the client, never hardcoded here.
	let client = Client::builder()
		.with_adapter_kind(AdapterKind::OpenAi)
		.build()?;

	let request = Request::from_state("I was charged twice for my last order.")
		.append_question("duplicate_charge", Question::noul("Does the message describe a duplicate charge?"));

	// -- Exec
	let res = client.exec(request).await?;

	// -- Check
	assert!(res.input_tokens > 0, "Should have used input tokens");
	assert!(!res.model.is_empty(), "Should have a model name");

	let answer = res.answer("duplicate_charge").ok_or("Should have a duplicate_charge answer")?;
	let noul = match answer {
		Answer::Noul(noul) => noul,
		other => return Err(format!("Should be a noul answer, was {other:?}").into()),
	};
	assert!(
		(0.0..=1.0).contains(&noul.noul),
		"Should be a probability, was {}",
		noul.noul
	);

	Ok(())
}
