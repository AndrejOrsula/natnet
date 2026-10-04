//! Bounds-checked little-endian [`Reader`] and append-only [`Writer`].
//! Truncated reads return [`Error::UnexpectedEof`].

use crate::error::{Error, Result};

/// A forward-only, bounds-checked cursor over a byte slice.
pub(crate) struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
    base: usize,
}

impl<'a> Reader<'a> {
    /// Wraps a slice at offset zero.
    pub(crate) fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            pos: 0,
            base: 0,
        }
    }

    /// Current read offset.
    pub(crate) fn position(&self) -> usize {
        self.base + self.pos
    }

    /// Bytes not yet consumed.
    pub(crate) fn remaining(&self) -> usize {
        self.data.len() - self.pos
    }

    /// Advances past `n` bytes, returning them, or fails if fewer remain.
    pub(crate) fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        let end = self.pos.checked_add(n).ok_or(Error::UnexpectedEof {
            offset: self.position(),
            needed: n,
            available: self.remaining(),
        })?;
        if end > self.data.len() {
            return Err(Error::UnexpectedEof {
                offset: self.position(),
                needed: n,
                available: self.remaining(),
            });
        }
        let slice = &self.data[self.pos..end];
        self.pos = end;
        Ok(slice)
    }

    /// Takes an exact region, preserving absolute payload offsets in errors.
    pub(crate) fn subreader(&mut self, len: usize) -> Result<Self> {
        let base = self.position();
        let data = self.take(len)?;
        Ok(Self { data, pos: 0, base })
    }

    /// Returns a previously consumed span, using absolute payload offsets.
    pub(crate) fn span(&self, start: usize, end: usize) -> Result<&'a [u8]> {
        let bounds = start.checked_sub(self.base).zip(end.checked_sub(self.base));
        if let Some((start, end)) = bounds
            && start <= end
            && end <= self.pos
        {
            return Ok(&self.data[start..end]);
        }
        Err(Error::InvalidValue {
            field: "reader span",
            reason: "span is outside the consumed region",
        })
    }

    /// Requires complete consumption of a bounded region.
    pub(crate) fn finish(&self) -> Result<()> {
        if self.remaining() == 0 {
            Ok(())
        } else {
            Err(Error::TrailingBytes {
                remaining: self.remaining(),
            })
        }
    }

    /// Reads a fixed-size array.
    pub(crate) fn array<const N: usize>(&mut self) -> Result<[u8; N]> {
        let slice = self.take(N)?;
        let mut out = [0u8; N];
        out.copy_from_slice(slice);
        Ok(out)
    }

    pub(crate) fn u8(&mut self) -> Result<u8> {
        Ok(self.array::<1>()?[0])
    }

    pub(crate) fn u16(&mut self) -> Result<u16> {
        Ok(u16::from_le_bytes(self.array()?))
    }

    pub(crate) fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.array()?))
    }

    pub(crate) fn i32(&mut self) -> Result<i32> {
        Ok(i32::from_le_bytes(self.array()?))
    }

    pub(crate) fn u64(&mut self) -> Result<u64> {
        Ok(u64::from_le_bytes(self.array()?))
    }

    pub(crate) fn f32(&mut self) -> Result<f32> {
        Ok(f32::from_le_bytes(self.array()?))
    }

    pub(crate) fn f64(&mut self) -> Result<f64> {
        Ok(f64::from_le_bytes(self.array()?))
    }

    pub(crate) fn vec3(&mut self) -> Result<[f32; 3]> {
        Ok([self.f32()?, self.f32()?, self.f32()?])
    }

    pub(crate) fn quat(&mut self) -> Result<[f32; 4]> {
        Ok([self.f32()?, self.f32()?, self.f32()?, self.f32()?])
    }

    /// Reads a NUL-terminated string, consuming the terminator.
    /// Replaces invalid UTF-8; returns [`Error::UnterminatedString`] if no
    /// terminator appears in the remaining buffer.
    pub(crate) fn cstr(&mut self) -> Result<String> {
        let start = self.position();
        let rel = self.data[self.pos..]
            .iter()
            .position(|&b| b == 0)
            .ok_or(Error::UnterminatedString { offset: start })?;
        let text = String::from_utf8_lossy(&self.data[self.pos..self.pos + rel]).into_owned();
        self.pos += rel + 1;
        Ok(text)
    }

    /// Reads `width` bytes, stopping at the first NUL; later bytes are ignored.
    pub(crate) fn fixed_cstr(&mut self, width: usize) -> Result<String> {
        let field = self.take(width)?;
        let end = field.iter().position(|&b| b == 0).unwrap_or(width);
        Ok(String::from_utf8_lossy(&field[..end]).into_owned())
    }

    /// Rejects counts exceeding remaining bytes at `min_entry_bytes` each.
    pub(crate) fn guard_count(&self, count: i32, min_entry_bytes: usize) -> Result<usize> {
        let count = usize::try_from(count).map_err(|_| Error::ImplausibleCount {
            count: count as usize,
            available: self.remaining(),
        })?;
        if let Some(max) = self.remaining().checked_div(min_entry_bytes)
            && count > max
        {
            return Err(Error::ImplausibleCount {
                count,
                available: self.remaining(),
            });
        }
        Ok(count)
    }
}

