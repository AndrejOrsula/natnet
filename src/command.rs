//! Validated NatNet command requests and guarded property writes.
//!
//! Command requests are validated before they are framed. A [`Request`] owns
//! immutable bytes and private metadata describing the operation that produced
//! them; it has no raw-byte constructor or mutable byte view. A transport can
//! read [`Request::as_bytes`] and remains responsible for the actual send
//! limits. The optional `net` client accepts these requests rather than
//! arbitrary byte slices.
//!
//! Tier-1 writes are consuming state machines. [`Tier1Write::begin`] exposes
//! the first target only. Its write status and value re-read must be observed
//! before [`CanaryStep::verify_status`] unlocks one next target. No batch of
//! unverified datagrams can be obtained from this module.

use std::collections::VecDeque;

use crate::bytes::Reader;
use crate::error::{Error, Result};
use crate::message::{
    self, HEADER_LEN, NAT_CONNECT, NAT_DISCONNECT, NAT_KEEPALIVE, NAT_REQUEST,
    NAT_REQUEST_FRAMEOFDATA, NAT_REQUEST_MODELDEF, NAT_RESPONSE,
};
use crate::serverinfo::ServerInfo;
use crate::version::Version;

/// A command component's location in the textual command grammar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CommandComponent {
    /// A Motive node, camera, or asset identity.
    Node,
    /// A Motive property name.
    Property,
    /// A textual property value.
    Value,
    /// Audit metadata naming the person or process that signed off.
    Authority,
}

/// Why a component is not representable in the ASCII command grammar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ComponentError {
    /// The component was empty where a value was required.
    Empty,
    /// A comma would create another command component.
    Comma,
    /// An interior NUL would terminate the command early.
    Nul,
    /// A control byte, including DEL, is not command text.
    Control(u8),
    /// Non-ASCII input; no implicit UTF-8 encoding is applied.
    NonAscii(u8),
}

/// An immutable, validated NatNet command datagram.
///
/// Private fields prevent forging: no `From<Vec<u8>>`, `AsRef<[u8]>`, `AsMut`
/// or `Deserialize` turns arbitrary bytes into a request. Read the bytes with
/// [`Request::as_bytes`]. `Debug` includes Tier-2 authority metadata,
/// which is an audit label, not authenticated identity.
///
/// ```compile_fail
/// natnet::command::connect_request().unwrap().as_bytes()[0] = 1;
/// ```
///
/// ```compile_fail
/// let _forged: natnet::command::Request = vec![0, 0, 0, 0].into();
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    bytes: Vec<u8>,
    metadata: RequestMetadata,
}

impl Request {
    /// Returns the complete NatNet datagram as an immutable byte slice.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    #[cfg(feature = "net")]
    pub(crate) fn kind(&self) -> RequestKind {
        self.metadata.kind
    }

    #[cfg(feature = "net")]
    pub(crate) fn response_kind(&self) -> ResponseKind {
        self.metadata.response
    }

