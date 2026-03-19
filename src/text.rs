use codepage_437::CP437_WINGDINGS;

use super::errors::EncodeError;

/// Serialize a multi-line code string for an object or a scroll.
///
/// Characters that do not have a CP437 equivalent will result in an [`EncodeError::EncodingError`].
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

/// Serialize a single-line string, such as a board title.
///
/// Newlines cannot be encoded because in single-line contexts, ZZT interprets that byte as a
/// dingbat, specifically the '♪' character. Passing a newline, or any non-CP437 character, will
/// result in an [`EncodeError::EncodingError`].
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

/// Deserialize a single-line string, such as a board title.
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

    fn serialize_line(input: &str) -> Vec<u8> {
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
        assert_eq!(bytes, serialize_line(&decode_oneline(&bytes)))
    }

    #[test]
    fn one_codepoint_per_byte() {
        // This check guarantees that length checks on Unicode Strings will tell you something useful
        // about how much headroom your text will have once serialized into a field in the file format.
        let bytes: Vec<u8> = (0..=255).collect();
        assert_eq!(256, decode_oneline(&bytes).chars().count());
        assert_eq!(256, decode_multiline(&bytes).chars().count());
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
