/// Encoding Detection Layer for the Dynamic Rule Engine.
/// Provides multi-layer content decoding (Unicode escapes, Base64, URL encoding)
/// to detect obfuscated payloads that bypass simple string matching.

/// Represents a decoded content fragment with its source encoding.
#[derive(Debug, Clone)]
pub struct DecodedContent {
    pub source: String,
    pub content: String,
}

/// Detects and decodes various encoding layers in response bodies.
pub struct EncodingDetector;

impl EncodingDetector {
    /// Analyzes the body for encoded content and returns all decoded versions.
    pub fn detect_and_decode(body: &str) -> Vec<DecodedContent> {
        let mut results = Vec::new();

        // 1. HTML entity decoding: &lt; &gt; &amp; &#60; &#x3C; etc.
        if body.contains('&') {
            let decoded = decode_html_entities(body);
            if decoded != body {
                results.push(DecodedContent {
                    source: "html_entity".to_string(),
                    content: decoded,
                });
            }
        }

        // 2. Unicode escape sequences: \u003c\u0073...
        if body.contains("\\u") {
            let decoded = decode_unicode_escapes(body);
            if decoded != body {
                results.push(DecodedContent {
                    source: "unicode_escape".to_string(),
                    content: decoded,
                });
            }
        }

        // 3. Base64 in HTML comments: <!-- Base64: ... -->
        if let Some(base64_content) = extract_base64_from_comments(body) {
            if let Some(decoded) = base64_decode(&base64_content) {
                results.push(DecodedContent {
                    source: "base64_comment".to_string(),
                    content: decoded,
                });
            }
        }

        // 4. URL encoding: Apache%2F2.4.49
        if body.contains('%') {
            let decoded = url_decode(body);
            if decoded != body {
                results.push(DecodedContent {
                    source: "url_encoded".to_string(),
                    content: decoded,
                });
            }
        }

        results
    }

    /// Checks if any decoded version of the body contains the given value.
    pub fn body_contains_decoded(body: &str, value: &str) -> Option<DecodedContent> {
        let decoded_layers = Self::detect_and_decode(body);
        let value_lower = value.to_lowercase();

        decoded_layers
            .into_iter()
            .find(|layer| layer.content.to_lowercase().contains(&value_lower))
    }
}

/// Decodes Unicode escape sequences (\uXXXX) in a string.
fn decode_unicode_escapes(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\\' && chars.peek() == Some(&'u') {
            chars.next(); // consume 'u'
            let hex: String = chars.by_ref().take(4).collect();
            if hex.len() == 4 {
                if let Ok(code_point) = u32::from_str_radix(&hex, 16) {
                    if let Some(decoded_char) = char::from_u32(code_point) {
                        result.push(decoded_char);
                        continue;
                    }
                }
            }
            // If decode failed, keep original
            result.push('\\');
            result.push('u');
            result.push_str(&hex);
        } else {
            result.push(ch);
        }
    }

    result
}

/// Decodes HTML entities in a string.
/// Supports named entities (&amp; &lt; &gt; &quot; &apos; etc.),
/// decimal numeric references (&#60;), and hex numeric references (&#x3C;).
fn decode_html_entities(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut remaining = input;

    while let Some(amp_pos) = remaining.find('&') {
        // Push everything before the '&'
        result.push_str(&remaining[..amp_pos]);
        remaining = &remaining[amp_pos..];

        // Try to find the closing ';'
        if let Some(semi_pos) = remaining[1..].find(';') {
            let entity = &remaining[1..semi_pos + 1];

            if let Some(decoded) = resolve_html_entity(entity) {
                result.push_str(&decoded);
                remaining = &remaining[semi_pos + 2..];
                continue;
            }
        }

        // Not a valid entity, push '&' literally and move on
        result.push('&');
        remaining = &remaining[1..];
    }

    // Push any remaining text
    result.push_str(remaining);
    result
}

/// Resolves a single HTML entity reference (without the & and ;).
fn resolve_html_entity(entity: &str) -> Option<String> {
    // Numeric references: &#60; or &#x3C;
    if let Some(stripped) = entity.strip_prefix('#') {
        let code_point = if let Some(hex_str) = stripped
            .strip_prefix('x')
            .or_else(|| stripped.strip_prefix('X'))
        {
            u32::from_str_radix(hex_str, 16).ok()
        } else {
            stripped.parse::<u32>().ok()
        };

        return code_point.and_then(char::from_u32).map(|c| c.to_string());
    }

    // Named entity references
    let decoded = match entity {
        // Core XML entities
        "lt" => "<",
        "gt" => ">",
        "amp" => "&",
        "quot" => "\"",
        "apos" => "'",
        // Common HTML entities
        "nbsp" => "\u{00A0}",
        "copy" => "©",
        "reg" => "®",
        "trade" => "™",
        "mdash" => "—",
        "ndash" => "–",
        "laquo" => "«",
        "raquo" => "»",
        "bull" => "•",
        "hellip" => "…",
        "lsquo" => "\u{2018}",
        "rsquo" => "\u{2019}",
        "ldquo" => "\u{201C}",
        "rdquo" => "\u{201D}",
        "frasl" => "/",
        "sol" => "/",
        "colon" => ":",
        "semi" => ";",
        "equals" => "=",
        "quest" => "?",
        "excl" => "!",
        "num" => "#",
        "percnt" => "%",
        "plus" => "+",
        "minus" => "−",
        "ast" => "*",
        "comma" => ",",
        "period" => ".",
        "tab" => "\t",
        "NewLine" => "\n",
        _ => return None,
    };

    Some(decoded.to_string())
}

