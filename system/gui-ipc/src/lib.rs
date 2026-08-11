//! IPC design stubs for compositor ↔ application communication.
//!
//! **Status:** prototype only — host-side in-memory queues stand in for future
//! kernel message ports and shared-memory capability grants.

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![deny(missing_docs)]

extern crate alloc;

use alloc::collections::VecDeque;
use alloc::string::String;
use core::fmt;

use aether_window::{Rect, WindowId};

/// Current IPC protocol version.
pub const GUI_IPC_VERSION: u32 = 1;

/// High-level message kinds exchanged between compositor and clients.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MessageKind {
    /// Client requests a new window surface.
    CreateWindow = 1,
    /// Client submits a damaged region for repaint.
    Damage = 2,
    /// Compositor acknowledges surface creation.
    SurfaceCreated = 3,
    /// Compositor requests client shutdown.
    CloseRequest = 4,
    /// Heartbeat / ping for bring-up tests.
    Ping = 5,
    /// Heartbeat response.
    Pong = 6,
}

/// Fixed-layout message header shared by all GUI IPC payloads.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MessageHeader {
    /// Must equal [`GUI_IPC_VERSION`].
    pub version: u32,
    /// Target or source window.
    pub window_id: u32,
    /// [`MessageKind`] discriminant.
    pub kind: u8,
    /// Reserved for future flags.
    pub flags: u8,
    /// Reserved padding.
    pub reserved: u16,
}

impl MessageHeader {
    /// Creates a header for `kind` and optional `window_id`.
    #[must_use]
    pub const fn new(kind: MessageKind, window_id: WindowId) -> Self {
        Self {
            version: GUI_IPC_VERSION,
            window_id: window_id.0,
            kind: kind as u8,
            flags: 0,
            reserved: 0,
        }
    }

    /// Parses the kind field when the version matches.
    #[must_use]
    pub const fn kind(&self) -> Option<MessageKind> {
        if self.version != GUI_IPC_VERSION {
            return None;
        }
        match self.kind {
            1 => Some(MessageKind::CreateWindow),
            2 => Some(MessageKind::Damage),
            3 => Some(MessageKind::SurfaceCreated),
            4 => Some(MessageKind::CloseRequest),
            5 => Some(MessageKind::Ping),
            6 => Some(MessageKind::Pong),
            _ => None,
        }
    }
}

/// Payload for [`MessageKind::CreateWindow`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreateWindowRequest {
    /// Requested window title.
    pub title: String,
    /// Initial bounds in screen space.
    pub bounds: Rect,
}

/// Payload for [`MessageKind::Damage`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DamageNotification {
    /// Dirty rectangle relative to the window surface.
    pub region: Rect,
}

/// Fully typed IPC message (host-side prototype representation).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IpcMessage {
    /// Window creation request from a client.
    CreateWindow {
        /// Message header.
        header: MessageHeader,
        /// Request body.
        body: CreateWindowRequest,
    },
    /// Client reports a damaged region.
    Damage {
        /// Message header.
        header: MessageHeader,
        /// Damaged region payload.
        body: DamageNotification,
    },
    /// Compositor created a shared surface for the client.
    SurfaceCreated {
        /// Message header.
        header: MessageHeader,
        /// Shared-memory region descriptor.
        region: SharedSurfaceRegion,
    },
    /// Compositor asks the client to exit.
    CloseRequest {
        /// Message header.
        header: MessageHeader,
    },
    /// Keep-alive probe.
    Ping {
        /// Message header.
        header: MessageHeader,
    },
    /// Keep-alive response.
    Pong {
        /// Message header.
        header: MessageHeader,
    },
}

impl IpcMessage {
    /// Returns the message header.
    #[must_use]
    pub fn header(&self) -> MessageHeader {
        match self {
            Self::CreateWindow { header, .. }
            | Self::Damage { header, .. }
            | Self::SurfaceCreated { header, .. }
            | Self::CloseRequest { header }
            | Self::Ping { header }
            | Self::Pong { header } => *header,
        }
    }
}

