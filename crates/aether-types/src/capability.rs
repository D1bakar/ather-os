//! Capability tokens and rights for the Aether security model.

use core::fmt;

/// Magic prefix embedded in every kernel-issued capability identifier.
pub const CAPABILITY_MAGIC: u32 = 0xAE7E_0001;

/// Rights that may be held on a kernel object via a capability.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct CapabilityRights(u8);

impl CapabilityRights {
    /// No rights.
    pub const NONE: Self = Self(0);
    /// Read object contents.
    pub const READ: Self = Self(1 << 0);
    /// Write object contents.
    pub const WRITE: Self = Self(1 << 1);
    /// Map memory backed by the object.
    pub const MAP: Self = Self(1 << 2);
    /// Execute mapped memory.
    pub const EXECUTE: Self = Self(1 << 3);
    /// Delegate rights to another process.
    pub const DELEGATE: Self = Self(1 << 4);
    /// Destroy or signal a process object.
    pub const DESTROY: Self = Self(1 << 5);

    /// Returns `true` if `self` contains all bits in `required`.
    #[must_use]
    pub const fn contains(self, required: Self) -> bool {
        (self.0 & required.0) == required.0
    }

    /// Returns the union of two right sets.
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

/// Kind of kernel object referenced by a capability.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum ObjectKind {
    /// Regular file or directory entry.
    File = 0,
    /// Device node.
    Device = 1,
    /// Memory mapping object.
    Memory = 2,
    /// Another process.
    Process = 3,
}

/// Alias used in syscall dispatch metadata.
pub type ObjectType = ObjectKind;

/// Alias used in syscall dispatch metadata.
pub type Rights = CapabilityRights;

/// Opaque capability identifier issued by the kernel.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(transparent)]
pub struct CapabilityId(u64);

impl CapabilityId {
    /// Creates a kernel-issued capability id from a table slot index.
    #[must_use]
    pub const fn from_slot(slot: u32) -> Self {
        Self(((CAPABILITY_MAGIC as u64) << 32) | slot as u64)
    }

    /// Creates a capability id from a raw user-supplied value.
    #[must_use]
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    /// Returns the raw wire value.
    #[must_use]
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    /// Returns `true` if the magic prefix matches a kernel-issued token.
    #[must_use]
    pub const fn is_valid(self) -> bool {
        (self.0 >> 32) as u32 == CAPABILITY_MAGIC
    }

    /// Returns the table slot encoded in this id.
    #[must_use]
    pub const fn slot(self) -> Option<u32> {
        if self.is_valid() {
            Some(self.0 as u32)
        } else {
            None
        }
    }
}

/// Descriptor stored in the kernel capability table for one slot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CapabilityDescriptor {
    /// Issued capability id.
    pub id: CapabilityId,
    /// Rights currently held on the object.
    pub rights: CapabilityRights,
    /// Object kind this capability references.
    pub object_kind: ObjectKind,
    /// Kernel object handle stub.
    pub object_handle: u32,
}

impl CapabilityDescriptor {
    /// Creates a new descriptor for the given slot and rights.
    #[must_use]
    pub const fn new(slot: u32, rights: CapabilityRights, object_kind: ObjectKind) -> Self {
        Self { id: CapabilityId::from_slot(slot), rights, object_kind, object_handle: slot }
    }

    /// Returns `true` if this descriptor grants all bits in `required`.
    #[must_use]
    pub const fn grants(self, required: CapabilityRights) -> bool {
        self.rights.contains(required)
    }
}

/// Convenience alias for a granted capability descriptor.
pub type Capability = CapabilityDescriptor;

impl fmt::Display for CapabilityRights {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:02x}", self.0)
    }
}