    #[cfg(feature = "net")]
    pub(crate) fn tier(&self) -> Option<WriteTier> {
        self.metadata.tier
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RequestMetadata {
    kind: RequestKind,
    response: ResponseKind,
    target: Option<String>,
    property: Option<String>,
    value: Option<PropertyValue>,
    tier: Option<WriteTier>,
    authority: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RequestKind {
    Connect,
    ModelDefinition,
    FrameOfData,
    GetProperty,
    SetProperty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ResponseKind {
    ServerInfo,
    ModelDefinition,
    FrameOfData,
    Status,
    Value,
}

/// Builds an empty NAT_CONNECT hello; use [`connect_request_with`] for options.
///
/// # Errors
/// This fixed request cannot fail.
pub fn connect_request() -> core::result::Result<Request, CommandError> {
    Ok(empty_request(
        NAT_CONNECT,
        RequestKind::Connect,
        ResponseKind::ServerInfo,
    ))
}

/// NAT_CONNECT sender block length: 256-byte name plus application and NatNet versions.
pub const CONNECT_SENDER_LEN: usize = 264;

/// NAT_CONNECT options payload length: sender block, flag, four-byte version, and NUL.
pub const CONNECT_OPTIONS_PAYLOAD_LEN: usize = CONNECT_SENDER_LEN + 6;

/// NAT_CONNECT connection options. A zero version requests the current
/// bitstream; version switching requires NatNet 3+ and unicast. Motive support
/// for these options is not verified by this crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ConnectOptions {
    /// Ask the server to send only subscribed data.
    pub subscribed_data_only: bool,
    /// Requested bitstream; `None` sends 0.0.0.0 (current version).
    pub bitstream_version: Option<Version>,
}

impl ConnectOptions {
    /// Options that request only `version`.
    #[must_use]
    pub const fn requesting(version: Version) -> Self {
        Self {
            subscribed_data_only: false,
            bitstream_version: Some(version),
        }
    }

    /// Reads options at offset 264. Returns `None` if fewer than 269 bytes are
    /// present or the encoded version is 0.0.0.0; ignores trailing bytes.
    #[must_use]
    pub fn parse(payload: &[u8]) -> Option<Self> {
        let flag = *payload.get(CONNECT_SENDER_LEN)?;
        let version = payload.get(CONNECT_SENDER_LEN + 1..CONNECT_SENDER_LEN + 5)?;
        let version = Version::from_bytes(version.try_into().ok()?);
        Some(Self {
            subscribed_data_only: flag != 0,
            bitstream_version: (version != Version::new(0, 0, 0, 0)).then_some(version),
        })
    }

    /// Stream version by the SDK rule: requested version for NatNet 3+, else
    /// the server's advertised version.
    #[must_use]
    pub fn stream_version(&self, info: &ServerInfo) -> Version {
        match self.bitstream_version {
            Some(requested) if info.protocol_version.major >= 3 => requested,
            _ => info.protocol_version,
        }
    }
}

/// Builds NAT_CONNECT with options at [`CONNECT_SENDER_LEN`] in a zeroed block.
///
/// # Errors
/// This fixed-size request cannot fail.
pub fn connect_request_with(
    options: ConnectOptions,
) -> core::result::Result<Request, CommandError> {
    let mut payload = [0u8; CONNECT_OPTIONS_PAYLOAD_LEN];
    payload[CONNECT_SENDER_LEN] = u8::from(options.subscribed_data_only);
    if let Some(version) = options.bitstream_version {
        payload[CONNECT_SENDER_LEN + 1..CONNECT_SENDER_LEN + 5]
            .copy_from_slice(&version.to_bytes());
    }
    let mut request = empty_request(NAT_CONNECT, RequestKind::Connect, ResponseKind::ServerInfo);
    request.bytes = message::frame(NAT_CONNECT, &payload);
    Ok(request)
}

/// Builds an empty NAT_REQUEST_MODELDEF request.
///
/// # Errors
/// This fixed request cannot fail.
pub fn request_modeldef() -> core::result::Result<Request, CommandError> {
    Ok(empty_request(
        NAT_REQUEST_MODELDEF,
        RequestKind::ModelDefinition,
        ResponseKind::ModelDefinition,
    ))
}

/// Builds an empty request for one FrameOfData on the command channel.
/// Motive's answer is unverified (SDK samples never send it). Unsupported
/// servers reply NAT_UNRECOGNIZED_REQUEST; `net` reports it as `Unsupported`.
///
/// # Errors
/// This fixed request cannot fail.
pub fn request_frame_of_data() -> core::result::Result<Request, CommandError> {
    Ok(empty_request(
        NAT_REQUEST_FRAMEOFDATA,
        RequestKind::FrameOfData,
        ResponseKind::FrameOfData,
    ))
}

/// Empty-payload NAT_KEEPALIVE datagram. Unicast clients send it periodically
/// on the command channel; it has no reply and is not a [`Request`].
#[must_use]
pub const fn keepalive_datagram() -> [u8; HEADER_LEN] {
    empty_datagram(NAT_KEEPALIVE)
}

/// Empty-payload NAT_DISCONNECT datagram; a client sends it when leaving.
/// Whether Motive stops a unicast stream at once on it is unverified.
#[must_use]
pub const fn disconnect_datagram() -> [u8; HEADER_LEN] {
    empty_datagram(NAT_DISCONNECT)
}

const fn empty_datagram(message_id: u16) -> [u8; HEADER_LEN] {
    let id = message_id.to_le_bytes();
    [id[0], id[1], 0, 0]
}

fn empty_request(message_id: u16, kind: RequestKind, response: ResponseKind) -> Request {
    Request {
        bytes: message::frame(message_id, &[]),
        metadata: RequestMetadata {
            kind,
            response,
            target: None,
            property: None,
            value: None,
            tier: None,
            authority: None,
        },
    }
}

/// Encodes read-only `GetProperty,<node>,<property>`; empty `node` selects
/// Motive's root scope. Property must be nonempty. Known names are
/// canonicalized; other names are validated and trimmed.
///
/// # Errors
///
/// Returns [`CommandError`] for an invalid component or payload over
/// `u16::MAX` bytes.
pub fn get_property(node: &str, property: &str) -> core::result::Result<Request, CommandError> {
    validate_component(node, CommandComponent::Node, true)?;
    let property = canonical_property(property)?;
    let payload = command_payload(&["GetProperty", node, &property])?;
    Ok(request_from_payload(
        &payload,
        RequestMetadata {
            kind: RequestKind::GetProperty,
            response: ResponseKind::Value,
            target: Some(node.to_owned()),
            property: Some(property),
            value: None,
            tier: None,
            authority: None,
        },
    ))
}

fn command_payload(components: &[&str]) -> core::result::Result<Vec<u8>, CommandError> {
    let mut payload = Vec::new();
    for (index, component) in components.iter().enumerate() {
        if index != 0 {
            payload.push(b',');
        }
        payload.extend_from_slice(component.as_bytes());
    }
    payload.push(0);
    if payload.len() > usize::from(u16::MAX) {
        return Err(CommandError::RequestTooLong {
            length: payload.len(),
            max: usize::from(u16::MAX),
        });
    }
    Ok(payload)
}

fn request_from_payload(payload: &[u8], metadata: RequestMetadata) -> Request {
    Request {
        bytes: message::frame(NAT_REQUEST, payload),
        metadata,
    }
}

fn validate_component(
    value: &str,
    component: CommandComponent,
    allow_empty: bool,
) -> core::result::Result<(), CommandError> {
    if value.is_empty() && !allow_empty {
        return Err(CommandError::InvalidComponent {
            component,
            reason: ComponentError::Empty,
        });
    }
    for &byte in value.as_bytes() {
        if !byte.is_ascii() {
            return Err(CommandError::InvalidComponent {
                component,
                reason: ComponentError::NonAscii(byte),
            });
        }
        match byte {
            b',' => {
                return Err(CommandError::InvalidComponent {
                    component,
                    reason: ComponentError::Comma,
                });
            }
            0 => {
                return Err(CommandError::InvalidComponent {
                    component,
                    reason: ComponentError::Nul,
                });
            }
            0..=0x1f | 0x7f => {
                return Err(CommandError::InvalidComponent {
                    component,
                    reason: ComponentError::Control(byte),
                });
            }
            _ => {}
        }
    }
    Ok(())
}

fn validate_authority(authority: &str) -> core::result::Result<(), CommandError> {
    validate_component(authority, CommandComponent::Authority, false)?;
    if authority.trim().is_empty() {
        return Err(CommandError::InvalidComponent {
            component: CommandComponent::Authority,
            reason: ComponentError::Empty,
        });
    }
    Ok(())
}

fn canonical_property(property: &str) -> core::result::Result<String, CommandError> {
    validate_component(property, CommandComponent::Property, false)?;
    let trimmed = property.trim();
    if trimmed.is_empty() {
        return Err(CommandError::InvalidComponent {
            component: CommandComponent::Property,
            reason: ComponentError::Empty,
        });
    }
    for candidate in TIER1_PROPERTIES {
        if trimmed.eq_ignore_ascii_case(candidate) {
            return Ok(candidate.to_owned());
        }
    }
    for candidate in Tier2Property::ALL {
        if trimmed.eq_ignore_ascii_case(candidate.property_name()) {
            return Ok(candidate.property_name().to_owned());
        }
    }
    Ok(trimmed.to_owned())
}

/// A property value carried by a SetProperty write.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropertyValue {
    /// A boolean flag.
    Bool(bool),
    /// An integer; valid Video Mode ranges are device-specific.
    Int(i64),
    /// A textual value in the NatNet ASCII command grammar.
    Text(String),
}

impl PropertyValue {
    /// The canonical wire representation.
    #[must_use]
    pub fn wire(&self) -> String {
        match self {
            Self::Bool(value) => i32::from(*value).to_string(),
            Self::Int(value) => value.to_string(),
            Self::Text(value) => value.clone(),
        }
    }

    fn validate(&self) -> core::result::Result<String, CommandError> {
        let wire = self.wire();
        validate_component(&wire, CommandComponent::Value, true)?;
        Ok(wire)
    }

    /// Whether a re-read string denotes this value.
    #[must_use]
    pub fn matches(&self, echo: &str) -> bool {
        match self {
            Self::Bool(value) => {
                let echo = echo.trim();
                let truthy = echo == "1" || echo.eq_ignore_ascii_case("true");
                let falsy = echo == "0" || echo.eq_ignore_ascii_case("false");
                (*value && truthy) || (!*value && falsy)
            }
            Self::Int(value) => echo
                .trim()
                .parse::<i64>()
                .is_ok_and(|found| found == *value),
            Self::Text(value) => echo == value,
        }
    }
}

/// The write tier of a property.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteTier {
    /// A remotely writable property requiring a canary and re-read.
    Tier1,
    /// A property requiring explicit per-action authorization.
    Tier2,
    /// A property not recognized by this crate.
    Unsanctioned,
}

/// Tier-2 properties requiring an explicit grant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Tier2Property {
    /// Stream transmission type.
    TransmissionType,
    /// Local streaming interface.
    LocalInterface,
    /// Data/command port.
    Port,
    /// Live/Edit mode.
    LiveEditMode,
    /// Recording start/stop.
    Recording,
    /// Camera-system calibration.
    Calibration,
    /// Asset definition, which is not a NatNet SetProperty operation.
    AssetDefinition,
}

