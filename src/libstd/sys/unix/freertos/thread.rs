use crate::ffi::CStr;
use crate::io;
use crate::mem;
use crate::ptr;
use crate::sys::mutex::Mutex;
use crate::time::Duration;
use crate::sync::{Arc, atomic::{AtomicU8, Ordering::SeqCst}};

use crate::sys::ffi::*;
use crate::sys::thread_local;

const PENDING: u8 = 0;
const RUNNING: u8 = 1;
const DETACHED: u8 = 2;
const EXITED: u8 = 3;

pub const DEFAULT_MIN_STACK_SIZE: usize = 4096;

pub struct Thread {
    id: TaskHandle_t,
    join_mutex: Arc<Mutex>,
    state: Arc<AtomicU8>,
}

unsafe impl Send for Thread {}
unsafe impl Sync for Thread {}

impl Thread {
    // unsafe: see thread::Builder::spawn_unchecked for safety requirements
    pub unsafe fn new(name: Option<&CStr>, stack: usize, p: Box<dyn FnOnce()>)
                          -> io::Result<Thread> {
        let join_mutex = Arc::new(Mutex::new());
        let state = Arc::new(AtomicU8::new(PENDING));

        let arg = box (join_mutex.clone(), state.clone(), box p);

        let name = name.unwrap_or_else(|| CStr::from_bytes_with_nul_unchecked(b"\0"));

        let mut thread = Thread { id: ptr::null_mut(), join_mutex, state };

        let arg = Box::into_raw(arg);

        let res = xTaskCreate(
            thread_start,
            name.as_ptr(),
            stack as u32,
            arg as *mut libc::c_void,
            5,
            &mut thread.id,
        );

        if res != pdTRUE {
            if thread.state.load(SeqCst) == PENDING {
                drop(Box::from_raw(arg));
            }

            if res == errCOULD_NOT_ALLOCATE_REQUIRED_MEMORY {
                return Err(io::Error::new(io::ErrorKind::Other, "could not allocate required memory for thread"));
            } else {
                return Err(io::ErrorKind::WouldBlock.into());
            }
        }

        return Ok(thread);

        extern fn thread_start(arg: *mut libc::c_void) -> *mut libc::c_void {
            unsafe {
                let arg = Box::<(Arc<Mutex>, Arc<AtomicU8>, Box<Box<dyn FnOnce()>>)>::from_raw(arg as *mut _);
                let (join_mutex, state, main) = *arg;

                join_mutex.lock();

                state.store(RUNNING, SeqCst);

                main();
                thread_local::cleanup();

                let previous_state = state.swap(EXITED, SeqCst);

                join_mutex.unlock();

                // We drop these here manually since we don't know
                // if `vTaskDelete` will ensure they are dropped.
                drop(state);
                drop(join_mutex);

                if previous_state == DETACHED {
                    vTaskDelete(ptr::null_mut());
                } else {
                    vTaskSuspend(ptr::null_mut());
                }
            }

            ptr::null_mut()
        }
    }

    pub fn yield_now() {
        unsafe { vTaskDelay(0) };
    }

    pub fn sleep(dur: Duration) {
        let tick_rate = unsafe { xPortGetTickRateHz() };

        let mut ticks_to_delay: u64 = dur
            .as_secs()
            .checked_mul(u64::from(tick_rate))
            .and_then(|ms| ms.checked_add(u64::from(dur.subsec_nanos() / tick_rate)))
            .expect("overflow converting duration to ticks");

        while ticks_to_delay > u64::from(crate::u32::MAX) {
            ticks_to_delay -= u64::from(crate::u32::MAX);
            unsafe { vTaskDelay(crate::u32::MAX) };
        }

        unsafe { vTaskDelay(ticks_to_delay as u32) };
    }

    pub fn join(self) {
        unsafe {
            assert!(self.id != xTaskGetCurrentTaskHandle());

            while self.state.load(SeqCst) == PENDING {}

            // Just wait for the thread to finish, the rest is handled by `Drop`.
            self.join_mutex.lock();
            self.join_mutex.unlock();
        }
    }

    pub fn id(&self) -> TaskHandle_t { self.id }

    pub fn into_id(self) -> TaskHandle_t {
        let id = self.id;
        mem::forget(self);
        id
    }
}

impl Drop for Thread {
    fn drop(&mut self) {
        // If the thread already exited before we could set the state to `DETACHED`, delete it manually here.
        if self.state.swap(DETACHED, SeqCst) == EXITED {
            unsafe {
                vTaskDelete(self.id);
            }
        }
    }
}

#[cfg_attr(test, allow(dead_code))]
pub mod guard {
    use crate::ops::Range;
    pub type Guard = Range<usize>;
    pub unsafe fn current() -> Option<Guard> { None }
    pub unsafe fn init() -> Option<Guard> { None }
}
