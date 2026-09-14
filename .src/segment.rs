//! One EDI segment — a tag and its data elements, each a list of components —
//! as the EDIFACT and X12 technologies both read it (ADR-0044). The separators
//! that cut a segment out of an interchange, and what a tag may look like, are
//! each syntax's own.

/// One segment: a tag and its elements, each a list of components.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Segment {
    pub tag: String,
    pub elements: Vec<Vec<String>>,
}

impl Segment {
    /// The components of element `index` (1-based, after the tag), or none.
    #[must_use]
    pub fn element(&self, index: usize) -> &[String] {
        index
            .checked_sub(1)
            .and_then(|at| self.elements.get(at))
            .map_or(&[], Vec::as_slice)
    }

    /// The first component of element `index`, or empty.
    #[must_use]
    pub fn simple(&self, index: usize) -> &str {
        self.element(index).first().map_or("", String::as_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn elements_are_counted_from_one_after_the_tag() {
        let segment = Segment {
            tag: "UNH".to_string(),
            elements: vec![
                vec!["1".to_string()],
                vec!["ORDERS".to_string(), "D".to_string()],
            ],
        };
        assert_eq!(segment.element(2), ["ORDERS", "D"]);
        assert_eq!(segment.simple(1), "1");
        assert_eq!(segment.simple(3), "");
        assert!(segment.element(0).is_empty());
    }
}
