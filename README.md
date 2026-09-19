# TypeSafe AI System One Rust Client Library (unofficial)

By the same author of the [genai](https://crates.io/crates/genai) crate (Jeremy Chone).

- Very early release `0.0.x`
- Extremely basic functionality / API surface for now
- Feel free to cherry pick what you need for now.

Part of the [zcoder.run](https://zcoder.run) Rust libraries, and will probably be used in the zcoder harness (still in the building).

## Usage

```rust
use sysone::{Client, Request};
use serde_json::json;

// Set TYPESAFE_API_KEY in the environment or configure it on the builder:
// Client::builder().with_api_key("...").build()?;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::default();

    let req = Request::from_state(json!({ "code": "fn main() {}" }))
        .append_question("intent", "What does this code do?")
        .append_question("improvements", "What should be improved?");

    let res = client.exec(req).await?;
    println!("{res}");
    Ok(())
}
```

### Request

`Request` follows the fluid API style, every value is an `impl Into<serde_json::Value>`, and every question is keyed, so the `questions` value is always a dictionary (JSON object) of question key to question body, a question with an existing key overriding the previous one:

Constructors:

- `Request::from_state(state)` - state only, no question yet (`Request::new` is an alias of it).
- `Request::from_state_questions(state, questions)` - state and questions at once.

```rust
use sysone::Request;
use serde_json::json;

// Replace the full question set (a dictionary of question key to question body)
let req = Request::from_state("current state")
    .with_questions(json!({ "intent": "What does this code do?" }));

// Or build it up one question at a time
let req = Request::default()
    .with_state(json!({ "step": 1 }))
    .append_question("intent", "What does this code do?")
    .extend_questions([("improvements", "What should be improved?"), ("tests", "What should be checked?")]);
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.

Copyright (c) 2026 BriteSnow, Inc., https://britesnow.com