impl Tier2Property {
    /// All recognized Tier-2 properties.
    pub const ALL: [Self; 7] = [
        Self::TransmissionType,
        Self::LocalInterface,
        Self::Port,
        Self::LiveEditMode,
        Self::Recording,
        Self::Calibration,
        Self::AssetDefinition,
    ];

    /// The canonical Motive property spelling.
    #[must_use]
    pub const fn property_name(self) -> &'static str {
        match self {
            Self::TransmissionType => "Transmission Type",
            Self::LocalInterface => "Local Interface",
            Self::Port => "Port",
            Self::LiveEditMode => "Mode",
            Self::Recording => "Record",
            Self::Calibration => "Calibration",
            Self::AssetDefinition => "Asset Definition",
        }
    }
}

const TIER1_PROPERTIES: [&str; 3] = ["Enabled", "Video Mode", "Streaming"];

/// Classifies a property name fail-closed.
#[must_use]
pub fn classify_property(property: &str) -> WriteTier {
    let property = property.trim();
    if TIER1_PROPERTIES
        .iter()
        .any(|candidate| property.eq_ignore_ascii_case(candidate))
    {
        return WriteTier::Tier1;
    }
    if Tier2Property::ALL
        .iter()
        .any(|candidate| property.eq_ignore_ascii_case(candidate.property_name()))
    {
        return WriteTier::Tier2;
    }
    WriteTier::Unsanctioned
}

