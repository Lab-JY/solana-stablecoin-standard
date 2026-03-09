use serde_json::{json, Value};

/// Parsed event from on-chain transaction logs
pub struct ParsedEvent {
    pub event_type: String,
    pub data: Value,
}

/// Anchor event discriminators: sha256("event:<EventName>")[..8]
/// Precomputed for all 13 SSS program events.
const EVENT_DISCRIMINATORS: &[([u8; 8], &str)] = &[
    ([0xee, 0xd9, 0x87, 0x0e, 0x93, 0x21, 0xdd, 0xa9], "StablecoinInitialized"),
    ([0xcf, 0xd4, 0x80, 0xc2, 0xaf, 0x36, 0x40, 0x18], "TokensMinted"),
    ([0xe6, 0xff, 0x22, 0x71, 0xe2, 0x35, 0xe3, 0x09], "TokensBurned"),
    ([0xdd, 0xd6, 0x3b, 0x1d, 0xf6, 0x32, 0x77, 0xce], "AccountFrozen"),
    ([0x31, 0x3f, 0x49, 0x69, 0x81, 0xbe, 0x28, 0x77], "AccountThawed"),
    ([0x48, 0x7b, 0x10, 0xbb, 0x32, 0xd6, 0x52, 0xc6], "StablecoinPaused"),
    ([0xb7, 0x50, 0x41, 0x3c, 0x80, 0x6d, 0x9b, 0x9b], "StablecoinUnpaused"),
    ([0x08, 0x7c, 0x42, 0x2d, 0xb0, 0x35, 0x31, 0x99], "MinterUpdated"),
    ([0x51, 0x25, 0xb0, 0x20, 0x1e, 0xcc, 0xfb, 0xf6], "RolesUpdated"),
    ([0xf5, 0x6d, 0xb3, 0x36, 0x87, 0x5c, 0x16, 0x40], "AuthorityTransferred"),
    ([0xaa, 0x2b, 0x19, 0x75, 0xfd, 0xc1, 0xc2, 0xe7], "AddressBlacklisted"),
    ([0x86, 0x15, 0x88, 0x6a, 0x29, 0x29, 0xf7, 0xe9], "AddressUnblacklisted"),
    ([0x33, 0x81, 0x83, 0x72, 0xce, 0xea, 0x8c, 0x7a], "TokensSeized"),
];

/// Parse Anchor events from transaction log lines.
///
/// Anchor CPI events appear in logs as:
///   Program data: <base64-encoded event data>
///
/// The format is: 8-byte discriminator (sha256 of "event:<EventName>") + borsh-serialized data.
/// We also detect known event patterns from the program log messages themselves.
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

/// Extract a key=value field from a comma-separated message fragment.
/// Given "recipient=abc" returns Some("abc"). Trims whitespace.
fn extract_field<'a>(segment: &'a str, key: &str) -> Option<&'a str> {
    let segment = segment.trim();
    let after = segment.strip_prefix(key)?.strip_prefix('=')?;
    Some(after.trim())
}

