use std::{error::Error, fmt};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProtectedText {
    pub text: String,
    originals: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlaceholderError {
    CountMismatch {
        expected: usize,
        actual: usize,
    },
    OrderMismatch {
        expected: Vec<usize>,
        actual: Vec<usize>,
    },
    ContentMismatch,
    MalformedMarkup,
}

impl fmt::Display for PlaceholderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CountMismatch { expected, actual } => {
                write!(
                    formatter,
                    "expected {expected} placeholders, found {actual}"
                )
            }
            Self::OrderMismatch { expected, actual } => {
                write!(
                    formatter,
                    "placeholder order {actual:?} does not match {expected:?}"
                )
            }
            Self::MalformedMarkup => formatter.write_str("malformed placeholder markup"),
            Self::ContentMismatch => formatter.write_str("control codes changed"),
        }
    }
}

/// Verifies that a reviewed translation preserves every original control code exactly.
///
/// # Errors
///
/// Returns [`PlaceholderError::ContentMismatch`] when codes are missing, added, or changed.
pub fn validate_control_codes(source: &str, target: &str) -> Result<(), PlaceholderError> {
    let source = protect_placeholders(source);
    let target = protect_placeholders(target);
    if source.originals == target.originals {
        Ok(())
    } else {
        Err(PlaceholderError::ContentMismatch)
    }
}

impl Error for PlaceholderError {}

#[must_use]
pub fn protect_placeholders(source: &str) -> ProtectedText {
    let mut text = String::with_capacity(source.len());
    let mut originals = Vec::new();
    let mut offset = 0;

    while offset < source.len() {
        if let Some(length) = control_code_length(&source[offset..]) {
            originals.push(source[offset..offset + length].to_owned());
            text.push_str("<ph id=\"");
            text.push_str(&(originals.len() - 1).to_string());
            text.push_str("\"/>");
            offset += length;
        } else {
            let Some(character) = source[offset..].chars().next() else {
                break;
            };
            text.push(character);
            offset += character.len_utf8();
        }
    }

    ProtectedText { text, originals }
}

/// Restores protected RPG Maker control codes after validating their count and order.
///
/// # Errors
///
/// Returns [`PlaceholderError`] when placeholder markup is malformed, missing, added, or reordered.
pub fn restore_placeholders(
    protected: &ProtectedText,
    translated: &str,
) -> Result<String, PlaceholderError> {
    let occurrences = parse_placeholders(translated)?;
    if occurrences.len() != protected.originals.len() {
        return Err(PlaceholderError::CountMismatch {
            expected: protected.originals.len(),
            actual: occurrences.len(),
        });
    }

    let expected = (0..protected.originals.len()).collect::<Vec<_>>();
    let actual = occurrences
        .iter()
        .map(|occurrence| occurrence.id)
        .collect::<Vec<_>>();
    if actual != expected {
        return Err(PlaceholderError::OrderMismatch { expected, actual });
    }

    let mut restored = String::with_capacity(translated.len());
    let mut cursor = 0;
    for occurrence in occurrences {
        restored.push_str(&translated[cursor..occurrence.start]);
        restored.push_str(&protected.originals[occurrence.id]);
        cursor = occurrence.end;
    }
    restored.push_str(&translated[cursor..]);
    Ok(restored)
}

fn control_code_length(value: &str) -> Option<usize> {
    if let Some(length) = renpy_token_length(value) {
        return Some(length);
    }
    let bytes = value.as_bytes();
    if bytes.len() < 5
        || bytes[0] != b'\\'
        || !matches!(bytes[1], b'V' | b'N' | b'C' | b'I')
        || bytes[2] != b'['
    {
        return None;
    }

    let mut index = 3;
    let digits_start = index;
    while index < bytes.len() && bytes[index].is_ascii_digit() {
        index += 1;
    }
    (index > digits_start && bytes.get(index) == Some(&b']')).then_some(index + 1)
}

fn renpy_token_length(value: &str) -> Option<usize> {
    let bytes = value.as_bytes();
    let closing = match bytes.first()? {
        b'[' if bytes.get(1) != Some(&b'[') => b']',
        b'{' if bytes.get(1) != Some(&b'{') => b'}',
        _ => return None,
    };
    let end = bytes[1..].iter().position(|byte| *byte == closing)? + 1;
    (end > 1 && !bytes[1..end].contains(&b'\n')).then_some(end + 1)
}

struct PlaceholderOccurrence {
    id: usize,
    start: usize,
    end: usize,
}

fn parse_placeholders(value: &str) -> Result<Vec<PlaceholderOccurrence>, PlaceholderError> {
    let mut occurrences = Vec::new();
    let mut cursor = 0;
    while let Some(relative_start) = value[cursor..].find("<ph") {
        let start = cursor + relative_start;
        let remainder = &value[start..];
        let (id, length) = parse_placeholder(remainder)?;
        let end = start + length;
        occurrences.push(PlaceholderOccurrence { id, start, end });
        cursor = end;
    }
    Ok(occurrences)
}

fn parse_placeholder(value: &str) -> Result<(usize, usize), PlaceholderError> {
    let bytes = value.as_bytes();
    if !bytes.starts_with(b"<ph") {
        return Err(PlaceholderError::MalformedMarkup);
    }
    let mut cursor = 3;
    skip_ascii_whitespace(bytes, &mut cursor);
    if !bytes
        .get(cursor..)
        .is_some_and(|rest| rest.starts_with(b"id"))
    {
        return Err(PlaceholderError::MalformedMarkup);
    }
    cursor += 2;
    skip_ascii_whitespace(bytes, &mut cursor);
    if bytes.get(cursor) != Some(&b'=') {
        return Err(PlaceholderError::MalformedMarkup);
    }
    cursor += 1;
    skip_ascii_whitespace(bytes, &mut cursor);
    let quote = *bytes.get(cursor).ok_or(PlaceholderError::MalformedMarkup)?;
    if !matches!(quote, b'\'' | b'"') {
        return Err(PlaceholderError::MalformedMarkup);
    }
    cursor += 1;
    let digits_start = cursor;
    while bytes.get(cursor).is_some_and(u8::is_ascii_digit) {
        cursor += 1;
    }
    if cursor == digits_start || bytes.get(cursor) != Some(&quote) {
        return Err(PlaceholderError::MalformedMarkup);
    }
    let id = value[digits_start..cursor]
        .parse::<usize>()
        .map_err(|_| PlaceholderError::MalformedMarkup)?;
    cursor += 1;
    skip_ascii_whitespace(bytes, &mut cursor);
    if bytes.get(cursor..cursor + 2) != Some(b"/>") {
        return Err(PlaceholderError::MalformedMarkup);
    }
    Ok((id, cursor + 2))
}

fn skip_ascii_whitespace(bytes: &[u8], cursor: &mut usize) {
    while bytes.get(*cursor).is_some_and(u8::is_ascii_whitespace) {
        *cursor += 1;
    }
}
