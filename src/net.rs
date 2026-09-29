//! Optional standard-library UDP transport (`net` feature).
//!
//! [`DataClient`] is receive-only. [`CommandClient`] is the bounded, typed
//! command path: it accepts only validated [`crate::command::Request`] values,
//! owns its socket transaction mutably, checks the configured peer, and uses
//! an absolute deadline. [`RawCommandClient`] is the explicit escape hatch for
//! callers that need unrestricted datagrams or socket controls.
//!
//! Peer filtering is a transport boundary, not authentication. Before each
//! request is sent, the command client discards every datagram already queued
//! on its socket, so a reply that arrived after an earlier transaction's
//! deadline cannot answer the next request. A late datagram from the
//! configured peer that arrives *after* that drain still cannot be
//! distinguished from a fresh response, because NatNet command replies have no
//! transaction identifier. Callers must treat a failed write exchange as an
//! unknown remote outcome and decide how to reconcile it from their own
//! application state.

use std::fmt;
use std::io;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use std::time::{Duration, Instant};

use crate::command::{
    self, CommandResponse, ConnectOptions, Request, RequestKind, ResponseKind, WriteTier,
};
use crate::error::Error as NatNetError;
use crate::message::{MessageHeader, NAT_FRAMEOFDATA, NAT_MODELDEF, NAT_RESPONSE, NAT_SERVERINFO};
use crate::serverinfo::ServerInfo;
use crate::version::Version;

/// Largest datagram accepted by the safe command receive path.
pub const MAX_COMMAND_DATAGRAM: usize = 65_535;

/// Largest IPv4 UDP payload. NatNet's `u16` header field is wider than this
/// transport limit, so a request can fit its NatNet header and still be
/// refused before socket I/O.
pub const MAX_IPV4_UDP_PAYLOAD: usize = 65_507;

/// Smallest [`DataClient::recv`] buffer that can hold every IPv4 UDP datagram
/// ([`MAX_IPV4_UDP_PAYLOAD`], 65 507 bytes).
///
/// A datagram longer than the receive buffer is truncated by the operating
/// system without a portable indication. With a buffer of at least this size
/// truncation cannot happen; with a shorter buffer, a datagram that fills it
/// exactly may have been truncated and is reported as an error.
pub const MIN_DATA_BUFFER: usize = MAX_IPV4_UDP_PAYLOAD;

/// NAT_UNRECOGNIZED_REQUEST, re-exported here for existing callers.
/// [`CommandClient`] ends a transaction on this reply from the configured peer
/// with [`io::ErrorKind::Unsupported`].
pub use crate::message::NAT_UNRECOGNIZED_REQUEST;

/// A receive-only NatNet data-channel UDP client.
///
/// # Socket options
///
/// [`DataClient::bind_multicast`] and [`DataClient::bind_unicast`] use
/// `std::net::UdpSocket::bind`, which can set neither `SO_REUSEADDR` nor
/// `SO_RCVBUF`, and this crate takes no socket dependency. Two consumers on
/// one host binding the same data port therefore get `AddrInUse`, and the
/// receive buffer is the operating-system default. The NatNet SDK 4.1
/// `PacketClient` sample sets `SO_REUSEADDR` on its data socket and requests
/// `SO_RCVBUF` of `0x10000` bytes (its comment says 1 MB); the SDK Python
/// client sets `SO_REUSEADDR`. A caller that needs either option creates and
/// configures the socket itself (for example with `socket2`), binds it,
/// joins any multicast group, and adopts it with [`DataClient::from_socket`].
#[derive(Debug)]
pub struct DataClient {
    socket: UdpSocket,
}

