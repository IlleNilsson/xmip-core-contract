//! The Stream a technology's tests read: text as bytes, with or without a
//! media type, under one `StreamId`. Every technology's tests built this
//! themselves until ADR-0044 moved it here; it sits behind the `test-support`
//! feature so a technology enables it from its dev-dependencies and ships
//! nothing of it.

use stream::Stream;
use xcore::StreamId;

/// `text` as a Stream with no media type.
#[must_use]
pub fn stream(text: &str) -> Stream {
    stream_as(text, None)
}

/// `text` as a Stream with `media_type`, when one is given.
#[must_use]
pub fn stream_as(text: &str, media_type: Option<&str>) -> Stream {
    Stream::new(
        StreamId::new(1),
        text.as_bytes().to_vec(),
        media_type.map(str::to_string),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fixture_carries_the_text_and_the_media_type_it_was_given() {
        let bare = stream("x");
        assert_eq!(bare.bytes(), b"x");
        assert_eq!(bare.media_type(), None);
        let typed = stream_as("{}", Some("application/json"));
        assert_eq!(typed.bytes(), b"{}");
        assert_eq!(typed.media_type(), Some("application/json"));
    }
}
