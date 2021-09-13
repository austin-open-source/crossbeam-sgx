//! Miscellaneous utilities.

use std::cell::{Cell, UnsafeCell};
use std::num::Wrapping;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
#[cfg(any(feature = "mesalock_sgx", target_env = "sgx"))]
use std::time::{Duration, Instant};
#[cfg(all(not(feature = "mesalock_sgx"), not(target_env = "sgx")))]
use std::time::{Duration, Instant};

use crossbeam_utils::Backoff;

#[cfg(any(feature = "mesalock_sgx", target_env = "sgx"))]
use std::untrusted::time::InstantEx;

/// Randomly shuffles a slice.
pub(crate) fn shuffle<T>(v: &mut [T]) {
    let len = v.len();
    if len <= 1 {
        return;
    }

    thread_local! {
        static RNG: Cell<Wrapping<u32>> = Cell::new(Wrapping(1_406_868_647));
    }

    let _ = RNG.try_with(|rng| {
        for i in 1..len {
            // This is the 32-bit variant of Xorshift.
            //
            // Source: https://en.wikipedia.org/wiki/Xorshift
            let mut x = rng.get();
            x ^= x << 13;
            x ^= x >> 17;
            x ^= x << 5;
            rng.set(x);

            let x = x.0;
            let n = i + 1;

            // This is a fast alternative to `let j = x % n`.
            //
            // Author: Daniel Lemire
            // Source: https://lemire.me/blog/2016/06/27/a-fast-alternative-to-the-modulo-reduction/
            let j = ((x as u64).wrapping_mul(n as u64) >> 32) as u32 as usize;

            v.swap(i, j);
        }
    });
}

/// Sleeps until the deadline, or forever if the deadline isn't specified.
pub(crate) fn sleep_until(deadline: Option<Instant>) {
    loop {
        match deadline {
            None => thread::sleep(Duration::from_secs(1000)),
            Some(d) => {
                let now = Instant::now();
                if now >= d {
                    break;
                }
                thread::sleep(d - now);
            }
        }
    }
}

// https://github.com/crossbeam-rs/crossbeam/issues/795
pub(crate) fn convert_timeout_to_deadline(timeout: Duration) -> Instant {
    match Instant::now().checked_add(timeout) {
        Some(deadline) => deadline,
        None => Instant::now() + Duration::from_secs(86400 * 365 * 30),
    }
}

/// A simple spinlock.
pub(crate) struct Spinlock<T> {
    flag: AtomicBool,
    value: UnsafeCell<T>,
}

impl<T> Spinlock<T> {
    /// Returns a new spinlock initialized with `value`.
    pub(crate) fn new(value: T) -> Spinlock<T> {
        Spinlock {
            flag: AtomicBool::new(false),
            value: UnsafeCell::new(value),
        }
    }

    /// Locks the spinlock.
    pub(crate) fn lock(&self) -> SpinlockGuard<'_, T> {
        let backoff = Backoff::new();
        while self.flag.swap(true, Ordering::Acquire) {
            backoff.snooze();
        }
        SpinlockGuard { parent: self }
    }
}

/// A guard holding a spinlock locked.
pub(crate) struct SpinlockGuard<'a, T> {
    parent: &'a Spinlock<T>,
}

impl<T> Drop for SpinlockGuard<'_, T> {
    fn drop(&mut self) {
        self.parent.flag.store(false, Ordering::Release);
    }
}

impl<T> Deref for SpinlockGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.parent.value.get() }
    }
}

impl<T> DerefMut for SpinlockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.parent.value.get() }
    }
}
