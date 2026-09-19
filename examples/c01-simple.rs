use serde_json::json;
use sysone::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let client = Client::builder().build()?;

	let state = "Hi, I've been trying to connect my Stripe account for 3 days and it keeps failing. I'm losing sales. Please help ASAP.";
	let questions = json!({
		"urgency": {
			"type": "noul",
			"instructions": "Does this message express urgency?"
		}
	});

	let res = client.exec(state, questions).await?;

	println!("{res:#?}");

	Ok(())
}
