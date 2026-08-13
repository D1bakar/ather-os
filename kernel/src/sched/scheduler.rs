//! Round-robin scheduler, idle task, and kernel-thread registry.

use super::task::{Task, TaskId, TaskState};
#[cfg(all(not(feature = "host-stub"), target_arch = "x86_64"))]
use crate::arch::x86_64::switch::CpuContext;
use core::ptr::NonNull;
use core::sync::atomic::{AtomicU64, Ordering};

#[cfg(all(not(feature = "host-stub"), target_arch = "x86_64"))]
use crate::arch::x86_64::switch::switch_context;

/// Default kernel stack size for idle and bring-up threads (4 KiB).
pub const KERNEL_STACK_SIZE: usize = 4096;

static NEXT_TASK_ID: AtomicU64 = AtomicU64::new(1);
#[cfg(not(feature = "host-stub"))]
static NEXT_PROCESS_ID: AtomicU64 = AtomicU64::new(1);

static mut CURRENT: Option<NonNull<Task>> = None;
static mut RUN_QUEUE_HEAD: Option<NonNull<Task>> = None;
#[cfg(not(feature = "host-stub"))]
static mut IDLE_TASK: Option<Task> = None;
#[cfg(not(feature = "host-stub"))]
static mut IDLE_STACK: [u8; KERNEL_STACK_SIZE] = [0; KERNEL_STACK_SIZE];
#[cfg(not(feature = "host-stub"))]
static mut WORKER_TASK: Option<Task> = None;
#[cfg(not(feature = "host-stub"))]
static mut WORKER_STACK: [u8; KERNEL_STACK_SIZE] = [0; KERNEL_STACK_SIZE];
#[cfg(not(feature = "host-stub"))]
static mut BOOT_CTX: CpuContext =
    CpuContext { rbx: 0, rbp: 0, r12: 0, r13: 0, r14: 0, r15: 0, rsp: 0, rip: 0, cr3: 0 };

/// Intrusive list of kernel threads (includes idle).
static mut KERNEL_THREADS: [Option<NonNull<Task>>; MAX_KERNEL_THREADS] = [None; MAX_KERNEL_THREADS];
static mut KERNEL_THREAD_COUNT: usize = 0;

const MAX_KERNEL_THREADS: usize = 16;

#[cfg(not(feature = "host-stub"))]
static mut SYSCALL_STACK_TOP: u64 = 0;

/// Initializes the scheduler, idle task, and empty run queue.
#[cfg(not(feature = "host-stub"))]
pub fn init() {
    // SAFETY: Called once on the BSP before other tasks exist.
    unsafe {
        let cr3 = CpuContext::current_cr3();
        let stack_top = core::ptr::addr_of_mut!(IDLE_STACK) as u64 + KERNEL_STACK_SIZE as u64;

        IDLE_TASK = Some(Task::new(allocate_task_id(), idle_entry as u64, stack_top, cr3));

        let idle_ptr =
            NonNull::from((&mut *core::ptr::addr_of_mut!(IDLE_TASK)).as_mut().expect("idle task"));
        (*idle_ptr.as_ptr()).state = TaskState::Ready;

        register_kernel_thread(idle_ptr);
        RUN_QUEUE_HEAD = Some(idle_ptr);
        SYSCALL_STACK_TOP = stack_top;

        #[cfg(all(not(feature = "host-stub"), target_arch = "x86_64"))]
        crate::arch::x86_64::gdt::set_kernel_stack(stack_top);
    }
}

/// Creates the demo worker kernel thread and enqueues it behind the idle task.
#[cfg(not(feature = "host-stub"))]
pub fn spawn_worker_thread() -> TaskId {
    // SAFETY: Called once on the BSP before scheduling starts.
    unsafe {
        let cr3 = CpuContext::current_cr3();
        let stack_top = core::ptr::addr_of_mut!(WORKER_STACK) as u64 + KERNEL_STACK_SIZE as u64;
        let id = allocate_task_id();
        WORKER_TASK = Some(Task::new(id, worker_entry as u64, stack_top, cr3));
        let worker_ptr = NonNull::from(
            (&mut *core::ptr::addr_of_mut!(WORKER_TASK)).as_mut().expect("worker task"),
        );
        register_kernel_thread(worker_ptr);
        enqueue(worker_ptr);
        id
    }
}

