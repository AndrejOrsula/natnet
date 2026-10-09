//! std TCP client for AUGI/1 (`imagery` + `net` features).
//!
//! 1. Dial with [`ImageClient::connect`], or adopt an accepted stream with
//!    [`ImageClient::from_stream`].
//! 2. Call [`ImageClient::recv_record`] in a loop and parse each record with
//!    [`parse_record`](crate::imagery::parse_record).
//!
//! **Not authenticated.** `connect` dials any address you give it. After the
//! handshake the client sends no application data. Choosing a safe peer is
//! yours.
//!
//! `std` cannot bind before connecting, so `connect` checks the local address
//! afterwards and fails with [`io::ErrorKind::AddrNotAvailable`] if it is not
//! the requested interface. [`Ipv4Addr::UNSPECIFIED`] skips the check.
//!
//! # Deadlines and records
//!
//! - Timeouts must be positive. One absolute deadline covers a whole record;
//!   arriving bytes do not extend it.
//! - Reads are non-blocking; an idle client polls with 50 µs to 1 ms backoff.
//! - Past the deadline the client makes at most one more read for the prefix
//!   and one for the body (up to 1 MiB), so data already queued can still
//!   complete the record. Otherwise it fails with `TimedOut`.
//! - Records over [`MAX_RECORD_LEN`] are refused. The buffer grows as data
//!   arrives (at most 1 MiB ahead), never from the declared length.
//!
//! `recv_record` returns `Ok(true)` for a record and `Ok(false)` for a clean
//! EOF between records. A timeout before any byte of a record is an idle poll:
//! the client stays usable. Any other failure (partial EOF, bad framing, a
//! timeout mid-record, I/O errors) shuts the stream down; later calls return
//! the same error. Framing does not check record order, bridge identity or
//! image contents.

use std::io::{self, ErrorKind, Read};
use std::net::{Ipv4Addr, Shutdown, SocketAddr, SocketAddrV4, TcpStream};
use std::time::{Duration, Instant};

use crate::Error as ImageryError;
use crate::imagery::{PREFIX_LEN, record_len};

/// Largest record [`ImageClient::recv_record`] will buffer, in bytes.
///
/// The wire permits 4 GiB payloads; this cap bounds memory use for received
/// records.
pub const MAX_RECORD_LEN: usize = 64 << 20;

#[derive(Debug)]
enum TerminalState {
    CleanEof,
    Failed {
        kind: ErrorKind,
        message: String,
        wire: Option<ImageryError>,
        raw_os_error: Option<i32>,
    },
}

impl TerminalState {
    /// The result every call after entering this state returns.
    fn replay(&self) -> io::Result<bool> {
        match self {
            Self::CleanEof => Ok(false),
            Self::Failed {
                kind,
                message,
                wire,
                raw_os_error,
            } => {
                let error = if let Some(wire) = wire {
                    io::Error::new(*kind, wire.clone())
                } else if let Some(code) = raw_os_error {
                    io::Error::from_raw_os_error(*code)
                } else {
                    io::Error::new(*kind, message.clone())
                };
                Err(error)
            }
        }
    }
}

/// One non-blocking read attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
    /// This many bytes were read (never zero).
    Read(usize),
    /// The peer closed the stream.
    Eof,
    /// Nothing was queued.
    Empty,
}

/// Longest sleep between two polls of an idle socket.
const POLL_MAX: Duration = Duration::from_millis(1);
/// First sleep after a poll found nothing queued.
const POLL_MIN: Duration = Duration::from_micros(50);
/// Smallest growth step of the record buffer before the deadline.
const GROW_MIN: usize = 64 * 1024;
/// Space the single post-deadline read is guaranteed, bounded by the record.
const LATE_READ: usize = 1 << 20;

/// End of the next read's window in a record buffer of `len` bytes, `filled`
/// of them received.
///
/// Past the deadline, earlier growth may have left more than [`LATE_READ`]
/// bytes of room; the single final read takes at most that, so post-deadline
/// work stays bounded however large the buffer has grown.
fn read_end(len: usize, filled: usize, late: bool) -> usize {
    if late {
        len.min(filled.saturating_add(LATE_READ))
    } else {
        len
    }
}

