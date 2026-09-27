//! Where in content an issue is, kept as a chain of steps on the stack and
//! spelled out only when an issue is raised: a document that holds costs no
//! path at all.
//!
//! A technology walks its content with a [`Place`], one step per level, and
//! writes the place in its representation's own form: a JSON document as a
//! JSON Pointer (RFC 6901, [`Place::pointer`]), a binary datum by its field
//! names (`order.lines[2]`, [`Place::dotted`]), an XML document as an
//! `XPath` (`/order/line[2]/@currency`, [`Place::xpath`]). The escape a JSON
//! Pointer token needs is [`escape`], for a caller that writes one itself.

use std::borrow::Cow;
use std::fmt::Write;

/// One place in content: the root, or one step below another place.
#[derive(Clone, Copy, Debug)]
pub enum Place<'a> {
    /// The whole content.
    Root,
    /// A named member: an object's key, a record's field, an element.
    Field(&'a Place<'a>, &'a str),
    /// A position among items, as the representation counts them.
    Index(&'a Place<'a>, usize),
    /// A map entry, by its key.
    Entry(&'a Place<'a>, &'a str),
    /// An XML attribute.
    Attribute(&'a Place<'a>, &'a str),
}

impl<'a> Place<'a> {
    /// The member `name` of this place.
    #[must_use]
    pub const fn field(&'a self, name: &'a str) -> Self {
        Self::Field(self, name)
    }

    /// Item `index` of this place.
    #[must_use]
    pub const fn index(&'a self, index: usize) -> Self {
        Self::Index(self, index)
    }

    /// The map entry `key` of this place.
    #[must_use]
    pub const fn entry(&'a self, key: &'a str) -> Self {
        Self::Entry(self, key)
    }

    /// The attribute `name` of this place.
    #[must_use]
    pub const fn attribute(&'a self, name: &'a str) -> Self {
        Self::Attribute(self, name)
    }

    /// Every step from the root down to this place, root first.
    fn steps(&self) -> Vec<Self> {
        let mut steps = Vec::new();
        let mut here = *self;
        loop {
            let parent = match here {
                Self::Root => break,
                Self::Field(parent, _)
                | Self::Index(parent, _)
                | Self::Entry(parent, _)
                | Self::Attribute(parent, _) => *parent,
            };
            steps.push(here);
            here = parent;
        }
        steps.reverse();
        steps
    }

    /// This place as a JSON Pointer (RFC 6901): `/lines/0/qty`, and the
    /// empty pointer for the whole document.
    #[must_use]
    pub fn pointer(&self) -> String {
        let mut out = String::new();
        for step in self.steps() {
            out.push('/');
            match step {
                Self::Root => {}
                Self::Field(_, name) | Self::Entry(_, name) | Self::Attribute(_, name) => {
                    out.push_str(&escape(name));
                }
                Self::Index(_, index) => {
                    let _ = write!(out, "{index}");
                }
            }
        }
        out
    }

    /// This place by field names: `order.lines[2]["sku"]`.
    #[must_use]
    pub fn dotted(&self) -> String {
        let mut out = String::new();
        for step in self.steps() {
            match step {
                Self::Root => {}
                Self::Field(_, name) | Self::Attribute(_, name) => {
                    if !out.is_empty() {
                        out.push('.');
                    }
                    out.push_str(name);
                }
                Self::Index(_, index) => {
                    let _ = write!(out, "[{index}]");
                }
                Self::Entry(_, key) => {
                    let _ = write!(out, "[{key:?}]");
                }
            }
        }
        out
    }

    /// This place as an `XPath`: `/order/line[2]/@currency`.
    #[must_use]
    pub fn xpath(&self) -> String {
        let mut out = String::new();
        for step in self.steps() {
            match step {
                Self::Root => {}
                Self::Field(_, name) | Self::Entry(_, name) => {
                    out.push('/');
                    out.push_str(name);
                }
                Self::Index(_, index) => {
                    let _ = write!(out, "[{index}]");
                }
                Self::Attribute(_, name) => {
                    out.push_str("/@");
                    out.push_str(name);
                }
            }
        }
        if out.is_empty() {
            out.push('/');
        }
        out
    }
}

/// One JSON Pointer reference token as RFC 6901 writes it: `~` as `~0`, `/`
/// as `~1`. Borrowed when there is nothing to escape.
#[must_use]
pub fn escape(token: &str) -> Cow<'_, str> {
    if token.contains(['~', '/']) {
        Cow::Owned(token.replace('~', "~0").replace('/', "~1"))
    } else {
        Cow::Borrowed(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_place_is_spelled_in_each_form() {
        let root = Place::Root;
        let order = root.field("order");
        let line = order.field("line");
        let second = line.index(2);
        let currency = second.attribute("currency");
        assert_eq!(currency.xpath(), "/order/line[2]/@currency");
        assert_eq!(second.dotted(), "order.line[2]");
        assert_eq!(second.pointer(), "/order/line/2");
        let sku = second.entry("sku");
        assert_eq!(sku.dotted(), "order.line[2][\"sku\"]");
    }

    #[test]
    fn the_root_is_the_empty_pointer_and_a_token_is_escaped() {
        assert_eq!(Place::Root.pointer(), "");
        assert_eq!(Place::Root.xpath(), "/");
        assert_eq!(Place::Root.dotted(), "");
        let root = Place::Root;
        assert_eq!(root.field("a/b~c").pointer(), "/a~1b~0c");
        assert!(matches!(escape("plain"), Cow::Borrowed("plain")));
        assert_eq!(escape("~1"), "~01");
    }
}