fn build_set_request(
    target: &str,
    property: &str,
    value: &PropertyValue,
    tier: WriteTier,
    authority: Option<String>,
) -> core::result::Result<Request, CommandError> {
    validate_component(target, CommandComponent::Node, true)?;
    let property = canonical_property(property)?;
    let wire = value.validate()?;
    let payload = command_payload(&["SetProperty", target, &property, &wire])?;
    Ok(request_from_payload(
        &payload,
        RequestMetadata {
            kind: RequestKind::SetProperty,
            response: ResponseKind::Status,
            target: Some(target.to_owned()),
            property: Some(property),
            value: Some(value.clone()),
            tier: Some(tier),
            authority,
        },
    ))
}

/// Explicit authorization to build one Tier-2 write; private fields prevent
/// forging. Consuming the grant does not limit sends because [`Request`] is
/// cloneable. Authority is audit metadata, not authenticated identity or replay
/// protection.
///
/// ```compile_fail
/// use natnet::command::{Tier2Grant, Tier2Property};
/// let _forged = Tier2Grant {
///     authority: String::from("caller"),
///     action: Tier2Property::Port,
/// };
/// ```
#[derive(Debug)]
pub struct Tier2Grant {
    authority: String,
    action: Tier2Property,
}

impl Tier2Grant {
    /// Records per-action sign-off as immutable audit metadata.
    ///
    /// # Errors
    ///
    /// Returns [`CommandError::InvalidComponent`] for empty or invalid metadata.
    pub fn sign_off(
        authority: impl Into<String>,
        action: Tier2Property,
    ) -> core::result::Result<Self, CommandError> {
        let authority = authority.into();
        validate_authority(&authority)?;
        Ok(Self { authority, action })
    }

    /// The signed audit authority.
    #[must_use]
    pub fn authority(&self) -> &str {
        &self.authority
    }

    /// The one action covered by this grant.
    #[must_use]
    pub const fn action(&self) -> Tier2Property {
        self.action
    }
}

/// A sanctioned Tier-2 write built by spending a [`Tier2Grant`].
#[derive(Debug)]
pub struct Tier2Write {
    request: Request,
    authority: String,
    action: Tier2Property,
}

impl Tier2Write {
    /// Builds the granted action and consumes its grant.
    ///
    /// # Errors
    ///
    /// Returns validation errors or [`CommandError::UnsupportedOperation`]
    /// for `AssetDefinition`.
    #[allow(clippy::needless_pass_by_value)]
    pub fn authorize(
        grant: Tier2Grant,
        node: impl Into<String>,
        value: PropertyValue,
    ) -> core::result::Result<Self, CommandError> {
        let Tier2Grant { authority, action } = grant;
        if matches!(action, Tier2Property::AssetDefinition) {
            return Err(CommandError::UnsupportedOperation {
                operation: "Asset Definition",
            });
        }
        let node = node.into();
        let request = build_set_request(
            &node,
            action.property_name(),
            &value,
            WriteTier::Tier2,
            Some(authority.clone()),
        )?;
        Ok(Self {
            request,
            authority,
            action,
        })
    }

    /// The immutable request datagram.
    #[must_use]
    pub fn request(&self) -> &Request {
        &self.request
    }

    /// Consumes the write and returns its immutable request.
    #[must_use]
    pub fn into_request(self) -> Request {
        self.request
    }

    /// The authority bound to the request.
    #[must_use]
    pub fn authority(&self) -> &str {
        &self.authority
    }