/// A receive-only AUGI/1 TCP client.
///
/// Pull whole records with [`ImageClient::recv_record`] and decode them with
/// [`parse_record`](crate::imagery::parse_record). The type exposes no send
/// method: after construction it transmits nothing but TCP acknowledgements.
#[derive(Debug)]
pub struct ImageClient {
    stream: TcpStream,
    terminal: Option<TerminalState>,
}

impl ImageClient {
    /// Connects to a bridge with a positive `connect_timeout`, verifying the
    /// kernel picked `interface` as the source address (pass
    /// [`Ipv4Addr::UNSPECIFIED`] to skip).
    ///
    /// The timeout bounds the connect only; each [`ImageClient::recv_record`]
    /// call supplies its own record timeout.
    ///
    /// # Errors
    ///
    /// Returns [`io::ErrorKind::InvalidInput`] for a zero timeout, propagates
    /// any [`io::Error`] from connecting or configuring the socket, and
    /// returns [`io::ErrorKind::AddrNotAvailable`] if the connection was
    /// routed out of an interface other than `interface`.
    pub fn connect(
        interface: Ipv4Addr,
        server: SocketAddrV4,
        connect_timeout: Duration,
    ) -> io::Result<Self> {
        validate_timeout(connect_timeout, "connect timeout")?;
        let stream = TcpStream::connect_timeout(&SocketAddr::V4(server), connect_timeout)?;
        if !interface.is_unspecified() {
            let chosen = stream.local_addr()?.ip();
            if chosen != interface {
                return Err(io::Error::new(
                    ErrorKind::AddrNotAvailable,
                    format!(
                        "route to {server} left via {chosen}, not the requested interface \
                         {interface} — fix the host route or pass the interface actually in use"
                    ),
                ));
            }
        }
        Self::from_stream(stream, connect_timeout)
    }

    /// Adopts an already-connected stream.
    ///
    /// The consumer may listen and accept the bridge's connection. The stream
    /// is switched to non-blocking mode. `record_timeout` must be positive;
    /// each receive call supplies its own deadline.
    ///
    /// # Errors
    ///
    /// Returns [`io::ErrorKind::InvalidInput`] for a zero timeout and
    /// propagates any socket error from changing the blocking mode.
    pub fn from_stream(stream: TcpStream, record_timeout: Duration) -> io::Result<Self> {
        validate_timeout(record_timeout, "record timeout")?;
        stream.set_nonblocking(true)?;
        Ok(Self {
            stream,
            terminal: None,
        })
    }