/// Host-stub no-op worker spawn (M0 CI).
#[cfg(feature = "host-stub")]
#[must_use]
pub fn spawn_worker_thread() -> TaskId {
    allocate_task_id()
}

/// Enables interrupts and performs the first context switch into the idle task.
///
/// Does not return on bare metal — execution continues in the scheduled tasks.
#[cfg(not(feature = "host-stub"))]
pub fn start() -> ! {
    crate::serial::write_str("Aether OS M4: scheduler initialized\r\n");
    crate::arch::x86_64::enable_interrupts();

    // SAFETY: Idle task and boot context are initialized; no other tasks run yet.
    unsafe {
        let idle_ptr =
            NonNull::from((&mut *core::ptr::addr_of_mut!(IDLE_TASK)).as_mut().expect("idle task"));
        let boot_ctx = core::ptr::addr_of_mut!(BOOT_CTX);
        let first = pick_next(idle_ptr);
        if first != idle_ptr {
            // Round-robin into worker/init immediately; idle alone only HLTs and
            // would otherwise depend on a timer tick before ring-3 init runs.
            (*idle_ptr.as_ptr()).state = TaskState::Ready;
            (*first.as_ptr()).state = TaskState::Running;
            CURRENT = Some(first);
            let first_ctx = &(*first.as_ptr()).context as *const CpuContext;
            switch_context(boot_ctx, first_ctx);
        } else {
            (*idle_ptr.as_ptr()).state = TaskState::Running;
            CURRENT = Some(idle_ptr);
            let idle_ctx = &(*idle_ptr.as_ptr()).context as *const CpuContext;
            switch_context(boot_ctx, idle_ctx);
        }
    }

    loop {
        unsafe {
            core::arch::asm!("hlt", options(nomem, nostack, preserves_flags));
        }
    }
}

/// Host-stub no-op scheduler start (M0 CI).
#[cfg(feature = "host-stub")]
pub fn start() -> ! {
    panic!("sched::start must not run in host-stub builds");
}

/// Host-stub no-op scheduler init (M0 CI).
#[cfg(feature = "host-stub")]
pub fn init() {}

#[cfg(not(feature = "host-stub"))]
static mut INIT_USER_TASK: Option<Task> = None;
#[cfg(not(feature = "host-stub"))]
static mut INIT_USER_STACK: [u8; KERNEL_STACK_SIZE] = [0; KERNEL_STACK_SIZE];

/// Creates the init user task and enqueues it on the run queue.
#[cfg(not(feature = "host-stub"))]
pub fn spawn_init_user_task(
    task_id: TaskId,
    pid: crate::process::ProcessId,
    cr3: u64,
    user_rip: u64,
    user_rsp: u64,
) {
    // SAFETY: Called once on the BSP before scheduling starts.
    unsafe {
        let stack_top = core::ptr::addr_of_mut!(INIT_USER_STACK) as u64 + KERNEL_STACK_SIZE as u64;
        let kernel_cr3 = CpuContext::current_cr3();
        INIT_USER_TASK = Some(Task::new_user(
            task_id,
            user_task_trampoline as u64,
            stack_top,
            kernel_cr3,
            user_rip,
            user_rsp,
            pid,
        ));
        let _ = cr3;
        let task_ptr = NonNull::from(
            (&mut *core::ptr::addr_of_mut!(INIT_USER_TASK)).as_mut().expect("init user task"),
        );
        register_kernel_thread(task_ptr);
        enqueue(task_ptr);
    }
}

