//! Optional std-TCP transport for AUGI/1 (`imagery` + `net` features) — a
//! thin convenience layer over the sans-io [`imagery`](crate::imagery) core.
//!
//! # This client connects out. That is a deliberate departure.
//!
//! [`DataClient`](crate::net::DataClient) is type-enforced receive-only: the
//! only wire output a NatNet data subscription can cause is the kernel's IGMP
//! membership report. [`ImageClient::connect`] breaks that posture — it opens
//! an outbound TCP connection and the three-way handshake is unambiguously
//! transmitted traffic.
//!
//! The departure is justified because of **who is on the far end**. The
//! transmit ban exists so that subscribing to a stream can never perturb the
//! motion-capture system: nothing this crate sends may reach Motive's command
//! port. An [`ImageClient`] connects to `motive-bridge`, a separate service
//! on its own TCP port. It is not a command channel to Motive, it
//! carries no NatNet command, and this type exposes no send method at all —
//! once connected, it is as receive-only as [`DataClient`](crate::net::DataClient).
//!
//! [`ImageClient::from_stream`] avoids the departure entirely and is the
//! preferred posture where the network allows it: the consumer listens and
//! the bridge dials out (the bridge's `connect_to`/[`MODE_CONNECT`] mode,
//! which also spares the Windows host an inbound firewall rule). The caller
//! owns the listener and decides whether its first accepted peer is trusted;
//! this transport performs no peer authentication.
//!
//! # Interface binding
//!
//! A dual-homed workstation's default route may be the wrong one.
//! `std` has no bind-before-connect for TCP (that needs a raw socket API this
//! zero-dependency crate will not take on), so [`ImageClient::connect`] connects
//! and then **verifies** the kernel chose the requested source address, failing
//! loudly rather than streaming down the wrong interface. Pass
//! [`Ipv4Addr::UNSPECIFIED`] to skip the check.
//!
//! # Timeouts and failure lifecycle
//!
//! [`ImageClient::connect`] requires a positive connect timeout and uses
//! `TcpStream::connect_timeout`. [`ImageClient::from_stream`] requires a
//! positive record timeout argument and puts the adopted socket into the
//! non-blocking mode the client relies on, whatever mode it inherited. Each
//! [`ImageClient::recv_record`] call requires its own positive record timeout.
//! The timeout is an **absolute deadline for the whole record**, including all
//! prefix, header, and payload bytes; receiving a byte does not restart it,
//! and a peer still sending when it passes does not buy more time with what it
//! sends next.
//!
//! **Waiting never uses a socket receive timeout.** Every read is
//! non-blocking; while nothing is queued the client sleeps for at most 1 ms
//! (starting at 50 µs, doubling while idle) and polls again until the
//! deadline. Microsoft documents for `SO_RCVTIMEO` that "if a blocking receive
//! call times out, the connection is in an indeterminate state and should be
//! closed", so a connection that is to survive an idle timeout cannot be
//! bounded that way. The cost is up to one poll interval of added latency
//! (longer where the platform's sleep granularity is coarser) and periodic
//! wake-ups while a call waits.
//!
//! The deadline bounds *waiting*, not reading: once it has passed, what is
//! left of the prefix or of the body is taken with a **single non-blocking
//! read** of up to the larger of 1 MiB and the space already allocated for the
//! record, so a record whose outstanding bytes are already queued and fit that
//! read is delivered whatever its kind — including when the deadline falls
//! between the read that collects the prefix and the read that collects the
//! payload. A read returning fewer bytes than it asked for has emptied the
//! receive queue, so a second read could only collect bytes that arrived after
//! the deadline; there is no second read, and a peer cannot lengthen the call
//! by continuing to send. A record whose remainder at the deadline is longer
//! than that one read, or longer than the receive queue holds at once, is not
//! completed late.
//!
//! **What a peer can make the client allocate is what it sends.** The declared
//! record length is checked against [`MAX_RECORD_LEN`] and then used only as a
//! bound: the destination buffer grows in chunks as bytes arrive (at least
//! 64 KiB, at most doubling what has been received), and past the deadline by
//! at most the 1 MiB final read. A twelve-byte prefix declaring 64 MiB costs
//! at most that final read's zero-fill, never the declared length.
//!
//! A successful call returns `Ok(true)` and replaces the caller's buffer with
//! one complete record. A clean close before any byte of the next prefix
//! returns `Ok(false)` and leaves the caller's buffer unchanged. A deadline
//! that passes before any byte of the next record has arrived returns
//! [`io::ErrorKind::TimedOut`] **without** ending the session: the buffer is
//! unchanged, [`ImageClient::is_terminal`] stays `false`, and the next call
//! waits again. A partial close, framing error, length-cap violation, a
//! timeout after any byte of a record was consumed, or other I/O error clears
//! the incomplete buffer, records the failure, makes the client terminal and
//! shuts down the connection. Further valid calls retain the error
//! kind/message, NatNet error source or OS error code, and never read the
//! stream; create a new client to reconnect. A `Bye` record is an ordinary
//! successful record, so a caller can stop at it without confusing protocol
//! shutdown with transport EOF. Record framing does not validate Hello
//! ordering, bridge identity, camera dimensions, or codec semantics; the
//! consumer owns those session checks. Zero or unrepresentable timeout
//! arguments fail before reading and leave both the buffer and client state
//! unchanged.
//!
//! [`MODE_CONNECT`]: crate::imagery::MODE_CONNECT

