//! Task identifiers and per-thread control blocks.

use crate::arch::x86_64::switch::CpuContext;
use crate::process::ProcessId;
use core::ptr::NonNull;

/// Unique task identifier (kernel-wide).
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct TaskId(pub u64);

impl TaskId {
    /// Creates a task id from a raw value.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
}

/// Task lifecycle state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TaskState {
    /// Eligible to run.
    Ready,
    /// Currently executing on a CPU.
    Running,
    /// Blocked on I/O or synchronization (stub — not wired yet).
    Blocked,
    /// Exited; pending teardown.
    Terminated,
}

/// Per-thread control block.
#[repr(C)]
pub struct Task {
    /// Stable task identifier.
    pub id: TaskId,
    /// Scheduling state.
    pub state: TaskState,
    /// Saved CPU context for context switches.
    pub context: CpuContext,
    /// Top of this task's kernel stack (initial RSP).
    pub kernel_stack_top: u64,
    /// Owning process, if any (`None` for pure kernel threads).
    pub process: Option<ProcessId>,
    /// Round-robin run-queue link.
    next: Option<NonNull<Task>>,
}

impl Task {
    /// Creates a new task control block with the given entry point and stack.
    #[must_use]
    pub const fn new(id: TaskId, entry: u64, kernel_stack_top: u64, cr3: u64) -> Self {
        Self {
            id,
            state: TaskState::Ready,
            context: CpuContext::for_entry(entry, kernel_stack_top, cr3),
            kernel_stack_top,
            process: None,
            next: None,
        }
    }

    /// Links this task to `next` in the round-robin queue.
    pub fn set_next(&mut self, next: Option<NonNull<Task>>) {
        self.next = next;
    }

    /// Returns the next pointer in the run queue.
    #[must_use]
    pub fn next(&self) -> Option<NonNull<Task>> {
        self.next
    }
}