impl DataClient {
    /// Joins `group` on the explicitly selected local interface.
    ///
    /// The socket binds to `0.0.0.0:port` so multicast datagrams are
    /// deliverable. Joining can cause the operating system's IGMP membership
    /// report; this type has no NatNet datagram-send operation.
    ///
    /// # Errors
    ///
    /// Returns the operating-system error from binding or joining the group.
    pub fn bind_multicast(interface: Ipv4Addr, group: Ipv4Addr, port: u16) -> io::Result<Self> {
        let socket = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port))?;
        socket.join_multicast_v4(&group, &interface)?;
        Ok(Self { socket })
    }

    /// Binds a unicast data socket to an explicitly selected local interface.
    ///
    /// # Errors
    ///
    /// Returns the operating-system error from binding the socket.
    pub fn bind_unicast(interface: Ipv4Addr, port: u16) -> io::Result<Self> {
        let socket = UdpSocket::bind(SocketAddrV4::new(interface, port))?;
        Ok(Self { socket })
    }

    /// Adopts a socket the caller has already created, configured, bound and,
    /// for multicast, joined to its group.
    ///
    /// This is the seam for socket options `std` cannot set, such as
    /// `SO_REUSEADDR` and `SO_RCVBUF` (see [the type docs](DataClient)).
    /// Nothing is transmitted, and the socket's options and blocking mode are
    /// left as the caller set them. The receive-only guarantee covers this
    /// type's API: a clone of the socket the caller kept is outside it.
    #[must_use]
    pub fn from_socket(socket: UdpSocket) -> Self {
        Self { socket }
    }

    /// Puts the receive socket into (non-)blocking mode.
    ///
    /// # Errors
    ///
    /// Returns the operating-system error from changing the socket mode.
    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        self.socket.set_nonblocking(nonblocking)
    }

    /// Sets the data receive timeout. `None` restores an indefinite wait.
    ///
    /// # Errors
    ///
    /// Returns the operating-system error from changing the timeout.
    pub fn set_read_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
        self.socket.set_read_timeout(timeout)
    }

    /// Returns the local address selected for the data socket.
    ///
    /// # Errors
    ///
    /// Returns the operating-system error from querying the socket.
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.socket.local_addr()
    }

    /// Receives one datagram and preserves its source address.
    ///
    /// Pass a buffer of at least [`MIN_DATA_BUFFER`] bytes. A shorter buffer
    /// still works for datagrams shorter than it, but a datagram that fills it
    /// exactly may have been truncated by the operating system, so that case
    /// is returned as an error rather than as possibly partial bytes.
    ///
    /// # Errors
    ///
    /// Returns [`io::ErrorKind::InvalidData`] when the received datagram fills
    /// a buffer shorter than [`MIN_DATA_BUFFER`] (the datagram is consumed and
    /// cannot be recovered; retry with a larger buffer for the next one).
    /// Otherwise returns the operating-system receive error, including
    /// `WouldBlock` or `TimedOut` when configured by the caller.
    pub fn recv(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        let (received, source) = self.socket.recv_from(buf)?;
        if received == buf.len() && buf.len() < MIN_DATA_BUFFER {
            return Err(invalid_data(format_args!(
                "datagram from {source} filled the {received}-byte receive buffer and may be \
                 truncated; use a buffer of at least MIN_DATA_BUFFER ({MIN_DATA_BUFFER}) bytes"
            )));
        }
        Ok((received, source))
    }
}

/// An unrestricted command-channel UDP client.
///
/// Raw operations intentionally live on this distinct type. The safe client
/// can be converted into it only by consuming the idle client with
/// [`CommandClient::into_raw`]. A raw client can transmit arbitrary bytes,
/// receive from any peer, and change socket options; those operations carry no
/// command validation or transaction guarantee.
#[derive(Debug)]
pub struct RawCommandClient {
    socket: UdpSocket,
    server: SocketAddr,
}

impl RawCommandClient {
    /// Binds an unrestricted command socket to `interface` and records the
    /// default destination `server`.
    ///
    /// # Errors
    ///
    /// Returns the operating-system error from binding the socket.
    pub fn bind(interface: Ipv4Addr, server: SocketAddrV4) -> io::Result<Self> {
        let socket = UdpSocket::bind(SocketAddrV4::new(interface, 0))?;
        Ok(Self {
            socket,
            server: SocketAddr::V4(server),
        })
    }

    /// The configured default destination.
    #[must_use]
    pub fn server(&self) -> SocketAddr {
        self.server
    }

