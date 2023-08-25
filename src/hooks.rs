use crate::dl_string::AllocatedDLWString;
use crate::path_processor::{process_file_path};
use std::thread;
use widestring::U16CString;

pub type FnGetFile = unsafe extern "C" fn(
    path: &AllocatedDLWString,
    p2: u64,
    p3: u64,
    p4: u64,
    p5: u64,
    p6: u64,
) -> &AllocatedDLWString;

pub static mut GET_FILE_ORIGINAL: FnGetFile = get_file_hook;

pub unsafe extern "C" fn get_file_hook(
    path: &AllocatedDLWString,
    p2: u64,
    p3: u64,
    p4: u64,
    p5: u64,
    p6: u64,
) -> &AllocatedDLWString {
    let dlw_string = GET_FILE_ORIGINAL(path, p2, p3, p4, p5, p6);
    let string = dlw_string.string.as_ref().to_string().unwrap();
    println!("{}", string);
    thread::spawn(|| {
        process_file_path(string);
    });

    dlw_string
}

pub type FnHashPath = unsafe extern "C" fn(param_1: usize, param_2: &AllocatedDLWString, param_3: usize, param_4: usize, param_5: usize, param_6: usize) -> usize;
pub static mut HASH_PATH_ORIGINAL: FnHashPath = hash_path_hook;
pub unsafe extern "C" fn hash_path_hook(param_1: usize, path: &AllocatedDLWString, param_3: usize, param_4: usize, param_5: usize, param_6: usize) -> usize {
    let string = path.string.as_ref().to_string().unwrap();
    thread::spawn(move || {
        process_file_path(string);
    });
    return HASH_PATH_ORIGINAL(param_1, path, param_3, param_4, param_5, param_6);
}

pub type FnHashPathTwo = unsafe extern "C" fn(param_1: usize, param_2: *const u16, param_3: usize) -> usize;
pub static mut HASH_PATH_TWO_ORIGINAL: FnHashPathTwo = hash_path_two_hook;

pub unsafe extern "C" fn hash_path_two_hook(param_1: usize, path: *const u16, param_3: usize) -> usize {
    let string = U16CString::from_ptr_str(path).to_string().unwrap();
    thread::spawn(move || {
        process_file_path(string);
    });
    return HASH_PATH_TWO_ORIGINAL(param_1, path, param_3);
}
