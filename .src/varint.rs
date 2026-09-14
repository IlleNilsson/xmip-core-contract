//! Base-128 varints and the cursor that reads them, shared by the Avro and
//! protobuf technologies (ADR-0044): both walk a datum by its schema without
//! keeping what is read, and both begin every value with a varint. Avro's
//! zig-zag and protobuf's tag are each technology's own, built over this.

/// A cursor over encoded bytes.
pub struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    #[must_use]
    pub const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, at: 0 }
    }

    /// How far the cursor is.
    #[must_use]
    pub const fn position(&self) -> usize {
        self.at
    }

    /// Whether every byte was read.
    #[must_use]
    pub const fn is_done(&self) -> bool {
        self.at >= self.bytes.len()
    }

    /// The next `count` bytes, `what` naming them when they are not there.
    ///
    /// # Errors
    /// Fewer than `count` bytes remain.
    pub fn take(&mut self, count: usize, what: &str) -> Result<&'a [u8], String> {
        let end = self
            .at
            .checked_add(count)
            .filter(|end| *end <= self.bytes.len())
            .ok_or_else(|| format!("{what} runs past the end"))?;
        let slice = &self.bytes[self.at..end];
        self.at = end;
        Ok(slice)
    }

    /// One base-128 varint, at most ten bytes.
    ///
    /// # Errors
    /// A varint that does not terminate within ten bytes, or the end.
    pub fn varint(&mut self) -> Result<u64, String> {
        let mut value: u64 = 0;
        for shift in (0..70).step_by(7) {
            let byte = self.take(1, "a varint")?[0];
            if shift < 64 {
                value |= u64::from(byte & 0x7f) << shift;
            }
            if byte & 0x80 == 0 {
                return Ok(value);
            }
        }
        Err("a varint over ten bytes".to_string())
    }
}

/// `value` as a base-128 varint.
#[must_use]
pub fn encode(mut value: u64) -> Vec<u8> {
    let mut out = Vec::with_capacity(10);
    loop {
        let byte = u8::try_from(value & 0x7f).unwrap_or(0);
        value >>= 7;
        if value == 0 {
            out.push(byte);
            return out;
        }
        out.push(byte | 0x80);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn varints_round_trip_and_the_cursor_says_where_it_is() {
        for value in [0, 1, 127, 128, 300, u64::MAX] {
            let bytes = encode(value);
            let mut reader = Reader::new(&bytes);
            assert_eq!(reader.varint().expect("varint"), value, "{value}");
            assert!(reader.is_done());
            assert_eq!(reader.position(), bytes.len());
        }
        assert_eq!(encode(300), [0xac, 0x02]);
        assert_eq!(encode(u64::MAX).len(), 10);
    }

    #[test]
    fn what_runs_past_the_end_or_never_ends_is_refused() {
        assert!(Reader::new(&[0x80; 11]).varint().is_err(), "eleven bytes");
        assert!(Reader::new(&[0x80]).varint().is_err(), "cut off");
        let mut reader = Reader::new(&[1, 2, 3]);
        assert_eq!(reader.take(2, "a pair").expect("pair"), [1, 2]);
        let error = reader.take(2, "a pair").expect_err("short");
        assert_eq!(error, "a pair runs past the end");
        assert_eq!(reader.position(), 2);
        assert!(!reader.is_done());
    }
}