use std::io::{self, ErrorKind, Read};
use std::net::{Ipv4Addr, Shutdown, SocketAddr, SocketAddrV4, TcpStream};
use std::time::{Duration, Instant};

use crate::Error as ImageryError;
use crate::imagery::{PREFIX_LEN, record_len};

/// Largest record [`ImageClient::recv_record`] will buffer, in bytes.
///
/// The wire allows a 4 GiB payload. A 640x512 Mono8 frame is 320 kB and a
/// 33x33 ray grid is 26 kB, so 64 MiB is generous by two orders of magnitude
/// while still refusing to allocate on a corrupt or hostile length.
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
    /// See the module docs for why this outbound connect is an accepted
    /// departure from the crate's receive-only transmit posture.
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
    /// This is the reverse-mode posture, where the consumer listens and the
    /// bridge dials out. Nothing is transmitted by this constructor. The
    /// stream is switched to non-blocking mode, which every
    /// [`ImageClient::recv_record`] wait relies on (see the module docs), so
    /// a socket that inherited either mode from its listener behaves the same.
    /// `record_timeout` must be positive; it is validated for compatibility
    /// and each receive call supplies its own deadline.
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
    /// A terminal client (clean EOF, a partial record, a framing error, a
    /// timeout after any byte of a record was consumed, or another I/O
    /// failure) never reads the stream again; create a new client to
    /// reconnect. An idle [`io::ErrorKind::TimedOut`] — the deadline passed
    /// before any byte of the next record arrived — leaves this `false`, and
    /// the next [`ImageClient::recv_record`] waits again.
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
    /// caller must supply a positive timeout for each operation; the deadline
    /// covers the complete record and cannot be extended by fragmented or
    /// slow delivery, nor by a peer still sending when it passes. Waiting is
    /// non-blocking polling against that deadline (at most 1 ms between
    /// polls); no socket receive timeout is used.
    ///
    /// The deadline bounds *waiting*, not reading. Once it has passed, the
    /// prefix and the body are each finished with a **single non-blocking
    /// read** of what the receive queue already holds, sized to at least the
    /// smaller of the record's remainder and 1 MiB. A record whose outstanding
    /// bytes are all queued and fit that read is delivered, including when the
    /// deadline falls between the prefix read and the payload read; a record
    /// that read cannot finish fails with [`io::ErrorKind::TimedOut`].
    ///
    /// If the deadline passes before **any** byte of the next record has been
    /// read, the call returns [`io::ErrorKind::TimedOut`] without ending the
    /// session: `buf` is unchanged, [`ImageClient::is_terminal`] stays
    /// `false`, and the caller may call again. A timeout after any byte of a
    /// record was consumed is terminal, because the stream position is then
    /// inside a record.
    ///
    /// The declared record length is capped by [`MAX_RECORD_LEN`] and never
    /// allocated up front: `buf` grows in bounded chunks as bytes arrive.
    ///
    /// # Errors
    ///
    /// Returns [`io::ErrorKind::UnexpectedEof`] when EOF follows any prefix,
    /// header, or payload byte. Returns [`io::ErrorKind::InvalidData`] for a
    /// bad magic, impossible length, or a record longer than
    /// [`MAX_RECORD_LEN`]. Returns [`io::ErrorKind::TimedOut`] for an idle
    /// deadline (not terminal, `buf` unchanged) or for a deadline that passes
    /// inside a record (terminal). Other I/O failures preserve their original
    /// [`io::Error`] on the first call. Terminal failures clear `buf` and shut
    /// down this connection; later calls with valid timeouts retain the error
    /// kind/message and structured NatNet source or OS code. Zero or
    /// unrepresentable timeout arguments return `InvalidInput` before reading
    /// and leave the buffer and client state unchanged.
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
            match self.read_step(&mut buf[filled..]) {
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
