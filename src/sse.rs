use crate::config::LinkConfig;
use bytes::Bytes;
use futures_util::StreamExt;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

/// SSE client handler
pub struct SseClient {
    config: LinkConfig,
}

impl SseClient {
    /// Create a new SSE client
    pub fn new(config: LinkConfig) -> Self {
        Self { config }
    }

    /// Connect to the SSE server and start receiving events
    pub async fn run<F>(&self, mut message_handler: F) -> anyhow::Result<()>
    where
        F: FnMut(Vec<u8>) -> anyhow::Result<()> + Send,
    {
        let mut reconnect_attempts = 0u32;
        let mut current_delay = self.config.initial_reconnect_delay();

        loop {
            match self.connect_and_receive(&mut message_handler).await {
                Ok(_) => {
                    info!("SSE connection closed normally");
                    break Ok(());
                }
                Err(e) => {
                    error!("SSE connection error: {}", e);

                    // Check if we should retry
                    if self.config.max_reconnect_attempts > 0
                        && reconnect_attempts >= self.config.max_reconnect_attempts
                    {
                        error!(
                            "Maximum reconnection attempts ({}) reached",
                            self.config.max_reconnect_attempts
                        );
                        return Err(e);
                    }

                    reconnect_attempts += 1;
                    warn!(
                        "Attempting reconnection #{} after {:?}",
                        reconnect_attempts, current_delay
                    );

                    sleep(current_delay).await;

                    // Exponential backoff with max limit
                    current_delay =
                        std::cmp::min(current_delay * 2, self.config.max_reconnect_delay());
                }
            }
        }
    }

    /// Connect to SSE server and receive events
    async fn connect_and_receive<F>(&self, message_handler: &mut F) -> anyhow::Result<()>
    where
        F: FnMut(Vec<u8>) -> anyhow::Result<()>,
    {
        info!(
            "Connecting to SSE server: {}",
            self.config.sse_url
        );

        let client = reqwest::Client::new();
        let response = client
            .get(&self.config.sse_url)
            .header("Accept", "text/event-stream")
            .header("Cache-Control", "no-cache")
            .send()
            .await?;

        let status = response.status();
        info!("SSE connection established: {}", status);

        if !status.is_success() {
            anyhow::bail!("SSE server returned error status: {}", status);
        }

        let mut stream = response.bytes_stream();
        let mut buffer = String::new();

        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(chunk) => {
                    self.process_chunk(&chunk, &mut buffer, message_handler)?;
                }
                Err(e) => {
                    error!("Error receiving SSE data: {}", e);
                    return Err(e.into());
                }
            }
        }

        Ok(())
    }

    /// Process a chunk of SSE data and extract complete events
    fn process_chunk<F>(
        &self,
        chunk: &Bytes,
        buffer: &mut String,
        message_handler: &mut F,
    ) -> anyhow::Result<()>
    where
        F: FnMut(Vec<u8>) -> anyhow::Result<()>,
    {
        let chunk_str = String::from_utf8_lossy(chunk);
        buffer.push_str(&chunk_str);

        // SSE events are separated by double newlines
        while let Some(event_end) = buffer.find("\n\n") {
            let event_text = buffer[..event_end].to_string();
            *buffer = buffer[event_end + 2..].to_string();

            // Parse the SSE event
            if let Some(data) = Self::parse_sse_event(&event_text) {
                debug!("Received SSE event: {} bytes", data.len());

                if data.len() > self.config.max_message_size {
                    warn!(
                        "Event size {} exceeds limit {}, skipping",
                        data.len(),
                        self.config.max_message_size
                    );
                    continue;
                }

                message_handler(data.into_bytes())?;
            }
        }

        Ok(())
    }

    /// Parse an SSE event and extract the data field(s)
    fn parse_sse_event(event_text: &str) -> Option<String> {
        let mut data_lines = Vec::new();

        for line in event_text.lines() {
            if let Some(data) = line.strip_prefix("data:") {
                data_lines.push(data.trim().to_string());
            } else if let Some(data) = line.strip_prefix("data: ") {
                data_lines.push(data.to_string());
            }
        }

        if data_lines.is_empty() {
            None
        } else {
            Some(data_lines.join("\n"))
        }
    }
}
