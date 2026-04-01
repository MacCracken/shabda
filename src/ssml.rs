//! SSML subset parser for speech synthesis markup.
//!
//! Parses a minimal subset of [SSML 1.1](https://www.w3.org/TR/speech-synthesis11/)
//! sufficient for common TTS control:
//!
//! - `<speak>` — root wrapper (optional)
//! - `<break time="500ms"/>` — insert pause
//! - `<emphasis level="strong|moderate|reduced">` — stress control
//! - `<prosody rate="slow|medium|fast|x-fast">` — speaking rate
//! - `<phoneme ph="...">` — literal phoneme override (passthrough as text)
//!
//! The parser is hand-rolled for `no_std + alloc` compatibility (no XML crate dependency).

use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

/// A parsed SSML node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SsmlNode {
    /// Plain text content.
    Text(String),
    /// A pause/break.
    Break {
        /// Duration in milliseconds.
        duration_ms: u32,
    },
    /// Emphasized text.
    Emphasis {
        /// Emphasis level.
        level: EmphasisLevel,
        /// Child nodes within the emphasis span.
        children: Vec<SsmlNode>,
    },
    /// Prosody-modified text.
    Prosody {
        /// Speaking rate override.
        rate: Option<SpeakingRate>,
        /// Child nodes within the prosody span.
        children: Vec<SsmlNode>,
    },
}

/// Emphasis level for SSML `<emphasis>` elements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum EmphasisLevel {
    /// Strong emphasis — maximum stress.
    Strong,
    /// Moderate emphasis — noticeable but not extreme.
    Moderate,
    /// Reduced emphasis — de-stressed.
    Reduced,
}

/// Speaking rate for SSML `<prosody>` elements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SpeakingRate {
    /// Extra slow (~75 WPM).
    XSlow,
    /// Slow (~100 WPM).
    Slow,
    /// Normal/medium (~150 WPM).
    Medium,
    /// Fast (~200 WPM).
    Fast,
    /// Extra fast (~250 WPM).
    XFast,
}

impl SpeakingRate {
    /// Returns the approximate words-per-minute for this rate.
    #[must_use]
    pub fn wpm(self) -> f32 {
        match self {
            Self::XSlow => 75.0,
            Self::Slow => 100.0,
            Self::Medium => 150.0,
            Self::Fast => 200.0,
            Self::XFast => 250.0,
        }
    }
}

/// Parses an SSML string into a sequence of nodes.
///
/// Supports a minimal SSML subset. Unknown tags are ignored (their text
/// content is preserved). The `<speak>` root wrapper is optional.
///
/// # Errors
///
/// Returns `Err` with a description if the SSML is malformed (unclosed tags,
/// invalid attributes).
///
/// # Examples
///
/// ```
/// use shabda::ssml::{parse, SsmlNode, EmphasisLevel};
///
/// let nodes = parse("Hello <break time=\"500ms\"/> world").unwrap();
/// assert_eq!(nodes.len(), 3);
/// assert!(matches!(&nodes[1], SsmlNode::Break { duration_ms: 500 }));
/// ```
pub fn parse(input: &str) -> Result<Vec<SsmlNode>, String> {
    let mut nodes = Vec::new();
    let mut pos = 0;
    let bytes = input.as_bytes();

    while pos < bytes.len() {
        if bytes[pos] == b'<' {
            // Parse a tag
            let tag_end =
                find_tag_end(input, pos).ok_or_else(|| String::from("unclosed tag bracket"))?;
            let tag_content = &input[pos + 1..tag_end];

            if tag_content.starts_with('/') {
                // Closing tag — handled by recursive parsers, skip here at top level
                pos = tag_end + 1;
            } else if tag_content.starts_with("speak") || tag_content.starts_with("speak ") {
                // <speak> wrapper — parse children until </speak>
                pos = tag_end + 1;
                let (children, new_pos) = parse_until(input, pos, "speak")?;
                nodes.extend(children);
                pos = new_pos;
            } else {
                let (node, new_pos) = parse_tag(input, pos)?;
                if let Some(n) = node {
                    nodes.push(n);
                }
                pos = new_pos;
            }
        } else {
            // Plain text until next '<' or end
            let text_end = input[pos..].find('<').map_or(input.len(), |i| pos + i);
            let text = &input[pos..text_end];
            let trimmed = text.trim();
            if !trimmed.is_empty() {
                nodes.push(SsmlNode::Text(String::from(trimmed)));
            }
            pos = text_end;
        }
    }

    Ok(nodes)
}

