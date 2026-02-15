wit_bindgen::generate!({ generate_all });

use crate::exports::wasmcloud::messaging::handler::{Guest, BrokerMessage};
use crate::wasi::logging::logging::*;

struct SseComponent;

impl Guest for SseComponent {
    fn handle_message(msg: BrokerMessage) -> Result<(), String> {
        // Log the received broker message
        log(
            Level::Info,
            "",
            &format!(
                "Received message - Subject: {}, Size: {} bytes",
                msg.subject,
                msg.body.len()
            ),
        );

        // Try to interpret body as UTF-8 text for logging
        let payload_text = match String::from_utf8(msg.body.clone()) {
            Ok(text) => text,
            Err(_) => format!("[binary data: {} bytes]", msg.body.len()),
        };

        // Log the payload (truncated if too long)
        let payload_preview = if payload_text.len() > 100 {
            format!("{}...", &payload_text[..100])
        } else {
            payload_text
        };

        log(
            Level::Info,
            "",
            &format!("Message payload: {}", payload_preview),
        );

        if let Some(reply_to) = &msg.reply_to {
            log(
                Level::Info,
                "",
                &format!("Reply-to: {}", reply_to),
            );
        }

        // Successfully handled the message
        Ok(())
    }
}

export!(SseComponent);