/// Descriptor for a shared-memory surface region (design stub).
///
/// On a future bare-metal/userspace build the compositor would grant a
/// capability referencing mapped pages; the host prototype stores a buffer id.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SharedSurfaceRegion {
    /// Opaque buffer identifier in the compositor address space.
    pub buffer_id: u32,
    /// Byte offset of the first pixel within the mapping.
    pub byte_offset: u64,
    /// Mapped byte length.
    pub byte_length: u64,
    /// Surface width in pixels.
    pub width: u32,
    /// Surface height in pixels.
    pub height: u32,
    /// Bytes per scan line.
    pub stride: u32,
}

impl SharedSurfaceRegion {
    /// Creates a descriptor for a newly allocated client buffer.
    #[must_use]
    pub const fn new(buffer_id: u32, width: u32, height: u32, stride: u32) -> Self {
        let byte_length = stride as u64 * height as u64;
        Self { buffer_id, byte_offset: 0, byte_length, width, height, stride }
    }
}

/// In-memory double-ended queue simulating a kernel message port.
#[derive(Clone, Debug, Default)]
pub struct MessageQueue {
    inbound: VecDeque<IpcMessage>,
}

impl MessageQueue {
    /// Creates an empty queue.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Enqueues a message.
    pub fn send(&mut self, message: IpcMessage) {
        self.inbound.push_back(message);
    }

    /// Dequeues the next message, if any.
    pub fn recv(&mut self) -> Option<IpcMessage> {
        self.inbound.pop_front()
    }

    /// Returns the number of pending messages.
    #[must_use]
    pub fn len(&self) -> usize {
        self.inbound.len()
    }

    /// Returns `true` when no messages are pending.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inbound.is_empty()
    }
}

/// Endpoint pairing used by compositor and client prototypes.
#[derive(Clone, Debug, Default)]
pub struct IpcEndpoint {
    to_compositor: MessageQueue,
    to_client: MessageQueue,
    next_buffer_id: u32,
}

impl IpcEndpoint {
    /// Creates a connected compositor/client endpoint pair.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Client → compositor queue.
    pub fn client_send(&mut self, message: IpcMessage) {
        self.to_compositor.send(message);
    }

    /// Compositor receives from clients.
    pub fn compositor_recv(&mut self) -> Option<IpcMessage> {
        self.to_compositor.recv()
    }

    /// Compositor → client queue.
    pub fn compositor_send(&mut self, message: IpcMessage) {
        self.to_client.send(message);
    }

    /// Client receives from compositor.
    pub fn client_recv(&mut self) -> Option<IpcMessage> {
        self.to_client.recv()
    }

    /// Allocates the next shared buffer identifier.
    pub fn alloc_buffer_id(&mut self) -> u32 {
        let id = self.next_buffer_id;
        self.next_buffer_id = self.next_buffer_id.saturating_add(1);
        id
    }
}

/// Errors returned by IPC helpers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IpcError {
    /// The message version or kind was not recognized.
    InvalidMessage,
    /// No message was available.
    WouldBlock,
}

impl fmt::Display for IpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidMessage => write!(f, "invalid IPC message"),
            Self::WouldBlock => write!(f, "no IPC message available"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for IpcError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_header_round_trip() {
        let header = MessageHeader::new(MessageKind::Ping, WindowId(7));
        assert_eq!(header.kind(), Some(MessageKind::Ping));
    }

    #[test]
    fn endpoint_routes_messages() {
        let mut ep = IpcEndpoint::new();
        ep.client_send(IpcMessage::Ping {
            header: MessageHeader::new(MessageKind::Ping, WindowId(0)),
        });
        assert!(ep.compositor_recv().is_some());
        assert!(ep.compositor_recv().is_none());
    }
}