    /// The local address selected for this socket.
    ///
    /// # Errors
    ///
    /// Returns the operating-system error from querying the socket.
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.socket.local_addr()
    }

    /// Sends arbitrary bytes to the configured default destination.
    ///
    /// # Errors
    ///
    /// Returns the operating-system send error.
    pub fn send(&self, datagram: &[u8]) -> io::Result<usize> {
        self.socket.send_to(datagram, self.server)
    }

    /// Sends arbitrary bytes to any destination.
    ///
    /// # Errors
    ///
    /// Returns the operating-system send error.
    pub fn send_to(&self, datagram: &[u8], destination: SocketAddr) -> io::Result<usize> {
        self.socket.send_to(datagram, destination)
    }

    /// Receives arbitrary bytes and returns their source address.
    ///
    /// # Errors
    ///
    /// Returns the operating-system receive error.
    pub fn recv(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        self.socket.recv_from(buf)
    }

    /// Changes raw receive blocking behavior.
    ///
    /// # Errors
    ///
    /// Returns the operating-system error from changing the socket mode.
    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        self.socket.set_nonblocking(nonblocking)
    }

    /// Changes the raw receive timeout.
    ///
    /// # Errors
    ///
    /// Returns the operating-system error from changing the timeout.
    pub fn set_read_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
        self.socket.set_read_timeout(timeout)
    }

    /// Changes the raw send timeout.
    ///
    /// # Errors
    ///
    /// Returns the operating-system error from changing the timeout.
    pub fn set_write_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
        self.socket.set_write_timeout(timeout)
    }

    /// Reads the current raw receive timeout.
    ///
    /// # Errors
    ///
    /// Returns the operating-system error from querying the timeout.
    pub fn read_timeout(&self) -> io::Result<Option<Duration>> {
        self.socket.read_timeout()
    }

    /// Reads the current raw send timeout.
    ///
    /// # Errors
    ///
    /// Returns the operating-system error from querying the timeout.
    pub fn write_timeout(&self) -> io::Result<Option<Duration>> {
        self.socket.write_timeout()
    }
}

/// A bounded command-channel UDP client.
///
/// The configured `server` is checked by the transport before a datagram is
/// accepted as a reply. Each transaction also filters message IDs and uses an
/// absolute deadline, so unrelated same-peer traffic cannot extend a wait.
///
/// The safe client owns receive and timeout operations for the duration of a
/// transaction. Those escape hatches are intentionally absent:
///
/// ```compile_fail
/// use natnet::net::CommandClient;
/// use std::net::{Ipv4Addr, SocketAddrV4};
///
/// let client = match CommandClient::bind(
///     Ipv4Addr::LOCALHOST,
///     SocketAddrV4::new(Ipv4Addr::LOCALHOST, 1510),
/// ) {
///     Ok(client) => client,
///     Err(_) => return,
/// };
/// let mut bytes = [0u8; 64];
/// client.recv(&mut bytes);
/// ```
///
/// ```compile_fail
/// use natnet::net::CommandClient;
/// use std::net::{Ipv4Addr, SocketAddrV4};
///
/// let client = match CommandClient::bind(
///     Ipv4Addr::LOCALHOST,
///     SocketAddrV4::new(Ipv4Addr::LOCALHOST, 1510),
/// ) {
///     Ok(client) => client,
///     Err(_) => return,
/// };
/// client.set_read_timeout(None);
/// ```
///
/// ```compile_fail
/// use natnet::net::CommandClient;
/// use std::net::{Ipv4Addr, SocketAddrV4};
///
/// let client = match CommandClient::bind(
///     Ipv4Addr::LOCALHOST,
///     SocketAddrV4::new(Ipv4Addr::LOCALHOST, 1510),
/// ) {
///     Ok(client) => client,
///     Err(_) => return,
/// };
/// let _socket = client.socket;
/// ```
///
/// A live mutable borrow prevents a second operation from using the same
/// client while that transaction is still held:
///
/// ```compile_fail
/// use natnet::net::CommandClient;
/// use std::net::{Ipv4Addr, SocketAddrV4};
///
/// let mut client = match CommandClient::bind(
///     Ipv4Addr::LOCALHOST,
///     SocketAddrV4::new(Ipv4Addr::LOCALHOST, 1510),
/// ) {
///     Ok(client) => client,
///     Err(_) => return,
/// };
/// let active = &mut client;
/// let _second = client.local_addr();
/// let _ = active.local_addr();
/// ```
///
/// Converting to the raw API consumes the safe client, so it cannot be reused
/// after that boundary:
///
/// ```compile_fail
/// use natnet::net::CommandClient;
/// use std::net::{Ipv4Addr, SocketAddrV4};
///
/// let client = match CommandClient::bind(
///     Ipv4Addr::LOCALHOST,
///     SocketAddrV4::new(Ipv4Addr::LOCALHOST, 1510),
/// ) {
///     Ok(client) => client,
///     Err(_) => return,
/// };
/// let _raw = client.into_raw();
/// let _ = client.local_addr();
/// ```
#[derive(Debug)]
pub struct CommandClient {
    socket: UdpSocket,
    server: SocketAddr,
}