    /// The bridge's address.
    ///
    /// # Errors
    ///
    /// Propagates any [`io::Error`] from the socket.
    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        self.stream.peer_addr()
    }

    /// Whether this client has entered its terminal state.
    ///
    /// Terminal clients never read again; create a new client to reconnect.
    /// An idle [`io::ErrorKind::TimedOut`] leaves this `false`.
    #[must_use]
    pub fn is_terminal(&self) -> bool {
        self.terminal.is_some()
    }

    /// The local address the stream is bound to.
    ///
    /// # Errors
    ///
    /// Propagates any [`io::Error`] from the socket.
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.stream.local_addr()
    }

    /// Reads exactly one record into `buf`, replacing its contents on success.
    ///
    /// Returns `Ok(true)` for a complete record and `Ok(false)` for clean EOF
    /// before the next prefix byte. Clean EOF leaves `buf` unchanged. The
    /// positive timeout is an absolute deadline for the entire record. Reads
    /// poll non-blockingly; after the deadline, one read may consume bytes
    /// already queued. A timeout before any record byte is read is nonterminal
    /// and preserves `buf`; timeout within a record is terminal.
    ///
    /// The length is capped by [`MAX_RECORD_LEN`] and does not cause an
    /// up-front allocation.
    ///
    /// # Errors
    ///
    /// Returns [`io::ErrorKind::UnexpectedEof`] for partial EOF and
    /// [`io::ErrorKind::InvalidData`] for invalid framing or an oversized
    /// record. Terminal failures clear `buf` and shut down the stream; later
    /// calls replay the kind and message, preserving a NatNet source or OS
    /// code. Zero or unrepresentable timeouts return
    /// [`io::ErrorKind::InvalidInput`] before reading, leaving `buf` and the
    /// client unchanged.
    pub fn recv_record(&mut self, buf: &mut Vec<u8>, record_timeout: Duration) -> io::Result<bool> {
        validate_timeout(record_timeout, "record timeout")?;
        if let Some(terminal) = &self.terminal {
            return terminal.replay();
        }

        let deadline = Instant::now().checked_add(record_timeout).ok_or_else(|| {
            io::Error::new(
                ErrorKind::InvalidInput,
                "record timeout cannot be represented as an absolute deadline",
            )
        })?;

        // The prefix goes to a local array, so an idle timeout or a clean EOF
        // leaves the caller's buffer exactly as it was.
        let mut prefix = [0u8; PREFIX_LEN];
        let mut prefix_read = 0;
        let mut poll = Poll::new();
        while prefix_read < PREFIX_LEN {
            let late = Instant::now() >= deadline;
            match self.read_step(&mut prefix[prefix_read..]) {
                Ok(Step::Read(count)) => {
                    prefix_read += count;
                    if prefix_read < PREFIX_LEN && late {
                        buf.clear();
                        return self.fail(deadline_expired());
                    }
                    poll.reset();
                }
                Ok(Step::Eof) if prefix_read == 0 => {
                    self.terminal = Some(TerminalState::CleanEof);
                    return Ok(false);
                }
                Ok(Step::Eof) => {
                    buf.clear();
                    return self.fail(truncated(PREFIX_LEN, prefix_read));
                }
                Ok(Step::Empty) if late && prefix_read == 0 => {
                    // Nothing of the next record was consumed: the stream is
                    // still at a record boundary, so this is an idle poll.
                    return Err(io::Error::new(
                        ErrorKind::TimedOut,
                        "no record arrived before the deadline",
                    ));
                }
                Ok(Step::Empty) if late => {
                    buf.clear();
                    return self.fail(deadline_expired());
                }
                Ok(Step::Empty) => poll.wait(deadline),
                Err(error) => {
                    buf.clear();
                    return self.fail(error);
                }
            }
        }

        let total = match record_len(&prefix) {
            Ok(Some(total)) => total,
            Ok(None) => {
                buf.clear();
                return self.fail(io::Error::new(
                    ErrorKind::InvalidData,
                    ImageryError::UnexpectedEof {
                        offset: 0,
                        needed: PREFIX_LEN,
                        available: prefix_read,
                    },
                ));
            }
            Err(error) => {
                buf.clear();
                return self.fail(io::Error::new(ErrorKind::InvalidData, error));
            }
        };
        if total > MAX_RECORD_LEN {
            buf.clear();
            return self.fail(io::Error::new(
                ErrorKind::InvalidData,
                format!("record of {total} bytes exceeds MAX_RECORD_LEN ({MAX_RECORD_LEN})"),
            ));
        }

        // The prefix is complete, so it is safe to replace the previous
        // record in the caller-owned buffer. Any later failure clears this
        // incomplete record before entering the terminal state.
        buf.clear();
        buf.extend_from_slice(&prefix);
        self.read_body(buf, total, deadline, &mut poll)?;
        debug_assert_eq!(buf.len(), total);
        Ok(true)
    }

    /// Reads the rest of a record whose prefix is already in `buf`.
    ///
    /// `total` is a length the *peer* chose. It bounds the reads below and is
    /// never allocated up front: `buf[..filled]` holds received bytes and
    /// `buf[filled..]` is zeroed space for the next read, grown only when the
    /// received bytes have used it up (or, past the deadline, to give the
    /// single final read `LATE_READ` bytes of room). Every failure clears
    /// `buf` and enters the terminal state.
    fn read_body(
        &mut self,
        buf: &mut Vec<u8>,
        total: usize,
        deadline: Instant,
        poll: &mut Poll,
    ) -> io::Result<()> {
        let mut filled = buf.len();
        while filled < total {
            let late = Instant::now() >= deadline;
            let room = if late {
                LATE_READ
            } else {
                GROW_MIN.max(filled)
            };
            if buf.len() == filled || late {
                let want = filled.saturating_add(room).min(total);
                if buf.len() < want {
                    buf.resize(want, 0);
                }
            }
            let end = read_end(buf.len(), filled, late);
            match self.read_step(&mut buf[filled..end]) {
                Ok(Step::Read(count)) => {
                    filled += count;
                    if filled < total && late {
                        buf.clear();
                        return self.fail(deadline_expired());
                    }
                    poll.reset();
                }
                Ok(Step::Eof) => {
                    buf.clear();
                    return self.fail(truncated(total, filled));
                }
                Ok(Step::Empty) if late => {
                    buf.clear();
                    return self.fail(deadline_expired());
                }
                Ok(Step::Empty) => poll.wait(deadline),
                Err(error) => {
                    buf.clear();
                    return self.fail(error);
                }
            }
        }
        Ok(())
    }

    /// One non-blocking read. `destination` is never empty here, so `Ok(0)`
    /// always means end of stream. An interrupted read transferred nothing and
    /// is retried at once.
    fn read_step(&mut self, destination: &mut [u8]) -> io::Result<Step> {
        debug_assert_ne!(destination, []);
        loop {
            match self.stream.read(destination) {
                Ok(0) => return Ok(Step::Eof),
                Ok(count) => return Ok(Step::Read(count)),
                Err(error) if error.kind() == ErrorKind::Interrupted => {}
                Err(error) if error.kind() == ErrorKind::WouldBlock => return Ok(Step::Empty),
                Err(error) => return Err(error),
            }
        }
    }

    fn fail<T>(&mut self, error: io::Error) -> io::Result<T> {
        self.terminal = Some(TerminalState::Failed {
            kind: error.kind(),
            message: error.to_string(),
            wire: error
                .get_ref()
                .and_then(|source| source.downcast_ref::<ImageryError>())
                .cloned(),
            raw_os_error: error.raw_os_error(),
        });
        // Preserve the original receive failure even if shutdown also fails.
        let _ = self.stream.shutdown(Shutdown::Both);
        Err(error)
    }
}

