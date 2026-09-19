use serde_json::json;
use sysone::{Client, Request};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let client = Client::builder().build()?;

	let state = "Hi, I've been trying to connect my Stripe account for
		3 days and it keeps failing. I'm losing sales. Please help ASAP.";

	let req = Request::from_state(state).append_question(
		"q123",
		json!({
				"type": "noul",
				"instructions": "Does this message express urgency?"
		}),
	);

	let res = client.exec(req).await?;

	println!("Model: {}", res.model);
	println!("Input tokens: {}", res.input_tokens);
	println!("Output tokens: {}", res.output_tokens);

	if let Some(cost) = res.cost {
		println!("Cost: ${cost:.6}");
	}

	println!("Answers: {:?}", res.answers);

	Ok(())
}
