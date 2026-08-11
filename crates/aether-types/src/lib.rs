//! Shared types used across Aether OS components.

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod address;
mod audit;
mod boot_info;
mod capability;
mod error;
mod page;
mod result;
mod security_config;
mod user;
mod user_ptr;

pub use address::{PhysicalAddress, VirtualAddress};
pub use audit::{AuditEventKind, AuditRecord, AuditSeverity};
pub use boot_info::{
    BootInfo, FramebufferInfo, MemoryMapEntry, SerialPortInfo, BOOT_INFO_MAGIC, BOOT_INFO_VERSION,
    MEMORY_TYPE_CONVENTIONAL,
};
pub use capability::{
    Capability, CapabilityDescriptor, CapabilityId, CapabilityRights, ObjectKind, ObjectType,
    Rights, CAPABILITY_MAGIC,
};
pub use error::{AetherError, ErrorCode};
pub use page::{PageFlags, PageSize};
pub use result::{from_error_code, to_error_code, AetherResult};
pub use security_config::SecurityDefaults;
pub use user::{
    is_kernel_address, is_user_address, USER_SPACE_MAX, USER_SPACE_MIN, USER_STACK_TOP,
};
pub use user_ptr::{
    is_canonical_user_address, is_non_user_address, validate_user_address, validate_user_buffer,
    validate_user_path_ptr, UserBuffer, USER_ADDRESS_MAX, USER_ADDRESS_MIN,
};
