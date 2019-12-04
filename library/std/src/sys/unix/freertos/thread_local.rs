use crate::collections::{LinkedList, BTreeMap};
use crate::ptr;
use crate::sys::ffi::*;
use crate::sys_common::rwlock::RWLock;

pub type Key = usize;
type Tls = BTreeMap<Key, *mut u8>;

const TLS_INDEX: BaseType_t = 1;

static LOCK: RWLock = RWLock::new();
static mut KEYS: LinkedList<(Key, Option<unsafe extern "C" fn(*mut u8)>)> = LinkedList::new();

#[inline]
unsafe fn get_tls() -> *mut Tls {
    pvTaskGetThreadLocalStoragePointer(ptr::null_mut(), TLS_INDEX) as *mut Tls
}

#[inline]
unsafe fn set_tls(tls: *mut Tls) {
    vTaskSetThreadLocalStoragePointer(ptr::null_mut(), TLS_INDEX, tls as *mut _);
}

#[inline]
pub unsafe fn cleanup() {
    if let Some(tls) = get_tls().as_mut() {
        let tls = Box::from_raw(tls);

        LOCK.read();

        for (k, dtor) in KEYS.iter() {
            if let (Some(v), Some(dtor)) = (tls.get(k), dtor) {
                dtor(*v)
            }
        }

        LOCK.read_unlock();

        set_tls(ptr::null_mut());
    }
}

#[inline]
pub unsafe fn create(dtor: Option<unsafe extern "C" fn(*mut u8)>) -> Key {
    LOCK.write();

    let key = KEYS.back().map(|&(i, _)| i + 1).unwrap_or(1);
    KEYS.push_back((key, dtor));

    LOCK.write_unlock();

    key
}

#[inline]
pub unsafe fn set(key: Key, value: *mut u8) {
    #[cfg(debug_assertions)]
    {
        LOCK.read();

        let mut found = false;

        for (k, _) in KEYS.iter() {
            if k == key {
                found = true;
                break;
            }
        }

        assert!(found);

        LOCK.read_unlock();
    }


    let tls = if let Some(tls) = get_tls().as_mut() {
        tls
    } else {
        let tls = Box::into_raw(box Tls::new());
        set_tls(tls);
        &mut *tls
    };

    if value.is_null() {
        tls.remove(&key);
    } else {
        tls.insert(key, value);
    }
}

#[inline]
pub unsafe fn get(key: Key) -> *mut u8 {
    if let Some(tls) = get_tls().as_mut() {
        tls.get(&key).cloned().unwrap_or_else(ptr::null_mut)
    } else {
        ptr::null_mut()
    }
}

#[inline]
pub unsafe fn destroy(key: Key) {
    LOCK.write();

    let mut i = 0;
    let mut found = false;

    for &(k, _) in KEYS.iter() {
        if k == key {
            found = true;
            break;
        }

        i += 1;
    }

    debug_assert!(found);

    if found {
        KEYS.remove(i);
    }

    LOCK.write_unlock();
}

#[inline]
pub fn requires_synchronized_create() -> bool {
    false
}