/// Parses a single tag at position `pos` (which points to '<').
/// Returns (optional node, new position after the tag/element).
fn parse_tag(input: &str, pos: usize) -> Result<(Option<SsmlNode>, usize), String> {
    let tag_end = find_tag_end(input, pos).ok_or_else(|| String::from("unclosed tag bracket"))?;
    let tag_content = &input[pos + 1..tag_end];

    // Self-closing tag?
    let self_closing = tag_content.ends_with('/');
    let tag_content = if self_closing {
        tag_content[..tag_content.len() - 1].trim()
    } else {
        tag_content.trim()
    };

    let (tag_name, attrs_str) = split_tag_name(tag_content);

    match tag_name {
        "break" => {
            let duration = parse_break_duration(attrs_str)?;
            Ok((
                Some(SsmlNode::Break {
                    duration_ms: duration,
                }),
                tag_end + 1,
            ))
        }
        "emphasis" => {
            let level = parse_emphasis_level(attrs_str);
            if self_closing {
                Ok((
                    Some(SsmlNode::Emphasis {
                        level,
                        children: Vec::new(),
                    }),
                    tag_end + 1,
                ))
            } else {
                let (children, new_pos) = parse_until(input, tag_end + 1, "emphasis")?;
                Ok((Some(SsmlNode::Emphasis { level, children }), new_pos))
            }
        }
        "prosody" => {
            let rate = parse_prosody_rate(attrs_str);
            if self_closing {
                Ok((
                    Some(SsmlNode::Prosody {
                        rate,
                        children: Vec::new(),
                    }),
                    tag_end + 1,
                ))
            } else {
                let (children, new_pos) = parse_until(input, tag_end + 1, "prosody")?;
                Ok((Some(SsmlNode::Prosody { rate, children }), new_pos))
            }
        }
        "phoneme" => {
            // Extract ph attribute and use as text content
            let ph = extract_attr(attrs_str, "ph");
            if self_closing {
                if let Some(phonemes) = ph {
                    Ok((Some(SsmlNode::Text(String::from(phonemes))), tag_end + 1))
                } else {
                    Ok((None, tag_end + 1))
                }
            } else {
                // Skip to closing tag, use text content
                let (children, new_pos) = parse_until(input, tag_end + 1, "phoneme")?;
                Ok((Some(SsmlNode::Text(children_to_text(&children))), new_pos))
            }
        }
        _ => {
            // Unknown tag — skip it, process content if not self-closing
            if self_closing {
                Ok((None, tag_end + 1))
            } else {
                let (mut children, new_pos) = parse_until(input, tag_end + 1, tag_name)?;
                // Pass through first child as if the tag wasn't there
                if children.is_empty() {
                    Ok((None, new_pos))
                } else {
                    Ok((Some(children.remove(0)), new_pos))
                }
            }
        }
    }
}

/// Parses content until a closing tag `</name>` is found.
/// Returns (children, position after closing tag).
fn parse_until(
    input: &str,
    start: usize,
    tag_name: &str,
) -> Result<(Vec<SsmlNode>, usize), String> {
    let mut nodes = Vec::new();
    let mut pos = start;
    let bytes = input.as_bytes();

    while pos < bytes.len() {
        if bytes[pos] == b'<' {
            // Check for closing tag
            let tag_end =
                find_tag_end(input, pos).ok_or_else(|| String::from("unclosed tag bracket"))?;
            let tag_content = &input[pos + 1..tag_end];

            if let Some(close_name) = tag_content.strip_prefix('/') {
                let close_name = close_name.trim();
                if close_name == tag_name {
                    return Ok((nodes, tag_end + 1));
                }
                // Mismatched close tag — skip it
                pos = tag_end + 1;
            } else {
                // Nested tag
                let (node, new_pos) = parse_tag(input, pos)?;
                if let Some(n) = node {
                    nodes.push(n);
                }
                pos = new_pos;
            }
        } else {
            let text_end = input[pos..].find('<').map_or(input.len(), |i| pos + i);
            let text = &input[pos..text_end];
            let trimmed = text.trim();
            if !trimmed.is_empty() {
                nodes.push(SsmlNode::Text(String::from(trimmed)));
            }
            pos = text_end;
        }
    }

    // Reached end without closing tag — return what we have
    Ok((nodes, pos))
}