/// Extracts Base64 content from HTML comments (<!-- Base64: ... -->)
fn extract_base64_from_comments(body: &str) -> Option<String> {
    // Look for patterns like <!-- Base64: XXXXX --> or <!--XXXXX-->
    let patterns = ["<!-- Base64:", "<!--Base64:", "<!-- base64:", "<!--base64:"];

    for pattern in &patterns {
        if let Some(start_idx) = body.find(pattern) {
            let content_start = start_idx + pattern.len();
            if let Some(end_idx) = body[content_start..].find("-->") {
                let base64_str = body[content_start..content_start + end_idx].trim();
                if !base64_str.is_empty() && is_likely_base64(base64_str) {
                    return Some(base64_str.to_string());
                }
            }
        }
    }

    None
}

/// Simple Base64 decoder (no external dependency).
fn base64_decode(input: &str) -> Option<String> {
    // Standard Base64 alphabet
    let alphabet = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let input_clean: Vec<u8> = input
        .bytes()
        .filter(|b| *b != b'=' && *b != b'\n' && *b != b'\r' && *b != b' ')
        .collect();

    let mut lookup = [255u8; 256];
    for (i, &b) in alphabet.iter().enumerate() {
        lookup[b as usize] = i as u8;
    }

    let mut output = Vec::new();
    let chunks = input_clean.chunks(4);

    for chunk in chunks {
        let mut buf: u32 = 0;
        let valid_chars = chunk.len();

        for (i, &byte) in chunk.iter().enumerate() {
            let val = lookup[byte as usize];
            if val == 255 {
                return None; // Invalid Base64 character
            }
            buf |= (val as u32) << (6 * (3 - i));
        }

        if valid_chars >= 2 {
            output.push((buf >> 16) as u8);
        }
        if valid_chars >= 3 {
            output.push((buf >> 8) as u8);
        }
        if valid_chars >= 4 {
            output.push(buf as u8);
        }
    }

    String::from_utf8(output).ok()
}

/// URL-decodes a string (%XX -> character).
fn url_decode(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars();

    while let Some(ch) = chars.next() {
        if ch == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if hex.len() == 2 {
                if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                    result.push(byte as char);
                    continue;
                }
            }
            result.push('%');
            result.push_str(&hex);
        } else {
            result.push(ch);
        }
    }

    result
}

/// Heuristic check: is this string likely Base64?
fn is_likely_base64(s: &str) -> bool {
    if s.len() < 4 {
        return false;
    }
    s.bytes().all(|b| {
        b.is_ascii_alphanumeric()
            || b == b'+'
            || b == b'/'
            || b == b'='
            || b == b'\n'
            || b == b'\r'
            || b == b' '
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unicode_decode() {
        let input = r#"\u003cscript\u003ealert(1)\u003c/script\u003e"#;
        let decoded = decode_unicode_escapes(input);
        assert!(decoded.contains("<script>"));
        assert!(decoded.contains("alert(1)"));
    }

    #[test]
    fn test_url_decode() {
        let input = "Apache%2F2.4.49";
        let decoded = url_decode(input);
        assert_eq!(decoded, "Apache/2.4.49");
    }

    #[test]
    fn test_base64_decode() {
        // "PHNjcmlwdD5hbGVydCgxKTwvc2NyaXB0Pg==" is base64 for "<script>alert(1)</script>"
        let decoded = base64_decode("PHNjcmlwdD5hbGVydCgxKTwvc2NyaXB0Pg==");
        assert!(decoded.is_some());
        assert_eq!(decoded.unwrap(), "<script>alert(1)</script>");
    }

    #[test]
    fn test_body_contains_decoded() {
        let body = r#"Normal text with \u003cscript\u003ealert(1)\u003c/script\u003e hidden"#;
        let result = EncodingDetector::body_contains_decoded(body, "<script>");
        assert!(result.is_some());
        assert_eq!(result.unwrap().source, "unicode_escape");
    }

    #[test]
    fn test_html_entity_named() {
        let input = "&lt;script&gt;alert(1)&lt;/script&gt;";
        let decoded = decode_html_entities(input);
        assert_eq!(decoded, "<script>alert(1)</script>");
    }

    #[test]
    fn test_html_entity_decimal_numeric() {
        // &#60; = '<', &#62; = '>'
        let input = "&#60;script&#62;alert(1)&#60;/script&#62;";
        let decoded = decode_html_entities(input);
        assert_eq!(decoded, "<script>alert(1)</script>");
    }

    #[test]
    fn test_html_entity_hex_numeric() {
        // &#x3C; = '<', &#x3E; = '>'
        let input = "&#x3C;script&#x3E;alert(1)&#x3C;/script&#x3E;";
        let decoded = decode_html_entities(input);
        assert_eq!(decoded, "<script>alert(1)</script>");
    }

    #[test]
    fn test_html_entity_mixed() {
        let input = "Tom &amp; Jerry &lt;3 &quot;cartoons&quot;";
        let decoded = decode_html_entities(input);
        assert_eq!(decoded, "Tom & Jerry <3 \"cartoons\"");
    }

    #[test]
    fn test_html_entity_passthrough_unknown() {
        // Unknown entities should be left as-is
        let input = "Hello &unknown; world";
        let decoded = decode_html_entities(input);
        assert_eq!(decoded, "Hello &unknown; world");
    }

    #[test]
    fn test_body_contains_decoded_html_entity() {
        let body = "This page shows &lt;script&gt;alert(1)&lt;/script&gt; as an example";
        let result = EncodingDetector::body_contains_decoded(body, "<script>");
        assert!(result.is_some());
        assert_eq!(result.unwrap().source, "html_entity");
    }
}
