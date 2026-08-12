//! Scheduler — tasks, round-robin dispatch, idle thread.

mod scheduler;
mod task;

#[cfg(not(feature = "host-stub"))]
pub use scheduler::{
    allocate_process_id, current_kernel_stack_top, current_task_is_user, current_user_entry,
    spawn_init_user_task,
};
pub use scheduler::{
    allocate_task_id, current_process_id, current_task_id, enqueue, init, kernel_stack_top,
    kernel_thread_count, kernel_thread_ids, register_kernel_thread, spawn_worker_thread, start,
    terminate_current, tick_preempt, yield_now, KERNEL_STACK_SIZE,
};
pub use task::{Task, TaskId, TaskState};
