use sysone::{ChoiceQuestion, Client, Request};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let client = Client::builder().build()?;

	let state = "My running shoes arrived in the wrong size. Can I swap them for a size 10?";

	let question = ChoiceQuestion::new("Which team should handle this?")
		.append_criteria("returns", "Exchanges, refunds, wrong or damaged items")
		.append_criteria("shipping", "Delivery status, delays, lost packages")
		.append_criteria("billing", "Charges, invoices, payment problems");

	let req = Request::from_state(state).append_question("department", question);

	let res = client.exec(req).await?;

	println!("Model: {}", res.model);
	println!("Input tokens: {}", res.input_tokens);
	println!("Output tokens: {}", res.output_tokens);

	if let Some(cost) = res.cost {
		println!("Cost: ${cost:.6}");
	}

	println!("Answers: {:?}", res.answers);

	if let Some(answer) = res.answer("department") {
		println!("Department answer: {answer:?}");
	}

	Ok(())
}