    /// The action bound to the request.
    #[must_use]
    pub const fn action(&self) -> Tier2Property {
        self.action
    }
}

struct PlannedTarget {
    target: String,
    set_request: Request,
    reread_request: Request,
}

impl core::fmt::Debug for PlannedTarget {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("PlannedTarget")
            .field("target", &self.target)
            .field("set_request", &self.set_request)
            .field("reread_request", &self.reread_request)
            .finish()
    }
}

/// Validated Tier-1 batch. All targets and requests validate before `begin`
/// exposes the first request.
#[derive(Debug)]
pub struct Tier1Write {
    property: String,
    value: PropertyValue,
    first: PlannedTarget,
    targets: VecDeque<PlannedTarget>,
}

impl Tier1Write {
    /// Plans a per-camera `Enabled` write.
    ///
    /// # Errors
    /// Invalid components, no targets, duplicate targets, or oversized requests.
    pub fn camera_enabled(
        serials: Vec<String>,
        enabled: bool,
    ) -> core::result::Result<Self, CommandError> {
        Self::new(serials, "Enabled", PropertyValue::Bool(enabled))
    }

    /// Plans a per-camera `Video Mode` write; device-specific ranges are not
    /// enforced here.
    ///
    /// # Errors
    /// Invalid components, no targets, duplicate targets, or oversized requests.
    pub fn camera_video_mode(
        serials: Vec<String>,
        mode: i64,
    ) -> core::result::Result<Self, CommandError> {
        Self::new(serials, "Video Mode", PropertyValue::Int(mode))
    }

    /// Plans a streaming-pane `Streaming` write over asset identities.
    ///
    /// # Errors
    /// Invalid components, no targets, duplicate targets, or oversized requests.
    pub fn asset_streaming(
        assets: Vec<String>,
        streaming: bool,
    ) -> core::result::Result<Self, CommandError> {
        Self::new(assets, "Streaming", PropertyValue::Bool(streaming))
    }

    fn new(
        targets: Vec<String>,
        property: &str,
        value: PropertyValue,
    ) -> core::result::Result<Self, CommandError> {
        if targets.is_empty() {
            return Err(CommandError::NoTargets);
        }
        let property = canonical_property(property)?;
        if classify_property(&property) != WriteTier::Tier1 {
            return Err(CommandError::Tier2Refused { property });
        }
        value.validate()?;

        for target in &targets {
            validate_component(target, CommandComponent::Node, false)?;
        }
        for (index, target) in targets.iter().enumerate() {
            if targets[index + 1..].iter().any(|other| other == target) {
                return Err(CommandError::DuplicateTarget {
                    target: target.clone(),
                });
            }
        }

        let mut planned = VecDeque::with_capacity(targets.len());
        for target in targets {
            let set_request =
                build_set_request(&target, &property, &value, WriteTier::Tier1, None)?;
            let reread_request = get_property(&target, &property)?;
            planned.push_back(PlannedTarget {
                target,
                set_request,
                reread_request,
            });
        }
        let Some(first) = planned.pop_front() else {
            return Err(CommandError::NoTargets);
        };
        Ok(Self {
            property,
            value,
            first,
            targets: planned,
        })
    }

    /// The canonical property name.
    #[must_use]
    pub fn property(&self) -> &str {
        &self.property
    }

    /// The planned value.
    #[must_use]
    pub fn value(&self) -> &PropertyValue {
        &self.value
    }

    /// Consumes the plan and exposes only its first target.
    #[must_use]
    pub fn begin(self) -> CanaryStep {
        CanaryStep {
            current: self.first,
            remaining: self.targets,
            property: self.property,
            value: self.value,
            canary: true,
            progress: Vec::new(),
        }
    }
}

/// One write/re-read pair; linear state cannot be reused after verification.
///
/// ```compile_fail
/// use natnet::command::Tier1Write;
/// let step = Tier1Write::camera_enabled(vec!["cam-1".into()], true).unwrap().begin();
/// let _copy = step.clone();
/// ```
///
/// ```compile_fail
/// use natnet::command::Tier1Write;
/// let step = Tier1Write::camera_enabled(vec!["cam-1".into()], true).unwrap().begin();
/// let _ = step.verify_echo("1").unwrap(); let _ = step.target();
/// ```
#[derive(Debug)]
pub struct CanaryStep {
    current: PlannedTarget,
    remaining: VecDeque<PlannedTarget>,
    property: String,
    value: PropertyValue,
    canary: bool,
    progress: Vec<ProgressRecord>,
}

impl CanaryStep {
    /// The target being written at this step.
    #[must_use]
    pub fn target(&self) -> &str {
        &self.current.target
    }

    /// Whether this is the first target in the sequence.
    #[must_use]
    pub const fn is_canary(&self) -> bool {
        self.canary
    }

    /// The immutable SetProperty request for this target.
    #[must_use]
    pub fn set_request(&self) -> &Request {
        &self.current.set_request
    }

