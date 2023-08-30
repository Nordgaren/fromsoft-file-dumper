use log::warn;
use std::cell::{Cell, OnceCell};
use std::ffi::c_void;
use std::mem;
use std::process::exit;
use std::ptr::addr_of;
use std::sync::{Mutex, OnceLock, RwLock};
use std::sync::atomic::{AtomicUsize, Ordering};
use windows::Win32::Foundation::{NTSTATUS, STATUS_ACCESS_VIOLATION};
use windows::Win32::System::Diagnostics::Debug::{
    AddVectoredExceptionHandler, RemoveVectoredExceptionHandler, EXCEPTION_POINTERS,
    PVECTORED_EXCEPTION_HANDLER,
};
use windows::Win32::System::Kernel::ExceptionContinueExecution;
use crate::shutdown;

// thread_local! {
//     static GAME_HANDLER: Cell<PVECTORED_EXCEPTION_HANDLER> = Cell::new(None);
//     static VEH_HANDLE: Cell<Option<*mut c_void>> = Cell::new(None);
// }
#[derive(Debug)]
struct HandlerInfo {
    handle: usize,
    handler: PVECTORED_EXCEPTION_HANDLER,
}

static mut HANDLE: AtomicUsize = AtomicUsize::new(10000);
static mut HANDLERS: OnceLock<RwLock<Vec<HandlerInfo>>> = OnceLock::new();

unsafe fn get_handlers_mut() -> &'static mut RwLock<Vec<HandlerInfo>> {
    if let Some(vec) = HANDLERS.get_mut() {
        return vec;
    }

    HANDLERS.get_or_init(|| RwLock::new(vec![]));
    return HANDLERS.get_mut().unwrap();
}

const EXCEPTION: NTSTATUS = NTSTATUS(0x406D1388);
// The Handler
pub unsafe extern "system" fn vectored_exception_handler(
    ExceptionInfo: *mut EXCEPTION_POINTERS,
) -> i32 {

    let mut handlers = get_handlers_mut().read().unwrap();
    for handler in handlers.iter() {
        let fun = handler.handler;
        if let Some(exception_handler) = handler.handler {
            let result = exception_handler(ExceptionInfo);
            if result == -1 {
                return result;
            }
        }
    }

    let reason =(*(*ExceptionInfo).ExceptionRecord).ExceptionCode;
    if reason != EXCEPTION {
        warn!("Shutting down with reason {:X}", reason.0);
        shutdown();
        //exit(reason.0);
    }

    0
}

pub unsafe extern "system" fn AddVectoredExceptionHandler_hook(
    first: u32,
    handler: PVECTORED_EXCEPTION_HANDLER,
) -> usize {
    let mut handlers = get_handlers_mut().get_mut().unwrap();
    let handle = HANDLE.fetch_add(1, Ordering::Relaxed);

    let info = HandlerInfo { handle, handler };
    if first == 0 {
        handlers.push(info)
    } else {
        handlers.insert(0, info)
    }

    handle
}

pub unsafe extern "system" fn RemoveVectoredExceptionHandler_hook(handle: usize) -> u32 {

    let mut handlers = get_handlers_mut()
        .write()
        .unwrap();

    match handlers.iter().position(|e| e.handle == handle) {
        Some(position) => {
            handlers.remove(position);
            return 0x1;
        },

        // Call the original to prevent messing with stuff that was registered before we hooked
        None => return RemoveVectoredExceptionHandler(handle as *const c_void),
    }
}
