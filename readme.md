# EvilCorp SecondPilot

A wrapper to the EvilCorp SecondPilot API, allowing developers to utilize its powerful auto-completion and code generation capabilities in their own applications. This client uses the `futures-util`, `reqwest`, and `serde` crates, among others.

## Features

- Simplifies interaction with EvilCorp SecondPilot API.
- Supports customization of API requests including model, stream, intent, and other parameters.
- Conveniently handles authorization with the EvilCorp SecondPilot API.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
evilcorp_secondpilot = "^2.0.0"
```

Then, import the library and use the `EvilcorpSecondPilotClient` struct as follows:

```rust
use evilcorp_secondpilot::{
    CompletionRequestBuilder,
    EvilcorpSecondPilotClient,
    Message,
};

let client =
    EvilcorpSecondPilotClient::new(
        "<your token>",
    );

let message = Message {
    content: "your message".to_string(),
    role: "user".to_string(),
};

let request = CompletionRequestBuilder::new()
    .with_model("<model>")
    .with_temperature(<temperature>)
    .with_top_p(<top_p>)
    .add_message(message)
    .build()
    .await;

let response = client.query(&request).await?;

println!("Response: {}", response);
```

## Structure

- `TokenResponse` is a helper struct that wraps the response from the token endpoint of the API.
- `CompletionRequest` is a helper struct that wraps a request to the completions endpoint.
- `Message` is a helper struct that wraps a message to be included in a `CompletionRequest`.
- `CompletionRequestBuilder` is a struct to help build `CompletionRequest` objects.
- `CompletionResponse`, `Choice`, and `Delta` are helper structs that wrap responses from the completions endpoint.
- `EvilcorpSecondPilotClient` is the main struct that communicates with the EvilCorp SecondPilot API.

## License

This project is licensed under the MIT License.

---

Please note that this project is in no way affiliated with, authorized, maintained, sponsored or endorsed by EvilCorp or any of its affiliates or subsidiaries. This is an independent and unofficial API wrapper.