impl CommandClient {
    /// Binds an explicit local interface and records the configured server IP
    /// and port for transport-owned peer filtering.
    ///
    /// # Errors
    ///
    /// Returns the operating-system error from binding or connecting the
    /// socket.
    pub fn bind(interface: Ipv4Addr, server: SocketAddrV4) -> io::Result<Self> {
        let socket = UdpSocket::bind(SocketAddrV4::new(interface, 0))?;
        let server = SocketAddr::V4(server);
        Ok(Self { socket, server })
    }

    /// The configured command peer.
    #[must_use]
    pub fn server(&self) -> SocketAddr {
        self.server
    }

    /// The local address selected for this client.
    ///
    /// # Errors
    ///
    /// Returns the operating-system error from querying the socket.
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.socket.local_addr()
    }

    /// Consumes the idle safe client and transfers its socket to the
    /// unrestricted raw API.
    #[must_use]
    pub fn into_raw(self) -> RawCommandClient {
        RawCommandClient {
            socket: self.socket,
            server: self.server,
        }
    }

    /// Sends one validated command request and receives its matching reply.
    ///
    /// The receive buffer is owned by this operation and can hold every valid
    /// UDP datagram. The caller cannot silently accept a truncated response by
    /// supplying a short buffer. A zero duration, deadline overflow, an IPv4
    /// request larger than the UDP payload limit, or a transport failure is
    /// returned before a successful exchange is reported.
    ///
    /// Datagrams already queued when the call starts are discarded before the
    /// request is sent, so a reply to an earlier, timed-out request cannot be
    /// returned here. A same-peer datagram arriving after the send is still
    /// accepted if its message id matches (see the module docs).
    ///
    /// # Errors
    ///
    /// Returns `InvalidInput` for an invalid timeout or oversized request;
    /// also returns `InvalidInput` when a Tier-1 SetProperty is supplied
    /// outside [`CommandClient::execute_tier1`];
    /// `TimedOut` when the absolute deadline expires, on every platform
    /// (including while discarding stale datagrams before the send);
    /// `Unsupported` as soon as the configured peer replies
    /// [`NAT_UNRECOGNIZED_REQUEST`]; `InvalidData` for a malformed matching
    /// reply; and the underlying socket error for send, receive, or
    /// option-restore failures.
    pub fn exchange(&mut self, request: &Request, timeout: Duration) -> io::Result<Vec<u8>> {
        if request.kind() == RequestKind::SetProperty && request.tier() == Some(WriteTier::Tier1) {
            return Err(invalid_input(
                "Tier-1 SetProperty must run through execute_tier1",
            ));
        }
        self.exchange_inner(request, timeout)
    }

    /// Alias for [`CommandClient::exchange`] using the typed request boundary.
    ///
    /// # Errors
    ///
    /// Propagates the errors documented for [`CommandClient::exchange`].
    pub fn transact(&mut self, request: &Request, timeout: Duration) -> io::Result<Vec<u8>> {
        self.exchange(request, timeout)
    }

    /// A typed-send spelling retained for callers migrating from the old
    /// arbitrary-byte `send` method. It accepts no raw byte slice.
    ///
    /// # Errors
    ///
    /// Propagates the errors documented for [`CommandClient::exchange`].
    pub fn send(&mut self, request: &Request, timeout: Duration) -> io::Result<Vec<u8>> {
        self.exchange(request, timeout)
    }

    /// Performs a bounded NAT_CONNECT exchange and validates the negotiated
    /// protocol layout. Unsupported SERVERINFO is returned with the decoded
    /// value attached for diagnostics.
    ///
    /// # Errors
    ///
    /// Returns a transport, decode, request-construction, or unsupported
    /// protocol error. Unsupported replies retain their decoded SERVERINFO.
    pub fn connect_handshake(&mut self, timeout: Duration) -> Result<ServerInfo, HandshakeError> {
        let request = command::connect_request().map_err(HandshakeError::Command)?;
        self.handshake(&request, timeout, |info| info.protocol_version)
    }

    /// Performs a bounded NAT_CONNECT exchange carrying `options` and
    /// validates the layout the server will stream,
    /// [`ConnectOptions::stream_version`], rather than the server's own
    /// version. Request a version only with unicast data; see
    /// [`ConnectOptions`].
    ///
    /// # Errors
    ///
    /// As [`CommandClient::connect_handshake`].
    pub fn connect_handshake_with(
        &mut self,
        options: ConnectOptions,
        timeout: Duration,
    ) -> Result<ServerInfo, HandshakeError> {
        let request = command::connect_request_with(options).map_err(HandshakeError::Command)?;
        self.handshake(&request, timeout, |info| options.stream_version(info))
    }

    fn handshake(
        &mut self,
        request: &Request,
        timeout: Duration,
        layout: impl FnOnce(&ServerInfo) -> Version,
    ) -> Result<ServerInfo, HandshakeError> {
        let datagram = self
            .exchange(request, timeout)
            .map_err(HandshakeError::Io)?;
        let info = ServerInfo::parse(&datagram).map_err(HandshakeError::Decode)?;
        if let Err(source) = layout(&info).validate() {
            return Err(HandshakeError::UnsupportedVersion { info, source });
        }
        Ok(info)
    }

    fn exchange_inner(&mut self, request: &Request, timeout: Duration) -> io::Result<Vec<u8>> {
        if timeout.is_zero() {
            return Err(invalid_input("command timeout must be nonzero"));
        }
        let deadline = Instant::now()
            .checked_add(timeout)
            .ok_or_else(|| invalid_input("command timeout overflows the monotonic clock"))?;
        if request.as_bytes().len() > MAX_IPV4_UDP_PAYLOAD {
            return Err(invalid_input(format_args!(
                "command datagram length {} exceeds IPv4 UDP limit {MAX_IPV4_UDP_PAYLOAD}",
                request.as_bytes().len()
            )));
        }

        let previous_read = self.socket.read_timeout()?;
        let previous_write = self.socket.write_timeout()?;
        let result = self.exchange_with_deadline(request, deadline);
        let restore_read = self.socket.set_read_timeout(previous_read);
        let restore_write = self.socket.set_write_timeout(previous_write);

        match (restore_read, restore_write) {
            (Err(error), _) | (_, Err(error)) => Err(error),
            (Ok(()), Ok(())) => result,
        }
    }

    fn exchange_with_deadline(&self, request: &Request, deadline: Instant) -> io::Result<Vec<u8>> {
        let mut buffer = vec![0u8; MAX_COMMAND_DATAGRAM].into_boxed_slice();
        self.drain_queued(&mut buffer, deadline)?;

        let write_remaining = remaining(deadline)?;
        self.socket.set_write_timeout(Some(write_remaining))?;
        self.socket.send_to(request.as_bytes(), self.server)?;

        let expected_id = expected_message_id(request.response_kind());
        loop {
            let remaining = remaining(deadline)?;
            self.socket.set_read_timeout(Some(remaining))?;
            let (received, source) = match self.socket.recv_from(&mut buffer) {
                Ok(received) => received,
                // Linux reports an `SO_RCVTIMEO` expiry as `EAGAIN`; the
                // documented contract is one timeout kind on every platform.
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    return Err(io::Error::new(
                        io::ErrorKind::TimedOut,
                        "command deadline elapsed",
                    ));
                }
                Err(error) => return Err(error),
            };
            if source != self.server {
                continue;
            }
            if received < 2 {
                continue;
            }
            let found = u16::from_le_bytes([buffer[0], buffer[1]]);
            if found == NAT_UNRECOGNIZED_REQUEST {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "server replied NAT_UNRECOGNIZED_REQUEST",
                ));
            }
            if found != expected_id {
                continue;
            }
            MessageHeader::parse(&buffer[..received])
                .map_err(|error| invalid_data(error.to_string()))?;
            return Ok(buffer[..received].to_vec());
        }
    }

    /// Discards every datagram already queued on the socket before a request
    /// is sent, so a reply that arrived after an earlier transaction's
    /// deadline cannot be accepted as the answer to this one.
    ///
    /// The drain is bounded by the transaction deadline, so a peer flooding
    /// the socket cannot hold it open. Blocking mode is restored on every
    /// exit; the drain error takes precedence over a restore error.
    fn drain_queued(&self, buffer: &mut [u8], deadline: Instant) -> io::Result<()> {
        self.socket.set_nonblocking(true)?;
        let drained = loop {
            if let Err(error) = remaining(deadline) {
                break Err(error);
            }
            match self.socket.recv_from(buffer) {
                Ok(_) => {}
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => break Ok(()),
                // Windows reports an ICMP port-unreachable caused by an
                // earlier send as `ConnectionReset` on a later UDP receive.
                // Anything queued before this request is stale by definition.
                Err(error)
                    if matches!(
                        error.kind(),
                        io::ErrorKind::Interrupted | io::ErrorKind::ConnectionReset
                    ) => {}
                Err(error) => break Err(error),
            }
        };
        let restored = self.socket.set_nonblocking(false);
        drained?;
        restored
    }
}

