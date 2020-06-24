use crate::cell::UnsafeCell;
use crate::ptr;
use crate::sync::atomic::{AtomicUsize, Ordering::SeqCst};

use crate::sys::ffi::*;

pub struct Mutex {
    inner: UnsafeCell<SemaphoreHandle_t>,
    initialized: AtomicUsize,
}

unsafe impl Send for Mutex {}
unsafe impl Sync for Mutex {}

const UNINITIALIZING: usize = 3;
const UNINITIALIZED: usize = 2;
const INITIALIZING: usize = 1;
const INITIALIZED: usize = 0;

#[allow(dead_code)] // sys isn't exported yet
impl Mutex {
    pub const fn new() -> Mutex {
        Mutex { inner: UnsafeCell::new(ptr::null_mut()), initialized: AtomicUsize::new(UNINITIALIZED) }
    }

    #[inline]
    pub unsafe fn init(&mut self) {
        self.atomic_init();
    }

    #[inline]
    unsafe fn atomic_init(&self) {
        loop {
            match self.initialized.compare_and_swap(UNINITIALIZED, INITIALIZING, SeqCst) {
                UNINITIALIZED => {
                    *self.inner.get() = xSemaphoreCreateMutex();
                    debug_assert!(!(*self.inner.get()).is_null());
                    self.initialized.store(INITIALIZED, SeqCst);
                    return;
                }
                INITIALIZED => return,
                _ => continue,
            }
        }
    }

    #[inline]
    pub unsafe fn lock(&self) {
        self.atomic_init();
        let r = xSemaphoreTake(*self.inner.get(), TickType_t::max_value());
        debug_assert_eq!(r, pdTRUE);
    }

    #[inline]
    pub unsafe fn unlock(&self) {
        self.atomic_init();
        let r = xSemaphoreGive(*self.inner.get());
        debug_assert_eq!(r, pdTRUE);
    }

    #[inline]
    pub unsafe fn try_lock(&self) -> bool {
        self.atomic_init();
        xSemaphoreTake(*self.inner.get(), 0) == pdTRUE
    }

    #[inline]
    pub unsafe fn destroy(&self) {
        loop {
            match self.initialized.compare_and_swap(INITIALIZED, UNINITIALIZING, SeqCst) {
                INITIALIZED => {
                    vSemaphoreDelete(*self.inner.get());
                    *self.inner.get() = ptr::null_mut();
                    self.initialized.store(UNINITIALIZED, SeqCst);
                    return;
                }
                UNINITIALIZED => return,
                _ => continue,
            }
        }
    }
}

impl Drop for Mutex {
    fn drop(&mut self) {
        unsafe { self.destroy() }
    }
}

pub struct ReentrantMutex {
    inner: UnsafeCell<SemaphoreHandle_t>,
    initialized: AtomicUsize,
}

unsafe impl Send for ReentrantMutex {}
unsafe impl Sync for ReentrantMutex {}

impl ReentrantMutex {
    pub const unsafe fn uninitialized() -> ReentrantMutex {
        ReentrantMutex {
            inner: UnsafeCell::new(ptr::null_mut()),
            initialized: AtomicUsize::new(UNINITIALIZED),
        }
    }

    #[inline]
    pub unsafe fn init(&self) {
        self.atomic_init();
    }

    #[inline]
    unsafe fn atomic_init(&self) {
        loop {
            match self.initialized.compare_and_swap(UNINITIALIZED, INITIALIZING, SeqCst) {
                UNINITIALIZED => {
                    *self.inner.get() = xSemaphoreCreateRecursiveMutex();
                    debug_assert!(!(*self.inner.get()).is_null());
                    self.initialized.store(INITIALIZED, SeqCst);
                    return;
                }
                INITIALIZED => return,
                _ => continue,
            }
        }
    }

    #[inline]
    pub unsafe fn lock(&self) {
        self.atomic_init();
        let r = xSemaphoreTakeRecursive(*self.inner.get(), TickType_t::max_value());
        debug_assert_eq!(r, pdTRUE);
    }

    #[inline]
    pub unsafe fn try_lock(&self) -> bool {
        self.atomic_init();
        xSemaphoreTakeRecursive(*self.inner.get(), 0) == pdTRUE
    }

    pub unsafe fn unlock(&self) {
        self.atomic_init();
        let r = xSemaphoreGiveRecursive(*self.inner.get());
        debug_assert_eq!(r, pdTRUE);
    }

    pub unsafe fn destroy(&self) {
        loop {
            match self.initialized.compare_and_swap(INITIALIZED, UNINITIALIZING, SeqCst) {
                INITIALIZED => {
                    vSemaphoreDelete(*self.inner.get());
                    *self.inner.get() = ptr::null_mut();
                    self.initialized.store(UNINITIALIZED, SeqCst);
                    return;
                }
                UNINITIALIZED => return,
                _ => continue,
            }
        }
    }
}

impl Drop for ReentrantMutex {
    fn drop(&mut self) {
        unsafe { self.destroy() }
    }
}