    /// The immutable GetProperty re-read request for this target.
    #[must_use]
    pub fn reread_request(&self) -> &Request {
        &self.current.reread_request
    }

    /// Verifies a request-context value re-read, without checking write status.
    /// Transports should prefer [`Self::verify_status`].
    ///
    /// # Errors
    /// [`CommandError::RereadMismatch`], wrapped in
    /// [`CommandError::VerificationFailed`] when prior targets completed.
    pub fn verify_echo(self, echo: &str) -> core::result::Result<Verified, CommandError> {
        self.finish(None, echo)
    }

    /// Decodes a generic response as a value and verifies it. Four-byte
    /// payloads parse as status, including Motive's unterminated `true`; for
    /// GetProperty use [`CommandResponse::parse_value`] with
    /// [`Self::verify_echo`].
    ///
    /// # Errors
    /// [`CommandError::Wire`], [`CommandError::UnverifiableReply`],
    /// [`CommandError::RereadMismatch`], or [`CommandError::VerificationFailed`].
    pub fn verify(self, reread_reply: &[u8]) -> core::result::Result<Verified, CommandError> {
        let response = match CommandResponse::parse(reread_reply) {
            Ok(response) => response,
            Err(error) => return Err(self.failure(CommandError::Wire(error))),
        };
        match response {
            CommandResponse::Value(value) => self.verify_echo(&value),
            CommandResponse::Status(_) => Err(self.failure(CommandError::UnverifiableReply)),
        }
    }

    /// Verifies write status, then the request-context value re-read. Failure
    /// consumes the step, so no next target can be obtained.
    ///
    /// # Errors
    /// [`CommandError::Wire`], [`CommandError::StatusFailure`],
    /// [`CommandError::StatusExpected`], [`CommandError::RereadMismatch`], or
    /// [`CommandError::VerificationFailed`].
    pub fn verify_status(
        self,
        status_reply: &[u8],
        reread_reply: &[u8],
    ) -> core::result::Result<Verified, CommandError> {
        let response = match CommandResponse::parse(status_reply) {
            Ok(response) => response,
            Err(error) => return Err(self.failure(CommandError::Wire(error))),
        };
        let status = match response {
            CommandResponse::Status(status) => status,
            CommandResponse::Value(_) => return Err(self.failure(CommandError::StatusExpected)),
        };
        if status != 0 {
            let target = self.current.target.clone();
            return Err(self.failure(CommandError::StatusFailure { target, status }));
        }
        let echo = match CommandResponse::parse_value(reread_reply) {
            Ok(echo) => echo,
            Err(error) => return Err(self.failure(CommandError::Wire(error))),
        };
        self.finish(Some(status), &echo)
    }

    fn failure(self, source: CommandError) -> CommandError {
        if self.progress.is_empty() {
            source
        } else {
            CommandError::VerificationFailed {
                source: Box::new(source),
                progress: self.progress,
            }
        }
    }

    fn finish(
        self,
        status: Option<i32>,
        echo: &str,
    ) -> core::result::Result<Verified, CommandError> {
        if !self.value.matches(echo) {
            let target = self.current.target.clone();
            let property = self.property.clone();
            let expected = self.value.wire();
            return Err(self.failure(CommandError::RereadMismatch {
                target,
                property,
                expected,
                found: echo.to_owned(),
            }));
        }
        let mut progress = self.progress;
        progress.push(ProgressRecord {
            target: self.current.target,
            phase: if self.canary {
                ProgressPhase::Canary
            } else {
                ProgressPhase::Target
            },
            status,
            reread: echo.to_owned(),
        });
        Ok(Verified {
            remaining: self.remaining,
            property: self.property,
            value: self.value,
            progress,
        })
    }
}

/// Verified step; [`Verified::next`] consumes it and exposes one next target.
#[derive(Debug)]
pub struct Verified {
    remaining: VecDeque<PlannedTarget>,
    property: String,
    value: PropertyValue,
    progress: Vec<ProgressRecord>,
}

impl Verified {
    /// Takes the next target pair; `None` means all targets were verified.
    #[must_use]
    pub fn next(self) -> Option<CanaryStep> {
        let mut remaining = self.remaining;
        let current = remaining.pop_front()?;
        Some(CanaryStep {
            current,
            remaining,
            property: self.property,
            value: self.value,
            canary: false,
            progress: self.progress,
        })
    }

    /// Progress records accumulated through this verified point.
    #[must_use]
    pub fn progress(&self) -> &[ProgressRecord] {
        &self.progress
    }

    /// Returns the completed log once no target remains.
    ///
    /// # Errors
    /// [`CommandError::UnfinishedTargets`] if targets remain.
    pub fn finish(self) -> core::result::Result<ProgressLog, CommandError> {
        if !self.remaining.is_empty() {
            return Err(CommandError::UnfinishedTargets {
                remaining: self.remaining.len(),
            });
        }
        Ok(ProgressLog {
            records: self.progress,
        })
    }
}