fn validate_timeout(timeout: Duration, label: &str) -> io::Result<()> {
    if timeout.is_zero() {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            format!("{label} must be finite and greater than zero"),
        ));
    }
    Ok(())
}

/// Bounded backoff between polls of an idle socket.
struct Poll {
    next: Duration,
}

impl Poll {
    const fn new() -> Self {
        Self { next: POLL_MIN }
    }

    /// Restarts the backoff after progress.
    fn reset(&mut self) {
        self.next = POLL_MIN;
    }

    /// Sleeps for the current interval, never past `deadline`, and doubles
    /// the interval up to [`POLL_MAX`].
    fn wait(&mut self, deadline: Instant) {
        let left = deadline.saturating_duration_since(Instant::now());
        std::thread::sleep(self.next.min(left));
        self.next = (self.next * 2).min(POLL_MAX);
    }
}

fn deadline_expired() -> io::Error {
    io::Error::new(ErrorKind::TimedOut, "record deadline expired")
}

fn truncated(needed: usize, available: usize) -> io::Error {
    io::Error::new(
        ErrorKind::UnexpectedEof,
        ImageryError::UnexpectedEof {
            offset: 0,
            needed,
            available,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::{LATE_READ, read_end};

    #[test]
    fn the_final_read_is_capped_at_late_read_even_with_more_room() {
        let filled = 3 * LATE_READ;
        let len = 2 * filled;
        assert_eq!(read_end(len, filled, true), filled + LATE_READ);
        assert_eq!(read_end(len, filled, false), len);
    }

    #[test]
    fn the_final_read_never_reaches_past_the_buffer() {
        assert_eq!(read_end(10, 4, true), 10);
        assert_eq!(read_end(usize::MAX, usize::MAX - 1, true), usize::MAX);
    }
}