/// Maximum initial capacity for a vector sized from a wire count.
const MAX_PREALLOC: usize = 256;

/// Allocates capacity for at most [`MAX_PREALLOC`] entries; the vector grows
/// only with entries actually decoded.
pub(crate) fn vec_for_count<T>(count: usize) -> Vec<T> {
    Vec::with_capacity(count.min(MAX_PREALLOC))
}

/// An append-only little-endian byte sink used by the serializers.
#[derive(Default)]
pub(crate) struct Writer {
    buf: Vec<u8>,
}

impl Writer {
    pub(crate) fn new() -> Self {
        Self { buf: Vec::new() }
    }

    pub(crate) fn into_vec(self) -> Vec<u8> {
        self.buf
    }

    /// Bytes written so far.
    pub(crate) fn len(&self) -> usize {
        self.buf.len()
    }

    /// Overwrites four previously written bytes at `at` with `v`.
    ///
    /// # Panics
    /// If fewer than four bytes were written at `at`; callers patch only a
    /// placeholder they wrote themselves.
    pub(crate) fn patch_i32(&mut self, at: usize, v: i32) {
        self.buf[at..at + 4].copy_from_slice(&v.to_le_bytes());
    }

    pub(crate) fn bytes(&mut self, b: &[u8]) {
        self.buf.extend_from_slice(b);
    }

    pub(crate) fn u8(&mut self, v: u8) {
        self.buf.push(v);
    }

    pub(crate) fn u16(&mut self, v: u16) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub(crate) fn u32(&mut self, v: u32) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub(crate) fn i32(&mut self, v: i32) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub(crate) fn u64(&mut self, v: u64) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub(crate) fn f32(&mut self, v: f32) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub(crate) fn f64(&mut self, v: f64) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub(crate) fn vec3(&mut self, v: [f32; 3]) {
        for c in v {
            self.f32(c);
        }
    }

    pub(crate) fn quat(&mut self, v: [f32; 4]) {
        for c in v {
            self.f32(c);
        }
    }

    /// Writes a NUL-terminated string.
    pub(crate) fn cstr(&mut self, s: &str) {
        self.buf.extend_from_slice(s.as_bytes());
        self.buf.push(0);
    }

    /// Writes a checked NUL-terminated string without changing it.
    pub(crate) fn try_cstr(&mut self, s: &str, field: &'static str) -> Result<()> {
        validate_cstr(s, field)?;
        self.cstr(s);
        Ok(())
    }

    /// Writes a fixed field only when the full string and terminator fit.
    pub(crate) fn try_fixed_cstr(
        &mut self,
        s: &str,
        width: usize,
        field: &'static str,
    ) -> Result<()> {
        validate_cstr(s, field)?;
        let max = width.checked_sub(1).ok_or(Error::InvalidValue {
            field,
            reason: "a terminated string needs a nonempty field",
        })?;
        if s.len() > max {
            return Err(Error::LengthOverflow {
                field,
                length: s.len(),
                max,
            });
        }
        self.fixed_cstr(s, width);
        Ok(())
    }

    /// Writes a NUL-terminated, zero-padded field, truncating to `width - 1`
    /// bytes. See [`Reader::fixed_cstr`].
    pub(crate) fn fixed_cstr(&mut self, s: &str, width: usize) {
        let take = s.len().min(width.saturating_sub(1));
        self.buf.extend_from_slice(&s.as_bytes()[..take]);
        self.buf.resize(self.buf.len() + (width - take), 0);
    }
}

/// Validates a string intended for a NUL-terminated wire field.
pub(crate) fn validate_cstr(s: &str, field: &'static str) -> Result<()> {
    if let Some(offset) = s.as_bytes().iter().position(|&byte| byte == 0) {
        Err(Error::InteriorNul { field, offset })
    } else {
        Ok(())
    }
}

/// Converts an entry count or section size without saturating the wire field.
pub(crate) fn checked_i32(value: usize, field: &'static str) -> Result<i32> {
    i32::try_from(value).map_err(|_| Error::LengthOverflow {
        field,
        length: value,
        max: i32::MAX as usize,
    })
}
