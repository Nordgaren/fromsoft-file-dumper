use crate::shutdown;
use log::warn;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{OnceLock, RwLock};
use windows::Win32::Foundation::NTSTATUS;
use windows::Win32::System::Diagnostics::Debug::{
    AddVectoredExceptionHandler, EXCEPTION_POINTERS, PVECTORED_EXCEPTION_HANDLER,
};

type FnRemoveVectoredExceptionHandler = unsafe extern "system" fn(handle: usize) -> u32;
type FnAddVectoredExceptionHandler =
    unsafe extern "system" fn(first: u32, handler: PVECTORED_EXCEPTION_HANDLER) -> usize;

#[derive(Debug)]
struct HandlerInfo {
    handle: usize,
    handler: PVECTORED_EXCEPTION_HANDLER,
}

static mut HANDLE: AtomicUsize = AtomicUsize::new(10000);
static mut HANDLERS: OnceLock<RwLock<Vec<HandlerInfo>>> = OnceLock::new();

unsafe fn get_handlers_mut() -> &'static RwLock<Vec<HandlerInfo>> {
    return HANDLERS.get_or_init(|| RwLock::new(vec![]));
}

const EXCEPTION: NTSTATUS = NTSTATUS(0x406D1388);

pub unsafe fn init_exception_handler() {
    AddVectoredExceptionHandler(1, Some(vectored_exception_handler));
}
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

    let reason = (*(*ExceptionInfo).ExceptionRecord).ExceptionCode;
    if reason != EXCEPTION {
        warn!("Shutting down with reason {:X}", reason.0);
        shutdown();
        //exit(reason.0);
    }

    0
}
pub static mut ADD_VECTORED_EXCEPTION_HANDLER_ORGINAL: FnAddVectoredExceptionHandler =
    AddVectoredExceptionHandler_hook;

pub unsafe extern "system" fn AddVectoredExceptionHandler_hook(
    first: u32,
    handler: PVECTORED_EXCEPTION_HANDLER,
) -> usize {
    let mut handlers = get_handlers_mut().write().unwrap();
    let handle = HANDLE.fetch_add(1, Ordering::Relaxed);

    let info = HandlerInfo { handle, handler };
    if first == 0 {
        handlers.push(info)
    } else {
        handlers.insert(0, info)
    }

    handle
}

pub static mut REMOVE_VECTORED_EXCEPTION_HANDLER_ORGINAL: FnRemoveVectoredExceptionHandler =
    RemoveVectoredExceptionHandler_hook;

pub unsafe extern "system" fn RemoveVectoredExceptionHandler_hook(handle: usize) -> u32 {
    let mut handlers = get_handlers_mut().write().unwrap();

    match handlers.iter().position(|e| e.handle == handle) {
        Some(position) => {
            handlers.remove(position);
            return 0x1;
        }

        // Call the original to prevent messing with stuff that was registered before we hooked
        None => return REMOVE_VECTORED_EXCEPTION_HANDLER_ORGINAL(handle),
    }
}