/// Finds the position of '>' that closes the tag starting at `pos` (which is '<').
fn find_tag_end(input: &str, pos: usize) -> Option<usize> {
    let rest = &input[pos + 1..];
    rest.find('>').map(|i| pos + 1 + i)
}

/// Splits "tagname attr1=val1 attr2=val2" into ("tagname", "attr1=val1 attr2=val2").
fn split_tag_name(content: &str) -> (&str, &str) {
    match content.find(|c: char| c.is_whitespace()) {
        Some(i) => (&content[..i], content[i..].trim()),
        None => (content, ""),
    }
}

/// Extracts an attribute value from an attributes string.
fn extract_attr<'a>(attrs: &'a str, name: &str) -> Option<&'a str> {
    let needle = alloc::format!("{name}=\"");
    if let Some(start) = attrs.find(&needle) {
        let val_start = start + needle.len();
        let rest = &attrs[val_start..];
        if let Some(end) = rest.find('"') {
            return Some(&rest[..end]);
        }
    }
    // Also try single quotes
    let needle = alloc::format!("{name}='");
    if let Some(start) = attrs.find(&needle) {
        let val_start = start + needle.len();
        let rest = &attrs[val_start..];
        if let Some(end) = rest.find('\'') {
            return Some(&rest[..end]);
        }
    }
    None
}

/// Parses the `time` attribute from a `<break>` tag.
fn parse_break_duration(attrs: &str) -> Result<u32, String> {
    if let Some(time) = extract_attr(attrs, "time") {
        if let Some(ms_str) = time.strip_suffix("ms") {
            ms_str
                .parse::<u32>()
                .map_err(|_| alloc::format!("invalid break time: {time}"))
        } else if let Some(s_str) = time.strip_suffix('s') {
            let secs: f32 = s_str
                .parse()
                .map_err(|_| alloc::format!("invalid break time: {time}"))?;
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            Ok((secs * 1000.0) as u32)
        } else {
            Err(alloc::format!("unknown time unit in: {time}"))
        }
    } else if let Some(strength) = extract_attr(attrs, "strength") {
        // Map strength to duration
        match strength {
            "none" => Ok(0),
            "x-weak" => Ok(100),
            "weak" => Ok(200),
            "medium" => Ok(400),
            "strong" => Ok(600),
            "x-strong" => Ok(1000),
            _ => Ok(300),
        }
    } else {
        // Default break duration
        Ok(300)
    }
}

/// Parses the `level` attribute from an `<emphasis>` tag.
fn parse_emphasis_level(attrs: &str) -> EmphasisLevel {
    match extract_attr(attrs, "level") {
        Some("strong") => EmphasisLevel::Strong,
        Some("moderate") => EmphasisLevel::Moderate,
        Some("reduced") => EmphasisLevel::Reduced,
        _ => EmphasisLevel::Moderate,
    }
}

/// Parses the `rate` attribute from a `<prosody>` tag.
fn parse_prosody_rate(attrs: &str) -> Option<SpeakingRate> {
    match extract_attr(attrs, "rate") {
        Some("x-slow") => Some(SpeakingRate::XSlow),
        Some("slow") => Some(SpeakingRate::Slow),
        Some("medium") => Some(SpeakingRate::Medium),
        Some("fast") => Some(SpeakingRate::Fast),
        Some("x-fast") => Some(SpeakingRate::XFast),
        _ => None,
    }
}

