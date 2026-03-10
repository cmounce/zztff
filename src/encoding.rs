use codepage_437::CP437_WINGDINGS;

use super::error::EncodeError;

/// Serialize a multi-line code string for an object or a scroll.
/// Uses ZZT's convention of CR-terminated lines.
pub fn encode_multiline(input: &str) -> Result<Vec<u8>, EncodeError> {
    input
        .chars()
        .map(|c| {
            if c == '\n' {
                Ok(b'\r')
            } else {
                CP437_WINGDINGS
                    .encode(c)
                    .ok_or_else(|| EncodeError::EncodingError(c))
            }
        })
        .collect()
}

/// Deserialize a multi-line code string.
pub fn decode_multiline(input: &[u8]) -> String {
    input
        .iter()
        .map(|&x| {
            if x == b'\r' {
                '\n'
            } else {
                CP437_WINGDINGS.decode(x)
            }
        })
        .collect()
}

/// Serialize a single-line string for a board title.
/// Newlines cannot be encoded because ZZT interprets them as dingbats (♪).
pub fn encode_oneline(input: &str) -> Result<Vec<u8>, EncodeError> {
    input
        .chars()
        .map(|c| {
            CP437_WINGDINGS
                .encode(c)
                .ok_or_else(|| EncodeError::EncodingError(c))
        })
        .collect()
}

/// Deserialize a board title.
pub fn decode_oneline(input: &[u8]) -> String {
    input.iter().map(|&x| CP437_WINGDINGS.decode(x)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_debug_snapshot;

    fn serialize_code(input: &str) -> Vec<u8> {
        encode_multiline(input).expect("Error in test")
    }

    fn serialize_title(input: &str) -> Vec<u8> {
        encode_oneline(input).expect("Error in test")
    }

    #[test]
    fn roundtrip_multiline() {
        let bytes: Vec<u8> = (0..=255).collect();
        assert_eq!(bytes, serialize_code(&decode_multiline(&bytes)))
    }

    #[test]
    fn roundtrip_oneline() {
        let bytes: Vec<u8> = (0..=255).collect();
        assert_eq!(bytes, serialize_title(&decode_oneline(&bytes)))
    }

    #[test]
    fn newlines_to_cr() {
        assert_debug_snapshot!(serialize_code("ABC\nDEF"), @r###"
        [
            65,
            66,
            67,
            13,
            68,
            69,
            70,
        ]
        "###)
    }

    #[test]
    fn byte_13_to_dingbat() {
        // Code is allowed to have newlines
        let bytes = serialize_code("ABC\nDEF");
        assert!(bytes.contains(&13));

        // But in a board title, that same byte is a dingbat
        assert_debug_snapshot!(decode_oneline(&bytes), @r#""ABC♪DEF""#);
    }

    #[test]
    fn wingdings() {
        assert_debug_snapshot!(decode_multiline(&(0..32).collect::<Vec<_>>()), @r###""\0☺☻♥♦♣♠•◘○◙♂♀\n♫☼►◄↕‼¶§▬↨↑↓→←∟↔▲▼""###);
        assert_debug_snapshot!(decode_multiline(&(112..144).collect::<Vec<_>>()), @r###""pqrstuvwxyz{|}~⌂ÇüéâäàåçêëèïîìÄÅ""###);
        assert_debug_snapshot!(decode_multiline(&(224..=255).collect::<Vec<_>>()), @r###""αßΓπΣσµτΦΘΩδ∞φε∩≡±≥≤⌠⌡÷≈°∙·√ⁿ²■\u{a0}""###);
    }
}