/// Parse structured log events from Program log messages.
/// Extracts structured fields instead of storing raw strings.
fn parse_log_event(log: &str) -> Option<ParsedEvent> {
    let msg = log.strip_prefix("Program log: ")?;

    if msg.starts_with("Mint:") {
        let body = msg.strip_prefix("Mint:").unwrap_or("");
        let parts: Vec<&str> = body.split(',').collect();
        let recipient = parts.iter().find_map(|p| extract_field(p, "recipient"));
        let amount = parts.iter().find_map(|p| extract_field(p, "amount"));
        let mint = parts.iter().find_map(|p| extract_field(p, "mint"));
        return Some(ParsedEvent {
            event_type: "MintEvent".to_string(),
            data: json!({
                "recipient": recipient.unwrap_or(""),
                "amount": amount.unwrap_or(""),
                "mint": mint.unwrap_or(""),
            }),
        });
    }

    if msg.starts_with("Burn:") {
        let body = msg.strip_prefix("Burn:").unwrap_or("");
        let parts: Vec<&str> = body.split(',').collect();
        let source = parts.iter().find_map(|p| extract_field(p, "source"));
        let amount = parts.iter().find_map(|p| extract_field(p, "amount"));
        return Some(ParsedEvent {
            event_type: "BurnEvent".to_string(),
            data: json!({
                "source": source.unwrap_or(""),
                "amount": amount.unwrap_or(""),
            }),
        });
    }

    if msg.starts_with("Blacklist:") {
        let body = msg.strip_prefix("Blacklist:").unwrap_or("");
        let parts: Vec<&str> = body.split(',').collect();
        let address = parts.iter().find_map(|p| extract_field(p, "address"));
        let reason = parts.iter().find_map(|p| extract_field(p, "reason"));
        return Some(ParsedEvent {
            event_type: "BlacklistEvent".to_string(),
            data: json!({
                "address": address.unwrap_or(""),
                "reason": reason.unwrap_or(""),
            }),
        });
    }

    if msg.starts_with("Seize:") {
        let body = msg.strip_prefix("Seize:").unwrap_or("");
        let parts: Vec<&str> = body.split(',').collect();
        let from = parts.iter().find_map(|p| extract_field(p, "from"));
        let to = parts.iter().find_map(|p| extract_field(p, "to"));
        let amount = parts.iter().find_map(|p| extract_field(p, "amount"));
        return Some(ParsedEvent {
            event_type: "SeizeEvent".to_string(),
            data: json!({
                "from": from.unwrap_or(""),
                "to": to.unwrap_or(""),
                "amount": amount.unwrap_or(""),
            }),
        });
    }

    if msg.starts_with("Freeze:") {
        let body = msg.strip_prefix("Freeze:").unwrap_or("");
        let parts: Vec<&str> = body.split(',').collect();
        let account = parts.iter().find_map(|p| extract_field(p, "account"));
        return Some(ParsedEvent {
            event_type: "FreezeEvent".to_string(),
            data: json!({
                "account": account.unwrap_or(""),
            }),
        });
    }

    if msg.starts_with("Thaw:") {
        let body = msg.strip_prefix("Thaw:").unwrap_or("");
        let parts: Vec<&str> = body.split(',').collect();
        let account = parts.iter().find_map(|p| extract_field(p, "account"));
        return Some(ParsedEvent {
            event_type: "ThawEvent".to_string(),
            data: json!({
                "account": account.unwrap_or(""),
            }),
        });
    }

    if msg.starts_with("Pause") || msg.starts_with("Unpause") {
        let action = if msg.starts_with("Unpause") {
            "unpause"
        } else {
            "pause"
        };
        return Some(ParsedEvent {
            event_type: "PauseEvent".to_string(),
            data: json!({
                "action": action,
            }),
        });
    }

    if msg.starts_with("RoleUpdate:") {
        let body = msg.strip_prefix("RoleUpdate:").unwrap_or("");
        let parts: Vec<&str> = body.split(',').collect();
        let role = parts.iter().find_map(|p| extract_field(p, "role"));
        let address = parts.iter().find_map(|p| extract_field(p, "address"));
        let action = parts.iter().find_map(|p| extract_field(p, "action"));
        return Some(ParsedEvent {
            event_type: "RoleUpdateEvent".to_string(),
            data: json!({
                "role": role.unwrap_or(""),
                "address": address.unwrap_or(""),
                "action": action.unwrap_or(""),
            }),
        });
    }

    if msg.starts_with("Initialize:") {
        let body = msg.strip_prefix("Initialize:").unwrap_or("");
        let parts: Vec<&str> = body.split(',').collect();
        let name = parts.iter().find_map(|p| extract_field(p, "name"));
        let symbol = parts.iter().find_map(|p| extract_field(p, "symbol"));
        let preset = parts.iter().find_map(|p| extract_field(p, "preset"));
        return Some(ParsedEvent {
            event_type: "InitializeEvent".to_string(),
            data: json!({
                "name": name.unwrap_or(""),
                "symbol": symbol.unwrap_or(""),
                "preset": preset.unwrap_or(""),
            }),
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

/// Identify event type from its 8-byte discriminator.
/// Matches against precomputed sha256("event:<EventName>")[..8] for all SSS events.
fn identify_event(discriminator: &[u8]) -> Option<String> {
    if discriminator.len() < 8 {
        return None;
    }

    let disc: [u8; 8] = discriminator[..8].try_into().ok()?;

    for (known_disc, name) in EVENT_DISCRIMINATORS {
        if disc == *known_disc {
            return Some(name.to_string());
        }
    }

    // Return a generic identifier for unrecognized Anchor events
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

    const TEST_PROGRAM: &str = "TestProgram111111111111111111111";

    fn wrap_logs(inner: &[&str]) -> Vec<String> {
        let mut logs = vec![format!("Program {TEST_PROGRAM} invoke [1]")];
        for line in inner {
            logs.push(line.to_string());
        }
        logs.push(format!("Program {TEST_PROGRAM} success"));
        logs
    }

    // --- Log-based event parsing tests ---

    #[test]
    fn test_parse_mint_event_structured() {
        let logs = wrap_logs(&["Program log: Mint: recipient=abc, amount=1000, mint=xyz"]);
        let events = parse_events(&logs, TEST_PROGRAM);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "MintEvent");
        assert_eq!(events[0].data["recipient"], "abc");
        assert_eq!(events[0].data["amount"], "1000");
        assert_eq!(events[0].data["mint"], "xyz");
    }

    #[test]
    fn test_parse_burn_event_structured() {
        let logs = wrap_logs(&["Program log: Burn: source=abc, amount=500"]);
        let events = parse_events(&logs, TEST_PROGRAM);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "BurnEvent");
        assert_eq!(events[0].data["source"], "abc");
        assert_eq!(events[0].data["amount"], "500");
    }

    #[test]
    fn test_parse_blacklist_event_structured() {
        let logs = wrap_logs(&["Program log: Blacklist: address=SomeAddr, reason=OFAC"]);
        let events = parse_events(&logs, TEST_PROGRAM);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "BlacklistEvent");
        assert_eq!(events[0].data["address"], "SomeAddr");
        assert_eq!(events[0].data["reason"], "OFAC");
    }

    #[test]
    fn test_parse_seize_event_structured() {
        let logs = wrap_logs(&["Program log: Seize: from=badActor, to=treasury, amount=9999"]);
        let events = parse_events(&logs, TEST_PROGRAM);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "SeizeEvent");
        assert_eq!(events[0].data["from"], "badActor");
        assert_eq!(events[0].data["to"], "treasury");
        assert_eq!(events[0].data["amount"], "9999");
    }

    #[test]
    fn test_parse_freeze_event_structured() {
        let logs = wrap_logs(&["Program log: Freeze: account=FrozenAcct"]);
        let events = parse_events(&logs, TEST_PROGRAM);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "FreezeEvent");
        assert_eq!(events[0].data["account"], "FrozenAcct");
    }

    #[test]
    fn test_parse_thaw_event_structured() {
        let logs = wrap_logs(&["Program log: Thaw: account=ThawedAcct"]);
        let events = parse_events(&logs, TEST_PROGRAM);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "ThawEvent");
        assert_eq!(events[0].data["account"], "ThawedAcct");
    }

    #[test]
    fn test_parse_pause_event() {
        let logs = wrap_logs(&["Program log: Pause"]);
        let events = parse_events(&logs, TEST_PROGRAM);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "PauseEvent");
        assert_eq!(events[0].data["action"], "pause");
    }

    #[test]
    fn test_parse_unpause_event() {
        let logs = wrap_logs(&["Program log: Unpause"]);
        let events = parse_events(&logs, TEST_PROGRAM);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "PauseEvent");
        assert_eq!(events[0].data["action"], "unpause");
    }

    #[test]
    fn test_parse_role_update_event_structured() {
        let logs = wrap_logs(&["Program log: RoleUpdate: role=minter, address=Minter1, action=grant"]);
        let events = parse_events(&logs, TEST_PROGRAM);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "RoleUpdateEvent");
        assert_eq!(events[0].data["role"], "minter");
        assert_eq!(events[0].data["address"], "Minter1");
        assert_eq!(events[0].data["action"], "grant");
    }

    #[test]
    fn test_parse_initialize_event_structured() {
        let logs = wrap_logs(&["Program log: Initialize: name=USDS, symbol=USDS, preset=default"]);
        let events = parse_events(&logs, TEST_PROGRAM);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "InitializeEvent");
        assert_eq!(events[0].data["name"], "USDS");
        assert_eq!(events[0].data["symbol"], "USDS");
        assert_eq!(events[0].data["preset"], "default");
    }

    // --- Anchor discriminator tests ---

    #[test]
    fn test_identify_known_discriminators() {
        for (disc, expected_name) in EVENT_DISCRIMINATORS {
            let result = identify_event(disc);
            assert_eq!(
                result.as_deref(),
                Some(*expected_name),
                "Discriminator for {expected_name} did not match"
            );
        }
    }

    #[test]
    fn test_identify_unknown_discriminator() {
        let unknown = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07];
        let result = identify_event(&unknown).unwrap();
        assert!(result.starts_with("AnchorEvent_"));
        assert!(result.contains("00010203"));
    }

    #[test]
    fn test_identify_short_discriminator() {
        let short = [0x00, 0x01, 0x02];
        assert!(identify_event(&short).is_none());
    }

    // --- Program context tracking tests ---

    #[test]
    fn test_ignores_other_program_logs() {
        let logs = vec![
            "Program OtherProgram invoke [1]".to_string(),
            "Program log: Mint: recipient=abc, amount=1000, mint=xyz".to_string(),
            "Program OtherProgram success".to_string(),
        ];
        let events = parse_events(&logs, TEST_PROGRAM);
        assert!(events.is_empty());
    }

    #[test]
    fn test_stops_parsing_after_program_failure() {
        let logs = vec![
            format!("Program {TEST_PROGRAM} invoke [1]"),
            format!("Program {TEST_PROGRAM} failed"),
            "Program log: Mint: recipient=abc, amount=1000, mint=xyz".to_string(),
        ];
        let events = parse_events(&logs, TEST_PROGRAM);
        assert!(events.is_empty());
    }

    #[test]
    fn test_multiple_events_in_single_invocation() {
        let logs = wrap_logs(&[
            "Program log: Mint: recipient=alice, amount=100, mint=usd",
            "Program log: Mint: recipient=bob, amount=200, mint=usd",
        ]);
        let events = parse_events(&logs, TEST_PROGRAM);
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].data["recipient"], "alice");
        assert_eq!(events[1].data["recipient"], "bob");
    }

    #[test]
    fn test_empty_logs() {
        let events = parse_events(&[], TEST_PROGRAM);
        assert!(events.is_empty());
    }

    #[test]
    fn test_logs_without_program_context() {
        let logs = vec![
            "Program log: Mint: recipient=abc, amount=1000, mint=xyz".to_string(),
        ];
        let events = parse_events(&logs, TEST_PROGRAM);
        assert!(events.is_empty());
    }

    // --- Field extraction edge cases ---

    #[test]
    fn test_mint_event_missing_fields() {
        let logs = wrap_logs(&["Program log: Mint: recipient=abc"]);
        let events = parse_events(&logs, TEST_PROGRAM);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data["recipient"], "abc");
        assert_eq!(events[0].data["amount"], "");
        assert_eq!(events[0].data["mint"], "");
    }

    #[test]
    fn test_extract_field_utility() {
        assert_eq!(extract_field("recipient=abc", "recipient"), Some("abc"));
        assert_eq!(extract_field(" amount=1000 ", "amount"), Some("1000"));
        assert_eq!(extract_field("role=minter", "amount"), None);
        assert_eq!(extract_field("", "key"), None);
    }
}