/// Returns `(user_rip, user_rsp, cr3)` for the currently running user task.
#[cfg(not(feature = "host-stub"))]
#[must_use]
pub fn current_user_entry() -> Option<(u64, u64, u64)> {
    // SAFETY: Read-only access to CURRENT.
    unsafe {
        CURRENT.and_then(|t| {
            let task = &*t.as_ptr();
            if task.is_user_task() {
                Some((task.user_rip, task.user_rsp, task.context.cr3))
            } else {
                None
            }
        })
    }
}

/// Returns the kernel stack top for the currently running task.
#[cfg(not(feature = "host-stub"))]
#[must_use]
pub fn current_kernel_stack_top() -> Option<u64> {
    // SAFETY: Read-only access to CURRENT.
    unsafe { CURRENT.map(|t| (*t.as_ptr()).kernel_stack_top) }
}

/// Returns the kernel stack top used for syscall entry (idle task stack).
#[cfg(not(feature = "host-stub"))]
#[must_use]
pub fn kernel_stack_top() -> u64 {
    // SAFETY: Written once during `init`.
    unsafe { SYSCALL_STACK_TOP }
}

/// Host-stub placeholder stack top.
#[cfg(feature = "host-stub")]
#[must_use]
pub fn kernel_stack_top() -> u64 {
    0
}

/// Returns the next process id (for future user-process bring-up).
#[cfg(not(feature = "host-stub"))]
#[must_use]
pub fn allocate_process_id() -> crate::process::ProcessId {
    crate::process::ProcessId(NEXT_PROCESS_ID.fetch_add(1, Ordering::Relaxed) as u32)
}

/// Allocates a new task id.
#[must_use]
pub fn allocate_task_id() -> TaskId {
    TaskId(NEXT_TASK_ID.fetch_add(1, Ordering::Relaxed))
}

#[cfg(test)]
pub(crate) unsafe fn set_run_queue_head_for_test(head: Option<NonNull<Task>>) {
    RUN_QUEUE_HEAD = head;
}

/// Adds a runnable task to the round-robin queue tail.
///
/// # Safety
///
/// `task` must remain valid for the lifetime of the scheduler.
pub unsafe fn enqueue(task: NonNull<Task>) {
    (*task.as_ptr()).state = TaskState::Ready;

    match RUN_QUEUE_HEAD {
        None => {
            (*task.as_ptr()).set_next(Some(task));
            RUN_QUEUE_HEAD = Some(task);
        }
        Some(head) => {
            // Walk the ring until the predecessor of `head` (queue tail).
            let mut tail = head;
            while let Some(next) = (*tail.as_ptr()).next() {
                if next == head {
                    break;
                }
                tail = next;
            }
            (*tail.as_ptr()).set_next(Some(task));
            (*task.as_ptr()).set_next(Some(head));
        }
    }
}

/// Registers a kernel thread in the global list (for diagnostics and debugging).
///
/// # Safety
///
/// `task` must remain valid for the lifetime of the scheduler.
pub unsafe fn register_kernel_thread(task: NonNull<Task>) {
    if KERNEL_THREAD_COUNT < MAX_KERNEL_THREADS {
        KERNEL_THREADS[KERNEL_THREAD_COUNT] = Some(task);
        KERNEL_THREAD_COUNT += 1;
    }
}

/// Returns the number of registered kernel threads.
#[must_use]
pub fn kernel_thread_count() -> usize {
    // SAFETY: Read-only counter after init.
    unsafe { KERNEL_THREAD_COUNT }
}

/// Returns registered kernel thread ids written into `buffer`.
#[must_use]
pub fn kernel_thread_ids(buffer: &mut [TaskId]) -> usize {
    // SAFETY: Read-only access to stable task ids.
    unsafe {
        let count = KERNEL_THREAD_COUNT.min(buffer.len());
        for (index, slot) in buffer.iter_mut().enumerate().take(count) {
            if let Some(task) = KERNEL_THREADS[index] {
                *slot = (*task.as_ptr()).id;
            }
        }
        count
    }
}

/// Voluntary yield — round-robin to the next runnable task.
#[cfg(not(feature = "host-stub"))]
pub fn yield_now() {
    schedule_internal(false);
}

