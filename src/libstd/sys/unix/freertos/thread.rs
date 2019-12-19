use crate::cmp;
use crate::ffi::CString;
use crate::io;
use crate::mem;
use crate::ptr;
use crate::sys::mutex::Mutex;
use crate::time::Duration;
use crate::sync::{Arc, atomic::{AtomicUsize, Ordering::SeqCst}};
use crate::sys_common::thread::*;
use crate::pin::Pin;

use crate::sys::ffi::*;

const RUNNING: usize = 0;
const DETACHED: usize = 1;
const EXITED: usize = 2;

pub const DEFAULT_MIN_STACK_SIZE: usize = 4096;

pub struct Thread {
    name: Pin<Box<CString>>,
    id: TaskHandle_t,
    join_mutex: Arc<Mutex>,
    state: Arc<AtomicUsize>,
}

// Some platforms may have pthread_t as a pointer in which case we still want
// a thread to be Send/Sync
unsafe impl Send for Thread {}
unsafe impl Sync for Thread {}

impl Thread {
    // unsafe: see thread::Builder::spawn_unchecked for safety requirements
    pub unsafe fn new(name: Option<CString>, stack: usize, p: Box<dyn FnOnce()>)
                          -> io::Result<Thread> {
        let join_mutex = Arc::new(Mutex::new());
        let state = Arc::new(AtomicUsize::new(RUNNING));

        let arg = box (join_mutex.clone(), state.clone(), box p);

        let name = Box::pin(name.unwrap_or_else(|| CString::from_vec_unchecked(b"rust_thread".to_vec())));

        let mut thread = Thread { name, id: ptr::null_mut(), join_mutex, state };

        let res = xTaskCreatePinnedToCore(
            thread_start,
            thread.name.as_ptr(),
            stack as u32,
            Box::into_raw(arg) as *mut libc::c_void,
            5,
            &mut thread.id,
            tskNO_AFFINITY,
        );

        if res != pdTRUE {
            if res == errCOULD_NOT_ALLOCATE_REQUIRED_MEMORY {
                return Err(io::Error::new(io::ErrorKind::Other, "could not allocate required memory for thread"));
            } else {
                return Err(io::ErrorKind::WouldBlock.into());
            }
        }

        return Ok(thread);

        extern fn thread_start(arg: *mut libc::c_void) -> *mut libc::c_void {
            unsafe {
                let arg = Box::<(Arc<Mutex>, Arc<AtomicUsize>, Box<Box<dyn FnOnce()>>)>::from_raw(arg as *mut _);
                let (join_mutex, state, main) = *arg;

                join_mutex.lock();

                start_thread(Box::into_raw(main) as *mut u8);

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
        let secs = dur.as_secs();
        let nanos = dur.subsec_nanos();

        let mut remaining_ms = u128::from(secs) * 1_000 + (u128::from(nanos) + 999999) / 1000000;

        unsafe {
            let ms_per_tick = u128::from(1000 / xPortGetTickRateHz());

            while remaining_ms > 0 {
                let ticks_to_delay = (remaining_ms + ms_per_tick - 1) / ms_per_tick;

                if ticks_to_delay > u128::from(crate::u32::MAX) {
                    remaining_ms -= ms_per_tick * u128::from(crate::u32::MAX);
                    vTaskDelay(crate::u32::MAX);
                } else {
                    remaining_ms = 0;
                    vTaskDelay(ticks_to_delay as u32);
                }
            }
        }
    }

    pub fn join(self) {
        unsafe {
            assert!(self.id != xTaskGetCurrentTaskHandle());

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