fn expected_message_id(response: ResponseKind) -> u16 {
    match response {
        ResponseKind::ServerInfo => NAT_SERVERINFO,
        ResponseKind::ModelDefinition => NAT_MODELDEF,
        ResponseKind::FrameOfData => NAT_FRAMEOFDATA,
        ResponseKind::Status | ResponseKind::Value => NAT_RESPONSE,
    }
}

fn remaining(deadline: Instant) -> io::Result<Duration> {
    let duration = deadline.saturating_duration_since(Instant::now());
    if duration.is_zero() {
        Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "command deadline elapsed",
        ))
    } else {
        Ok(duration)
    }
}

fn invalid_input(message: impl fmt::Display) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message.to_string())
}

fn invalid_data(message: impl fmt::Display) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.to_string())
}

/// A bounded handshake failure. Unsupported replies retain their decoded
/// SERVERINFO so callers can log the producer's reported version.
#[derive(Debug)]
pub enum HandshakeError {
    /// Socket or deadline failure.
    Io(io::Error),
    /// The typed NAT_CONNECT request could not be constructed.
    Command(command::CommandError),
    /// The peer's datagram was not a valid SERVERINFO.
    Decode(NatNetError),
    /// SERVERINFO decoded, but its protocol layout is not supported.
    UnsupportedVersion {
        /// Decoded SERVERINFO retained for diagnostics.
        info: ServerInfo,
        /// The version validation failure.
        source: NatNetError,
    },
}