/// Structured progress from a completed Tier-1 sequence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgressLog {
    records: Vec<ProgressRecord>,
}

impl ProgressLog {
    /// The ordered canary/target records.
    #[must_use]
    pub fn records(&self) -> &[ProgressRecord] {
        &self.records
    }
}

/// One successful write and request-context re-read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgressRecord {
    target: String,
    phase: ProgressPhase,
    status: Option<i32>,
    reread: String,
}

impl ProgressRecord {
    /// The target identity.
    #[must_use]
    pub fn target(&self) -> &str {
        &self.target
    }

    /// Whether this is the first-target canary or a later target.
    #[must_use]
    pub const fn phase(&self) -> ProgressPhase {
        self.phase
    }

    /// The observed SetProperty status when the transport supplied one.
    #[must_use]
    pub const fn status(&self) -> Option<i32> {
        self.status
    }

    /// The value returned by the target re-read.
    #[must_use]
    pub fn reread(&self) -> &str {
        &self.reread
    }
}

/// Tier-1 sequence phase recorded in [`ProgressRecord`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressPhase {
    /// The mandatory first target.
    Canary,
    /// A target unlocked by all previous verified steps.
    Target,
}

/// Stringly Tier-1 entry point. Unknown and Tier-2 properties fail closed;
/// allowlisted names are canonicalized.
#[derive(Debug)]
pub struct SetProperty;

impl SetProperty {
    /// Plans a validated Tier-1 write.
    ///
    /// # Errors
    ///
    /// Returns [`CommandError::Tier2Refused`] for unapproved properties or a
    /// construction error for invalid components and targets.
    pub fn plan(
        nodes: Vec<String>,
        property: &str,
        value: PropertyValue,
    ) -> core::result::Result<Tier1Write, CommandError> {
        let property = canonical_property(property)?;
        match classify_property(&property) {
            WriteTier::Tier1 => Tier1Write::new(nodes, &property, value),
            WriteTier::Tier2 | WriteTier::Unsanctioned => {
                Err(CommandError::Tier2Refused { property })
            }
        }
    }
}

/// Parsed NAT_RESPONSE (id 3).
///
/// A payload of exactly four bytes is read as an integer status; the
/// SetProperty status layout has not been confirmed against a live server.
/// Any other payload is a textual value: its bytes up to the first NUL, or
/// the whole payload when it has none.
///
/// Motive 3.1.2.1 (NatNet 4.1.0.0) sends GetProperty values without a NUL.
/// In a read-only sweep on 2026-10-10 (55 node-property pairs, each read
/// once), no reply had one; payloads were 1, 4, 5, 8 and 10 bytes long, and
/// `Enabled` arrived as the four bytes `true`. So a four-character value has
/// the status length, and only the request knows which was meant:
/// GetProperty callers should use [`CommandResponse::parse_value`] or
/// [`CommandResponse::parse_value_payload`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandResponse {
    /// A four-byte integer status.
    Status(i32),
    /// A textual value: the payload up to its first NUL, or all of it when
    /// there is none. Invalid UTF-8 is replaced with U+FFFD.
    Value(String),
}

impl CommandResponse {
    /// Parses a complete NAT_RESPONSE datagram.
    ///
    /// # Errors
    /// [`Error::UnexpectedEof`] for a datagram shorter than the 4-byte
    /// header; [`Error::UnexpectedMessageId`] when the header is not
    /// [`NAT_RESPONSE`].
    pub fn parse(datagram: &[u8]) -> Result<Self> {
        Self::parse_payload(message::expect(datagram, NAT_RESPONSE)?)
    }

    /// Parses a NAT_RESPONSE payload by its length alone: four bytes are a
    /// status, any other length a value.
    ///
    /// # Errors
    /// None for any payload; the `Result` keeps the signature stable.
    pub fn parse_payload(payload: &[u8]) -> Result<Self> {
        if payload.len() == 4 {
            let mut reader = Reader::new(payload);
            return Ok(Self::Status(reader.i32()?));
        }
        Self::parse_value_payload(payload).map(Self::Value)
    }

    /// Parses a NAT_RESPONSE datagram known by its request to carry a value.
    ///
    /// # Errors
    /// [`Error::UnexpectedEof`] for a datagram shorter than the 4-byte
    /// header; [`Error::UnexpectedMessageId`] when the header is not
    /// [`NAT_RESPONSE`].
    pub fn parse_value(datagram: &[u8]) -> Result<String> {
        Self::parse_value_payload(message::expect(datagram, NAT_RESPONSE)?)
    }

    /// Parses a request-context value payload at every payload length: the
    /// text up to the first NUL, or the whole payload when it has none.
    ///
    /// # Errors
    /// None for any payload; the `Result` keeps the signature stable.
    pub fn parse_value_payload(payload: &[u8]) -> Result<String> {
        Reader::new(payload).fixed_cstr(payload.len())
    }

