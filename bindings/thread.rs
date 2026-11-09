use core::sync::atomic::{AtomicBool, Ordering};

use lock_api::GuardSend;

#[cpp(func = "this_thread::sleep_for")]
pub fn sleep(ms: u32);

#[cpp(func = "this_thread::yield")]
pub fn thread_yield();

pub struct RawMutex(AtomicBool);

unsafe impl lock_api::RawMutex for RawMutex {
    const INIT: Self = Self(AtomicBool::new(false));

    type Guard = GuardSend;

    fn lock(&self) -> Self::Guard {
        loop {
            if let Some(guard) = self.try_lock() {
                return guard;
            }
            thread_yield();
        }
    }

    fn try_lock(&self) -> Option<Self::Guard> {
        self.0
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .ok()
            .map(|_| GuardSend::new())
    }

    unsafe fn unlock(&self, _guard: Self::Guard) {
        self.0.store(false, Ordering::Release);
    }

    fn is_locked(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}

pub type Mutex<T> = lock_api::Mutex<RawMutex, T>;
pub type MutexGuard<'a, T> = lock_api::MutexGuard<'a, RawMutex, T>;