/// Host-stub no-op voluntary yield (M0 CI).
#[cfg(feature = "host-stub")]
pub fn yield_now() {}

/// Timer-driven preemption — skipped while a ring-3 user task is running (M6).
#[cfg(not(feature = "host-stub"))]
pub fn tick_preempt() {
    if current_task_is_user() {
        return;
    }
    schedule_internal(true);
}

/// Host-stub no-op timer preemption (M0 CI).
#[cfg(feature = "host-stub")]
pub fn tick_preempt() {}

/// Returns the currently running task id, if any.
#[must_use]
pub fn current_task_id() -> Option<TaskId> {
    // SAFETY: Read-only access to CURRENT.
    unsafe { CURRENT.map(|t| (*t.as_ptr()).id) }
}

/// Returns the owning process id of the currently running task, if set.
#[must_use]
pub fn current_process_id() -> Option<crate::process::ProcessId> {
    // SAFETY: Read-only access to CURRENT.
    unsafe { CURRENT.and_then(|t| (*t.as_ptr()).process) }
}

/// Marks the running task terminated and yields to the next runnable task.
///
/// Does not return on bare metal when another task is runnable.
#[cfg(not(feature = "host-stub"))]
pub fn terminate_current() {
    // SAFETY: Scheduler state is only mutated with interrupts managed by callers.
    unsafe {
        let current = match CURRENT {
            Some(c) => c,
            None => return,
        };
        (*current.as_ptr()).state = TaskState::Terminated;
    }
    yield_now();
}

/// Host-stub no-op process termination (M5 CI).
#[cfg(feature = "host-stub")]
pub fn terminate_current() {}

#[cfg(not(feature = "host-stub"))]
fn schedule_internal(_from_timer: bool) {
    // SAFETY: Scheduler state is only mutated with interrupts managed by callers.
    unsafe {
        let current = match CURRENT {
            Some(c) => c,
            None => return,
        };

        let next = pick_next(current);
        if next == current {
            return;
        }

        (*current.as_ptr()).state = TaskState::Ready;
        (*next.as_ptr()).state = TaskState::Running;
        CURRENT = Some(next);

        #[cfg(target_arch = "x86_64")]
        {
            let cur_ctx = &mut (*current.as_ptr()).context as *mut CpuContext;
            let next_ctx = &(*next.as_ptr()).context as *const CpuContext;
            switch_context(cur_ctx, next_ctx);
        }
        // Resumed on this task's stack when scheduled again (bare metal).
    }
}

#[cfg(not(feature = "host-stub"))]
unsafe fn pick_next(current: NonNull<Task>) -> NonNull<Task> {
    let start = current;
    let mut cursor = match (*current.as_ptr()).next() {
        Some(n) => n,
        None => return current,
    };
    loop {
        if (*cursor.as_ptr()).state != TaskState::Terminated {
            return cursor;
        }
        cursor = match (*cursor.as_ptr()).next() {
            Some(n) if n != start => n,
            _ => return current,
        };
    }
}

/// Returns whether the currently running task executes user code in ring 3.
#[cfg(not(feature = "host-stub"))]
#[must_use]
pub fn current_task_is_user() -> bool {
    // SAFETY: Read-only access to CURRENT.
    unsafe { CURRENT.is_some_and(|t| (*t.as_ptr()).is_user_task()) }
}

#[cfg(not(feature = "host-stub"))]
extern "C" fn user_task_trampoline() -> ! {
    let (user_rip, user_rsp, _) = current_user_entry().expect("user task");
    let user_cr3 = crate::process::with_current(|proc| proc.page_table_root.as_u64())
        .expect("user process page table");
    let kernel_stack = current_kernel_stack_top().expect("user kernel stack");
    crate::arch::x86_64::gdt::set_kernel_stack(kernel_stack);
    crate::arch::x86_64::set_syscall_handler_stack(kernel_stack);
    // SAFETY: Validated during ELF load and task setup.
    unsafe {
        crate::arch::x86_64::enter_user_mode(user_rip, user_rsp, user_cr3);
    }
}