    /// Builds a status datagram for tests and mock peers.
    #[must_use]
    pub fn status_datagram(status: i32) -> Vec<u8> {
        message::frame(NAT_RESPONSE, &status.to_le_bytes())
    }

    /// Builds a value datagram for tests and mock peers.
    #[must_use]
    pub fn value_datagram(value: &str) -> Vec<u8> {
        let mut payload = value.as_bytes().to_vec();
        payload.push(0);
        message::frame(NAT_RESPONSE, &payload)
    }
}

/// Command construction, validation, and canary-state errors.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CommandError {
    /// A command component is not representable by the ASCII grammar.
    InvalidComponent {
        /// Which component failed.
        component: CommandComponent,
        /// The validation failure.
        reason: ComponentError,
    },
    /// A request payload, including its NUL terminator, exceeds the NatNet
    /// `u16` declared-size field.
    RequestTooLong {
        /// Offered payload length.
        length: usize,
        /// Maximum representable payload length.
        max: usize,
    },
    /// A target was repeated exactly in one batch.
    DuplicateTarget {
        /// The repeated identity.
        target: String,
    },
    /// A write named no targets.
    NoTargets,
    /// A command operation is not available over NatNet.
    UnsupportedOperation {
        /// Stable operation label.
        operation: &'static str,
    },
    /// A property was not authorized for the stringly Tier-1 path.
    Tier2Refused {
        /// The canonical or supplied property name.
        property: String,
    },
    /// The write status was not the protocol success status.
    StatusFailure {
        /// The target whose write returned the status.
        target: String,
        /// The returned status code.
        status: i32,
    },
    /// The reply shape was not a status where a SetProperty status was needed.
    StatusExpected,
    /// A canary re-read echoed a different value.
    RereadMismatch {
        /// The target whose value differed.
        target: String,
        /// The property being written.
        property: String,
        /// The value written.
        expected: String,
        /// The value observed.
        found: String,
    },
    /// Verification failed after earlier targets had completed. The source
    /// retains the original failure while `progress` keeps the durable records
    /// that preceded it.
    VerificationFailed {
        /// The failure for the current target.
        source: Box<CommandError>,
        /// Ordered records completed before the failure.
        progress: Vec<ProgressRecord>,
    },
    /// The generic decoder saw a status where it could not prove a value.
    UnverifiableReply,
    /// A reply could not be decoded.
    Wire(Error),
    /// A caller attempted to finish while targets remained locked.
    UnfinishedTargets {
        /// Number of targets still requiring a write/re-read.
        remaining: usize,
    },
}

impl CommandError {
    /// Returns records completed before a verification failure.
    #[must_use]
    pub fn progress(&self) -> &[ProgressRecord] {
        match self {
            Self::VerificationFailed { progress, .. } => progress,
            _ => &[],
        }
    }
}

impl core::fmt::Display for CommandError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidComponent { component, reason } => {
                write!(
                    formatter,
                    "invalid {component:?} command component: {reason:?}"
                )
            }
            Self::RequestTooLong { length, max } => {
                write!(formatter, "command payload length {length} exceeds {max}")
            }
            Self::DuplicateTarget { target } => write!(formatter, "duplicate target {target:?}"),
            Self::NoTargets => formatter.write_str("write named no target units"),
            Self::UnsupportedOperation { operation } => {
                write!(
                    formatter,
                    "unsupported NatNet command operation {operation:?}"
                )
            }
            Self::Tier2Refused { property } => write!(
                formatter,
                "Tier-2 or unsanctioned property {property:?} needs an explicit Tier2Grant"
            ),
            Self::StatusFailure { target, status } => {
                write!(
                    formatter,
                    "target {target:?} returned SetProperty status {status}"
                )
            }
            Self::StatusExpected => formatter.write_str("SetProperty reply was not a status"),
            Self::RereadMismatch {
                target,
                property,
                expected,
                found,
            } => write!(
                formatter,
                "target {target:?} re-read mismatch on {property:?}: wrote {expected:?}, read {found:?}"
            ),
            Self::VerificationFailed { source, progress } => write!(
                formatter,
                "Tier-1 verification failed after {} completed target(s): {source}",
                progress.len()
            ),
            Self::UnverifiableReply => {
                formatter.write_str("canary re-read reply was a status, not a value")
            }
            Self::Wire(error) => write!(formatter, "reply decode error: {error}"),
            Self::UnfinishedTargets { remaining } => {
                write!(formatter, "{remaining} targets remain unverified")
            }
        }
    }
}

impl std::error::Error for CommandError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Wire(error) => Some(error),
            Self::VerificationFailed { source, .. } => Some(source.as_ref()),
            _ => None,
        }
    }
}
