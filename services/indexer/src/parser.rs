use serde_json::{json, Value};

/// Parsed event from on-chain transaction logs
pub struct ParsedEvent {
    pub event_type: String,
    pub data: Value,
}

/// Parse Anchor events from transaction log lines.
///
/// Anchor CPI events appear in logs as:
///   Program data: <base64-encoded event data>
///
/// The format is: 8-byte discriminator (sha256 of "event:<EventName>") + borsh-serialized data.
/// For simplicity, we detect known event patterns from the log messages themselves.
pub fn parse_events(logs: &[String], program_id: &str) -> Vec<ParsedEvent> {
    let mut events = Vec::new();
    let mut in_program = false;

    for log in logs {
        // Track program invocation context
        if log.contains(&format!("Program {program_id} invoke")) {
            in_program = true;
            continue;
        }
        if log.contains(&format!("Program {program_id} success"))
            || log.contains(&format!("Program {program_id} failed"))
        {
            in_program = false;
            continue;
        }

        if !in_program {
            continue;
        }

        // Parse program log messages for known event patterns
        if let Some(event) = parse_log_event(log) {
            events.push(event);
        }

        // Parse Anchor CPI event data
        if let Some(data_str) = log.strip_prefix("Program data: ") {
            if let Some(event) = parse_anchor_event(data_str) {
                events.push(event);
            }
        }
    }

    events
}

/// Parse structured log events from Program log messages
fn parse_log_event(log: &str) -> Option<ParsedEvent> {
    let msg = log.strip_prefix("Program log: ")?;

    // Detect common SSS events from log messages
    if msg.starts_with("Mint:") {
        let parts: Vec<&str> = msg.splitn(4, ',').collect();
        return Some(ParsedEvent {
            event_type: "MintEvent".to_string(),
            data: json!({
                "raw": msg,
                "parts": parts,
            }),
        });
    }

    if msg.starts_with("Burn:") {
        let parts: Vec<&str> = msg.splitn(4, ',').collect();
        return Some(ParsedEvent {
            event_type: "BurnEvent".to_string(),
            data: json!({
                "raw": msg,
                "parts": parts,
            }),
        });
    }

    if msg.starts_with("Blacklist:") {
        return Some(ParsedEvent {
            event_type: "BlacklistEvent".to_string(),
            data: json!({ "raw": msg }),
        });
    }

    if msg.starts_with("Seize:") {
        return Some(ParsedEvent {
            event_type: "SeizeEvent".to_string(),
            data: json!({ "raw": msg }),
        });
    }

    if msg.starts_with("Pause") || msg.starts_with("Unpause") {
        return Some(ParsedEvent {
            event_type: "PauseEvent".to_string(),
            data: json!({ "raw": msg }),
        });
    }

    if msg.starts_with("Freeze:") || msg.starts_with("Thaw:") {
        return Some(ParsedEvent {
            event_type: "FreezeEvent".to_string(),
            data: json!({ "raw": msg }),
        });
    }

    if msg.starts_with("RoleUpdate:") {
        return Some(ParsedEvent {
            event_type: "RoleUpdateEvent".to_string(),
            data: json!({ "raw": msg }),
        });
    }

    if msg.starts_with("Initialize:") {
        return Some(ParsedEvent {
            event_type: "InitializeEvent".to_string(),
            data: json!({ "raw": msg }),
        });
    }

    None
}

/// Parse Anchor CPI event from base64-encoded "Program data:" line
fn parse_anchor_event(data_b64: &str) -> Option<ParsedEvent> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(data_b64.trim())
        .ok()?;

    if bytes.len() < 8 {
        return None;
    }

    let discriminator = &bytes[..8];
    let payload = &bytes[8..];

    // Map known discriminators to event types
    // These are the first 8 bytes of sha256("event:<EventName>")
    let event_type = identify_event(discriminator)?;

    Some(ParsedEvent {
        event_type,
        data: json!({
            "discriminator": hex::encode(discriminator),
            "payload_hex": hex::encode(payload),
            "payload_len": payload.len(),
        }),
    })
}

/// Identify event type from discriminator
/// In a production system, these would be computed from actual event struct names
fn identify_event(discriminator: &[u8]) -> Option<String> {
    // Return a generic event type if we can't identify it
    // In production, you'd compute sha256("event:MintEvent")[..8] etc.
    Some(format!("AnchorEvent_{}", hex::encode(discriminator)))
}

mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_mint_event() {
        let logs = vec![
            format!("Program {} invoke [1]", "TestProgram111111111111111111111"),
            "Program log: Mint: recipient=abc, amount=1000, mint=xyz".to_string(),
            format!("Program {} success", "TestProgram111111111111111111111"),
        ];
        let events = parse_events(&logs, "TestProgram111111111111111111111");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "MintEvent");
    }

    #[test]
    fn test_parse_burn_event() {
        let logs = vec![
            format!("Program {} invoke [1]", "TestProgram111111111111111111111"),
            "Program log: Burn: source=abc, amount=500".to_string(),
            format!("Program {} success", "TestProgram111111111111111111111"),
        ];
        let events = parse_events(&logs, "TestProgram111111111111111111111");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "BurnEvent");
    }

    #[test]
    fn test_ignores_other_program_logs() {
        let logs = vec![
            "Program OtherProgram invoke [1]".to_string(),
            "Program log: Mint: something".to_string(),
            "Program OtherProgram success".to_string(),
        ];
        let events = parse_events(&logs, "TestProgram111111111111111111111");
        assert!(events.is_empty());
    }
}
