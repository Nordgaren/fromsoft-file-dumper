use crate::dl_string::AllocatedDLWString;
use crate::path_processor::process_file_path;
use std::thread;

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
    thread::spawn(|| {
        process_file_path(string);
    });

    dlw_string
}
