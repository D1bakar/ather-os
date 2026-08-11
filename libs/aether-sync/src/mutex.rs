//! Busy-wait spin mutex backed by an atomic flag.

use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};

/// A mutual exclusion primitive that spins until the lock becomes available.
///
/// Suitable for kernel critical sections where the holder never blocks and
/// hold times are bounded.
pub struct SpinMutex<T> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
}

impl<T> SpinMutex<T> {
    /// Creates a new unlocked mutex containing `value`.
    pub const fn new(value: T) -> Self {
        Self { locked: AtomicBool::new(false), data: UnsafeCell::new(value) }
    }

    /// Acquires the lock, spinning until it becomes available.
    pub fn lock(&self) -> SpinMutexGuard<'_, T> {
        while self
            .locked
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            while self.locked.load(Ordering::Relaxed) {
                core::hint::spin_loop();
            }
        }

        SpinMutexGuard { lock: self }
    }

    /// Attempts to acquire the lock without spinning.
    ///
    /// Returns [`None`] if another thread holds the lock.
    pub fn try_lock(&self) -> Option<SpinMutexGuard<'_, T>> {
        if self.locked.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed).is_ok() {
            Some(SpinMutexGuard { lock: self })
        } else {
            None
        }
    }

    /// Returns a mutable reference to the inner data.
    ///
    /// The caller must ensure exclusive access (e.g. single-threaded boot code).
    pub fn get_mut(&mut self) -> &mut T {
        self.data.get_mut()
    }
}

// SAFETY: `T` is Send; the lock serializes access to `data`.
unsafe impl<T: Send> Send for SpinMutex<T> {}

// SAFETY: `T` is Send; only one guard can exist at a time due to the atomic lock.
unsafe impl<T: Send> Sync for SpinMutex<T> {}

/// RAII guard returned by [`SpinMutex::lock`] and [`SpinMutex::try_lock`].
pub struct SpinMutexGuard<'a, T> {
    lock: &'a SpinMutex<T>,
}

impl<T> Deref for SpinMutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        // SAFETY: The guard holds the lock; no other accessor can exist.
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> DerefMut for SpinMutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: The guard holds the lock; no other accessor can exist.
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T> Drop for SpinMutexGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.locked.store(false, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lock_and_unlock() {
        let mutex = SpinMutex::new(0u32);
        {
            let mut guard = mutex.lock();
            *guard = 42;
        }
        let guard = mutex.lock();
        assert_eq!(*guard, 42);
    }

    #[test]
    fn try_lock_succeeds_when_free() {
        let mutex = SpinMutex::new(());
        assert!(mutex.try_lock().is_some());
    }

    #[test]
    fn try_lock_fails_when_held() {
        let mutex = SpinMutex::new(());
        let _guard = mutex.lock();
        assert!(mutex.try_lock().is_none());
    }

    #[test]
    fn const_constructor() {
        static M: SpinMutex<u8> = SpinMutex::new(7);
        let guard = M.lock();
        assert_eq!(*guard, 7);
    }
}