impl HandshakeError {
    /// Returns the raw negotiated information when an unsupported version was
    /// reported by the peer.
    #[must_use]
    pub fn server_info(&self) -> Option<&ServerInfo> {
        match self {
            Self::UnsupportedVersion { info, .. } => Some(info),
            _ => None,
        }
    }
}

impl fmt::Display for HandshakeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "command handshake I/O error: {error}"),
            Self::Command(error) => write!(formatter, "command handshake request error: {error}"),
            Self::Decode(error) => write!(formatter, "invalid SERVERINFO reply: {error}"),
            Self::UnsupportedVersion { source, .. } => {
                write!(
                    formatter,
                    "unsupported SERVERINFO protocol version: {source}"
                )
            }
        }
    }
}

impl std::error::Error for HandshakeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Command(error) => Some(error),
            Self::Decode(error) => Some(error),
            Self::UnsupportedVersion { source, .. } => Some(source),
        }
    }
}

/// The point in a Tier-1 transaction at which a failure occurred.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier1Phase {
    /// Waiting for the SetProperty status.
    SetPropertyStatus,
    /// Waiting for or validating the GetProperty re-read.
    GetPropertyReread,
}

/// A transport or command failure recorded by the Tier-1 executor.
#[derive(Debug)]
pub enum Tier1Failure {
    /// The socket exchange failed or timed out. This also covers a preflight
    /// failure before the write reached the socket, so the error does not by
    /// itself prove that a remote write was attempted.
    Io(io::Error),
    /// The received datagram failed command decoding or verification.
    Command(command::CommandError),
}

impl fmt::Display for Tier1Failure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => error.fmt(formatter),
            Self::Command(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for Tier1Failure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Command(error) => Some(error),
        }
    }
}

/// A failed sequential Tier-1 execution.
#[derive(Debug)]
pub enum Tier1ExecutionError {
    /// A nonzero server status explicitly rejected the current write.
    StatusFailure {
        /// The target whose write was rejected.
        target: String,
        /// The producer's nonzero status code.
        status: i32,
        /// Earlier targets whose write and re-read were verified.
        progress: Vec<command::ProgressRecord>,
    },
    /// The write or its transport preflight failed, so the remote outcome
    /// could not be established. The executor has stopped and never retries
    /// automatically.
    UnknownRemoteOutcome {
        /// The target associated with the failed operation. Its write may not
        /// have reached the producer.
        target: String,
        /// The operation stage that failed.
        phase: Tier1Phase,
        /// The transport or verification failure.
        source: Box<Tier1Failure>,
        /// Earlier targets whose write and re-read were verified.
        progress: Vec<command::ProgressRecord>,
    },
}

