# TypeSafe AI System One Rust Client Library (unofficial)

By the same author of the [genai](https://crates.io/crates/genai) crate (Jeremy Chone).

- Very early release `0.0.x`
- Extremely basic functionality / API surface for now
- Feel free to cherry pick what you need for now.

Part of the [zcoder.run](https://zcoder.run) Rust libraries, and will probably be used in the zcoder harness (still in the building).

## Usage

```rust
use serde_json::json;
use sysone::{Client, Request};

// Set TYPESAFE_API_KEY in the environment or configure it on the builder:
// Client::builder().with_api_key("...").build()?;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::default();

    let req = Request::from_state(json!({ "code": "fn main() {}" }))
        .append_question("intent", json!({ "type": "noul", "instructions": "What does this code do?" }))
        .append_question(0, json!({ "type": "choice", "choices": ["ok", "fix"] }));

    let res = client.exec(req).await?;

    println!("Model: {}", res.model);
    println!("Tokens: {} in / {} out", res.input_tokens, res.output_tokens);
    if let Some(cost) = res.cost {
        println!("Cost: ${cost:.6}");
    }

    if let Some(answer) = res.answer("intent") {
        println!("Intent answer: {answer:?}");
    }

    Ok(())
}
```

### Request

`Request` follows the fluid API style, every value is an `impl Into<serde_json::Value>`, and every question is keyed with `impl Into<QKey>`. Questions are stored as a JSON object of question key to question body, where a question with an existing key overrides the previous one.

Constructors:

- `Request::from_state(state)`: state only, no questions yet (`Request::new` is an alias of it).
- `Request::from_state_questions(state, questions)`: state and questions at once.

```rust
use serde_json::json;
use sysone::Request;

// Replace the full question set (a dictionary of question key to question body)
let req = Request::from_state("current state")
    .with_questions(json!({ "intent": "What does this code do?" }));

// Or build it up one question at a time using named or indexed keys
let req = Request::default()
    .with_state(json!({ "step": 1 }))
    .append_question("intent", json!({ "type": "noul" }))
    .append_question(0, json!({ "type": "choice" }))
    .extend_questions([(1, json!({ "type": "score" })), ("summary", json!({ "type": "noul" }))]);
```

### Question Keys (QKey)

Questions can be keyed either by name or by index:

- `QKey::Name("intent".to_string())`: wire key `"intent"`.
- `QKey::Idx(0)`: wire key `"q0"`.

`append_question` and `extend_questions` accept `impl Into<QKey>`, so `&str`, `String`, and `usize` work directly.

### Response

`Client::exec` returns a typed `Response`:

- `res.model`: model identifier used for the call.
- `res.input_tokens`: input tokens consumed.
- `res.output_tokens`: output tokens consumed.
- `res.cost`: client-computed cost in USD (`Some(f64)` for supported models, `None` otherwise).
- `res.answers`: vector of `(QKey, Answer)` pairs.
- `res.answer(key)`: lookup helper accepting `impl Into<QKey>` (`&str`, `String`, or `usize`).

### Typed Answers

The `Answer` enum provides typed access to the three response kinds:

- `Answer::Noul(NoulAnswer)`: contains `noul: f64`.
- `Answer::Choice(ChoiceAnswer)`: contains `choice: String`, `confidence: f64`, and `probabilities: HashMap<String, f64>`.
- `Answer::Score(ScoreAnswer)`: contains `score: f64`, `confidence: f64`, `probabilities: HashMap<String, f64>`, and `legend: HashMap<String, String>`.

### Pricing

Client-side token pricing is available via `sysone::pricer`:

- Fixed rate of `0.042` USD per one million tokens (`PRICE_PER_MILLION_TOKENS`).
- Applied to models starting with `jev` (such as `jev-latest`).
- Other models return `None` for pricing and cost.
- Cost is computed client-side from the input tokens via `pricer::cost(model, input_tokens)`.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.

Copyright (c) 2026 BriteSnow, Inc., https://britesnow.com
