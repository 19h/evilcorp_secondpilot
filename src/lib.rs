use chrono::Utc;
use futures_util::StreamExt;
use reqwest::{
    Client,
    header::{AUTHORIZATION, HeaderMap, HeaderValue, USER_AGENT},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct TokenResponse {
    pub chat_enabled: bool,
    pub code_quote_enabled: bool,
    pub copilotignore_enabled: bool,
    pub expires_at: i64,
    pub public_suggestions: String,
    pub refresh_in: i64,
    pub sku: String,
    pub telemetry: String,
    pub token: String,
    pub tracking_id: String,
}

#[derive(Debug, Serialize)]
pub struct CompletionRequest {
    pub stream: bool,
    pub intent: bool,
    pub messages: Vec<Message>,
    pub model: String,
    pub temperature: f32,
    pub top_p: i32,
    pub n: i32,
}

impl Default for CompletionRequest {
    fn default() -> Self {
        Self {
            stream: false,
            intent: false,
            messages: vec![],
            model: "idk".to_string(),
            temperature: 0.1,
            top_p: 1,
            n: 1,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Message {
    pub content: String,
    pub role: String,
}

pub struct CompletionRequestBuilder {
    request: CompletionRequest,
}

impl CompletionRequestBuilder {
    pub fn new() -> Self {
        Self {
            request: CompletionRequest::default(),
        }
    }

    pub fn with_stream(mut self, stream: bool) -> Self {
        self.request.stream = stream;
        self
    }

    pub fn with_intent(mut self, intent: bool) -> Self {
        self.request.intent = intent;
        self
    }

    pub fn with_model(mut self, model: String) -> Self {
        self.request.model = model;
        self
    }

    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.request.temperature = temperature;
        self
    }

    pub fn with_top_p(mut self, top_p: i32) -> Self {
        self.request.top_p = top_p;
        self
    }

    pub fn with_n(mut self, n: i32) -> Self {
        self.request.n = n;
        self
    }

    pub fn add_message(mut self, message: Message) -> Self {
        self.request.messages.push(message);
        self
    }

    pub fn with_messages(mut self, messages: Vec<Message>) -> Self {
        self.request.messages = messages;
        self
    }

    pub async fn build(self) -> CompletionRequest {
        self.request
    }
}

#[derive(Debug, Deserialize)]
pub struct CompletionResponse {
    pub choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
pub struct Choice {
    pub delta: Delta,
}

#[derive(Debug, Deserialize)]
pub struct Delta {
    pub content: String,
}

pub struct EvilcorpSecondPilotClient {
    pub token: String,
    pub token_expires_at: i64,
}

impl EvilcorpSecondPilotClient {
    pub fn new(token: String) -> Self {
        Self { token, token_expires_at: 0 }
    }

    pub async fn get_token(
        &self
    ) -> Result<TokenResponse, Box<dyn std::error::Error>> {
        let client = Client::new();

        Ok(
            client
                .get("https://api.github.com/copilot_internal/v2/token")
                .header(USER_AGENT, "GithubCopilot/3.99.99")
                .header(AUTHORIZATION, format!("Bearer {}", self.token))
                .send()
                .await?
                .json::<TokenResponse>()
                .await?
        )
    }

    pub async fn get_or_refresh_token(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        if self.token_expires_at < Utc::now().timestamp() {
            let response = self.get_token().await?;

            self.token = response.token;
            self.token_expires_at = response.expires_at;
        }

        Ok(
            self.token
                .clone(),
        )
    }

    pub async fn query_simple(
        &mut self,
        message: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let message = Message {
            content: message.to_string(),
            role: "user".to_string(),
        };

        let request = CompletionRequest {
            messages: vec![message],

            ..Default::default()
        };

        self.query(
            &request,
        ).await
    }

    pub async fn query(
        &mut self,
        req: &CompletionRequest,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let client = Client::new();

        let token =
            self.get_or_refresh_token()
                .await?;

        let response = client
            .post("https://copilot-proxy.githubusercontent.com/v1/chat/completions")
            .header(
                USER_AGENT,
                "GithubCopilot/1.86.92",
            )
            .header(
                AUTHORIZATION,
                format!("Bearer {}", token),
            )
            .json(&req)
            .send()
            .await?;

        let mut stream = response.bytes_stream();

        let mut final_string = String::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?.to_vec();
            let chunk_str = std::str::from_utf8(&chunk)?;

            for line in chunk_str.lines() {
                if !line.starts_with("data: ") {
                    continue;
                }

                let line = &line[6..];

                if let Some(delta) = serde_json::from_str::<CompletionResponse>(line)
                    .ok()
                    .and_then(|r| r.choices.into_iter().next())
                    .map(|c| c.delta)
                {
                    final_string.push_str(&delta.content);
                }
            }
        }

        Ok(final_string)
    }
}