/// Concatenates all text nodes in a children list.
fn children_to_text(children: &[SsmlNode]) -> String {
    let mut result = String::new();
    for child in children {
        if let SsmlNode::Text(t) = child {
            if !result.is_empty() {
                result.push(' ');
            }
            result.push_str(t);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain_text() {
        let nodes = parse("hello world").unwrap();
        assert_eq!(nodes, vec![SsmlNode::Text(String::from("hello world"))]);
    }

    #[test]
    fn test_break_ms() {
        let nodes = parse("hello <break time=\"500ms\"/> world").unwrap();
        assert_eq!(nodes.len(), 3);
        assert_eq!(nodes[1], SsmlNode::Break { duration_ms: 500 });
    }

    #[test]
    fn test_break_seconds() {
        let nodes = parse("<break time=\"1.5s\"/>").unwrap();
        assert_eq!(nodes, vec![SsmlNode::Break { duration_ms: 1500 }]);
    }

    #[test]
    fn test_break_strength() {
        let nodes = parse("<break strength=\"strong\"/>").unwrap();
        assert_eq!(nodes, vec![SsmlNode::Break { duration_ms: 600 }]);
    }

    #[test]
    fn test_emphasis() {
        let nodes = parse("<emphasis level=\"strong\">important</emphasis>").unwrap();
        assert_eq!(
            nodes,
            vec![SsmlNode::Emphasis {
                level: EmphasisLevel::Strong,
                children: vec![SsmlNode::Text(String::from("important"))],
            }]
        );
    }

    #[test]
    fn test_emphasis_default_level() {
        let nodes = parse("<emphasis>word</emphasis>").unwrap();
        match &nodes[0] {
            SsmlNode::Emphasis { level, .. } => assert_eq!(*level, EmphasisLevel::Moderate),
            _ => panic!("expected emphasis node"),
        }
    }

    #[test]
    fn test_prosody_rate() {
        let nodes = parse("<prosody rate=\"slow\">text</prosody>").unwrap();
        assert_eq!(
            nodes,
            vec![SsmlNode::Prosody {
                rate: Some(SpeakingRate::Slow),
                children: vec![SsmlNode::Text(String::from("text"))],
            }]
        );
    }

    #[test]
    fn test_speak_wrapper() {
        let nodes = parse("<speak>hello world</speak>").unwrap();
        assert_eq!(nodes, vec![SsmlNode::Text(String::from("hello world"))]);
    }

    #[test]
    fn test_nested_elements() {
        let nodes =
            parse("Start <emphasis level=\"strong\">very <break time=\"200ms\"/> important</emphasis> end")
                .unwrap();
        assert_eq!(nodes.len(), 3);
        assert_eq!(nodes[0], SsmlNode::Text(String::from("Start")));
        match &nodes[1] {
            SsmlNode::Emphasis { level, children } => {
                assert_eq!(*level, EmphasisLevel::Strong);
                assert_eq!(children.len(), 3);
            }
            _ => panic!("expected emphasis"),
        }
        assert_eq!(nodes[2], SsmlNode::Text(String::from("end")));
    }

    #[test]
    fn test_speaking_rate_wpm() {
        assert!((SpeakingRate::Slow.wpm() - 100.0).abs() < f32::EPSILON);
        assert!((SpeakingRate::Fast.wpm() - 200.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_serde_roundtrip_node() {
        let node = SsmlNode::Emphasis {
            level: EmphasisLevel::Strong,
            children: vec![SsmlNode::Text(String::from("hello"))],
        };
        let json = serde_json::to_string(&node).unwrap();
        let roundtripped: SsmlNode = serde_json::from_str(&json).unwrap();
        assert_eq!(node, roundtripped);
    }

    #[test]
    fn test_serde_roundtrip_rate() {
        let rate = SpeakingRate::Fast;
        let json = serde_json::to_string(&rate).unwrap();
        let roundtripped: SpeakingRate = serde_json::from_str(&json).unwrap();
        assert_eq!(rate, roundtripped);
    }

    #[test]
    fn test_serde_roundtrip_level() {
        let level = EmphasisLevel::Strong;
        let json = serde_json::to_string(&level).unwrap();
        let roundtripped: EmphasisLevel = serde_json::from_str(&json).unwrap();
        assert_eq!(level, roundtripped);
    }

    #[test]
    fn test_empty_input() {
        let nodes = parse("").unwrap();
        assert!(nodes.is_empty());
    }
}
