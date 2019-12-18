use crate::cell::UnsafeCell;
use crate::mem::MaybeUninit;
use crate::ptr;
use crate::sync::atomic::{AtomicU8, Ordering::SeqCst};

use super::freertos::*;

pub struct Mutex { inner: UnsafeCell<SemaphoreHandle_t>, initialized: AtomicU8 }

unsafe impl Send for Mutex {}
unsafe impl Sync for Mutex {}

#[allow(dead_code)] // sys isn't exported yet
impl Mutex {
    pub const fn new() -> Mutex {
        Mutex { inner: UnsafeCell::new(ptr::null_mut()), initialized: AtomicU8::new(2) }
    }

    #[inline]
    pub unsafe fn init(&mut self) {
        self.atomic_init();
    }

    #[inline]
    unsafe fn atomic_init(&self) {
        match self.initialized.compare_and_swap(2, 1, SeqCst) {
            2 => {
                *self.inner.get() = xSemaphoreCreateMutex();
                debug_assert!(!(*self.inner.get()).is_null());
                self.initialized.store(0, SeqCst);
            },
            1 => {
                while !self.initialized.load(SeqCst) == 0 {}
            },
            _ => return,
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
        xSemaphoreTake(*self.inner.get(), 0) == 1
    }

    #[inline]
    pub unsafe fn destroy(&self) {
        if self.initialized.load(SeqCst) == 0 {
            vSemaphoreDelete(*self.inner.get());
            *self.inner.get() = ptr::null_mut();
            self.initialized.store(2, SeqCst);
        }
    }
}

impl Drop for Mutex {
    fn drop(&mut self) {
        unsafe { self.destroy() }
    }
}

pub struct ReentrantMutex { inner: UnsafeCell<SemaphoreHandle_t>, initialized: AtomicU8 }

unsafe impl Send for ReentrantMutex {}
unsafe impl Sync for ReentrantMutex {}

impl ReentrantMutex {
    pub const unsafe fn uninitialized() -> ReentrantMutex {
        ReentrantMutex { inner: UnsafeCell::new(ptr::null_mut()), initialized: AtomicU8::new(2) }
    }

    #[inline]
    pub unsafe fn init(&mut self) {
        self.atomic_init();
    }

    #[inline]
    unsafe fn atomic_init(&self) {
        match self.initialized.compare_and_swap(2, 1, SeqCst) {
            2 => {
                *self.inner.get() = xSemaphoreCreateRecursiveMutex();
                debug_assert!(!(*self.inner.get()).is_null());
                self.initialized.store(0, SeqCst);
            },
            1 => {
                while !self.initialized.load(SeqCst) == 0 {}
            },
            _ => return,
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
        xSemaphoreTakeRecursive(*self.inner.get(), 0) == 1
    }

    pub unsafe fn unlock(&self) {
        self.atomic_init();
        let r = xSemaphoreGiveRecursive(*self.inner.get());
        debug_assert_eq!(r, pdTRUE);
    }

    pub unsafe fn destroy(&self) {
        if self.initialized.load(SeqCst) == 0 {
            vSemaphoreDelete(*self.inner.get());
            *self.inner.get() = ptr::null_mut();
            self.initialized.store(2, SeqCst);
        }
    }
}

impl Drop for ReentrantMutex {
    fn drop(&mut self) {
        unsafe { self.destroy() }
    }
}
