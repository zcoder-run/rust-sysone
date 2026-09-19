# TypeSafe AI System One Rust Client Library (unofficial)

By the same author of the [genai](https://crates.io/crates/genai) crate (Jeremy Chone).

- Very early release `0.0.x`
- Extremely basic functionality / API surface for now
- Feel free to cherry pick what you need for now. 

Part of the [zcoder.run](https://zcoder.run) Rust libraries, and will probably be used in the zcoder harness (still in the building).

## Usage

```rust
use sysone::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::default();
    let res = client.exec("current state", "questions").await?;
    println!("{res}");
    Ok(())
}
```

## License


Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.

Copyright (c) 2026 BriteSnow, Inc., https://britesnow.com
