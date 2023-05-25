use futures_util::StreamExt;
use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION, USER_AGENT},
    Client,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct TokenResponse {
    pub token: String,
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
}

impl EvilcorpSecondPilotClient {
    pub fn new(token: String) -> Self {
        Self { token }
    }

    pub async fn query_simple(
        &self,
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

        let token = self.get_token().await?;

        self.query(
            &request,
            &token,
        ).await
    }

    pub async fn query(
        &self,
        req: &CompletionRequest,
        token: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let client = Client::new();

        let response = client
            .post("https://copilot-proxy.githubusercontent.com/v1/chat/completions")
            .header(USER_AGENT, "GithubCopilot/1.86.92")
            .header(AUTHORIZATION, format!("Bearer {}", token))
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

    async fn get_token(&self) -> Result<String, Box<dyn std::error::Error>> {
        let client = Client::new();
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("token {}", self.token))?,
        );
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static("GithubCopilot/1.86.92"),
        );
        let response = client
            .get("https://api.github.com/copilot_internal/v2/token")
            .headers(headers)
            .send()
            .await?
            .json::<TokenResponse>()
            .await?;
        Ok(response.token)
    }
}