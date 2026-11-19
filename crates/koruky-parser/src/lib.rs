//! Parser for KORUKY's command language.

use koruky_core::EpisodeRequest;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ParseError {
    #[error("command is empty")]
    Empty,
}

/// Parse a basic anime search request.
pub fn parse_search(input: &str) -> Result<EpisodeRequest, ParseError> {
    let title = input.trim();

    if title.is_empty() {
        return Err(ParseError::Empty);
    }

    Ok(EpisodeRequest {
        title: title.to_owned(),
        season: None,
        episode: None,
        audio_language: None,
        subtitle_language: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_title() {
        let request = parse_search("  Frieren  ").expect("valid title");

        assert_eq!(request.title, "Frieren");
    }

    #[test]
    fn rejects_empty_input() {
        assert_eq!(parse_search("  "), Err(ParseError::Empty));
    }
}