impl Tier1ExecutionError {
    /// Returns the ordered records completed before this failure.
    #[must_use]
    pub fn progress(&self) -> &[command::ProgressRecord] {
        match self {
            Self::StatusFailure { progress, .. } | Self::UnknownRemoteOutcome { progress, .. } => {
                progress
            }
        }
    }
}

impl fmt::Display for Tier1ExecutionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StatusFailure { target, status, .. } => {
                write!(
                    formatter,
                    "target {target:?} rejected write with status {status}"
                )
            }
            Self::UnknownRemoteOutcome {
                target,
                phase,
                source,
                ..
            } => write!(
                formatter,
                "target {target:?} has unknown remote outcome during {phase:?}: {source}"
            ),
        }
    }
}

impl std::error::Error for Tier1ExecutionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::UnknownRemoteOutcome { source, .. } => Some(source.as_ref()),
            Self::StatusFailure { .. } => None,
        }
    }
}

/// Executes a validated Tier-1 batch one target at a time.
///
/// A write status and request-context re-read must both succeed before the
/// next target is sent. Every failure after a write attempt stops progression;
/// a preflight failure also stops before any send. The returned error records
/// the target, prior progress and any unknown remote outcome. No retry or
/// rollback is attempted.
impl CommandClient {
    /// Executes a validated Tier-1 write sequence with one write/status and
    /// request-context re-read per target.
    ///
    /// # Errors
    ///
    /// Returns a structured status or unknown-remote-outcome failure. Once a
    /// write is attempted, the sequence stops on any later transport or
    /// verification failure and never retries automatically. Every failure
    /// carries the progress records completed before the failed target.
    pub fn execute_tier1(
        &mut self,
        write: command::Tier1Write,
        timeout: Duration,
    ) -> Result<Vec<command::ProgressRecord>, Tier1ExecutionError> {
        let mut step = write.begin();
        let mut completed = Vec::new();
        loop {
            let target = step.target().to_owned();
            let status = self
                .exchange_inner(step.set_request(), timeout)
                .map_err(|source| Tier1ExecutionError::UnknownRemoteOutcome {
                    target: target.clone(),
                    phase: Tier1Phase::SetPropertyStatus,
                    source: Box::new(Tier1Failure::Io(source)),
                    progress: completed.clone(),
                })?;

            match CommandResponse::parse(&status) {
                Ok(CommandResponse::Status(0)) => {}
                Ok(CommandResponse::Status(status)) => {
                    return Err(Tier1ExecutionError::StatusFailure {
                        target,
                        status,
                        progress: completed,
                    });
                }
                Ok(CommandResponse::Value(_)) => {
                    return Err(Tier1ExecutionError::UnknownRemoteOutcome {
                        target,
                        phase: Tier1Phase::SetPropertyStatus,
                        source: Box::new(Tier1Failure::Command(
                            command::CommandError::StatusExpected,
                        )),
                        progress: completed,
                    });
                }
                Err(source) => {
                    return Err(Tier1ExecutionError::UnknownRemoteOutcome {
                        target,
                        phase: Tier1Phase::SetPropertyStatus,
                        source: Box::new(Tier1Failure::Command(command::CommandError::Wire(
                            source,
                        ))),
                        progress: completed,
                    });
                }
            }

            let reread = self
                .exchange_inner(step.reread_request(), timeout)
                .map_err(|source| Tier1ExecutionError::UnknownRemoteOutcome {
                    target: target.clone(),
                    phase: Tier1Phase::GetPropertyReread,
                    source: Box::new(Tier1Failure::Io(source)),
                    progress: completed.clone(),
                })?;
            let verified = step.verify_status(&status, &reread).map_err(|source| {
                Tier1ExecutionError::UnknownRemoteOutcome {
                    target: target.clone(),
                    phase: Tier1Phase::GetPropertyReread,
                    source: Box::new(Tier1Failure::Command(source)),
                    progress: completed.clone(),
                }
            })?;
            let progress = verified.progress().to_vec();
            match verified.next() {
                Some(next) => {
                    completed = progress;
                    step = next;
                }
                None => return Ok(progress),
            }
        }
    }
}
