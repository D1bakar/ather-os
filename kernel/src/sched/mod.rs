//! Scheduler — tasks, round-robin dispatch, idle thread.

mod scheduler;
mod task;

pub use scheduler::{
    allocate_task_id, current_task_id, enqueue, init, kernel_stack_top, kernel_thread_count,
    kernel_thread_ids, register_kernel_thread, spawn_worker_thread, start, tick_preempt, yield_now,
    KERNEL_STACK_SIZE,
};
pub use task::{Task, TaskId, TaskState};