#[cfg(not(feature = "host-stub"))]
extern "C" fn worker_entry() -> ! {
    loop {
        crate::serial::write_str("[worker] kernel thread tick\r\n");
        yield_now();
    }
}

#[cfg(not(feature = "host-stub"))]
extern "C" fn idle_entry() -> ! {
    loop {
        unsafe {
            core::arch::asm!("hlt", options(nomem, nostack, preserves_flags));
        }
    }
}

#[cfg(test)]
fn pick_next_task(current: NonNull<Task>) -> NonNull<Task> {
    // SAFETY: Test-only helper mirroring production round-robin policy.
    unsafe {
        let start = current;
        let mut cursor = match (*current.as_ptr()).next() {
            Some(n) => n,
            None => return current,
        };
        loop {
            if (*cursor.as_ptr()).state != TaskState::Terminated {
                return cursor;
            }
            cursor = match (*cursor.as_ptr()).next() {
                Some(n) if n != start => n,
                _ => return current,
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocate_task_ids_are_unique() {
        let a = allocate_task_id();
        let b = allocate_task_id();
        assert_ne!(a, b);
    }

    #[test]
    fn round_robin_ring_selects_next_task() {
        let mut idle = Task::new(TaskId(1), 0, 0x1000, 0);
        let mut worker = Task::new(TaskId(2), 0, 0x2000, 0);

        let idle_ptr = NonNull::from(&mut idle);
        let worker_ptr = NonNull::from(&mut worker);
        idle.set_next(Some(worker_ptr));
        worker.set_next(Some(idle_ptr));

        assert_eq!(pick_next_task(idle_ptr), worker_ptr);
        assert_eq!(pick_next_task(worker_ptr), idle_ptr);
    }

    #[test]
    fn boot_queue_starts_with_worker_after_idle() {
        let mut idle = Task::new(TaskId(1), 0, 0x1000, 0);
        let mut worker = Task::new(TaskId(2), 0, 0x2000, 0);
        let mut init = Task::new(TaskId(3), 0, 0x3000, 0);

        let idle_ptr = NonNull::from(&mut idle);
        let worker_ptr = NonNull::from(&mut worker);
        let init_ptr = NonNull::from(&mut init);

        unsafe {
            set_run_queue_head_for_test(Some(idle_ptr));
            enqueue(worker_ptr);
            enqueue(init_ptr);
        }

        assert_eq!(pick_next_task(idle_ptr), worker_ptr);
    }

    #[test]
    fn enqueue_extends_circular_run_queue() {
        let mut idle = Task::new(TaskId(1), 0, 0x1000, 0);
        let mut worker = Task::new(TaskId(2), 0, 0x2000, 0);
        let mut init = Task::new(TaskId(3), 0, 0x3000, 0);

        let idle_ptr = NonNull::from(&mut idle);
        let worker_ptr = NonNull::from(&mut worker);
        let init_ptr = NonNull::from(&mut init);

        unsafe {
            set_run_queue_head_for_test(Some(idle_ptr));
            enqueue(worker_ptr);
            enqueue(init_ptr);
        }

        assert_eq!(idle.next(), Some(worker_ptr));
        assert_eq!(worker.next(), Some(init_ptr));
        assert_eq!(init.next(), Some(idle_ptr));
    }

    #[test]
    fn round_robin_skips_terminated_tasks() {
        let mut idle = Task::new(TaskId(1), 0, 0x1000, 0);
        let mut worker = Task::new(TaskId(2), 0, 0x2000, 0);
        let mut exited = Task::new(TaskId(3), 0, 0x3000, 0);
        exited.state = TaskState::Terminated;

        let idle_ptr = NonNull::from(&mut idle);
        let worker_ptr = NonNull::from(&mut worker);
        let exited_ptr = NonNull::from(&mut exited);
        idle.set_next(Some(exited_ptr));
        exited.set_next(Some(worker_ptr));
        worker.set_next(Some(idle_ptr));

        assert_eq!(pick_next_task(idle_ptr), worker_ptr);
    }
}
