use std::thread;
use crate::dl_string::AllocatedDLWString;
use crate::path_processor::process_file_path;

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

//
// pub type FnOpenFile = unsafe extern "system" fn(
//     lpFileName: &mut MicrosoftDiskFileOperator,
//     param_2: u8,
// ) -> usize;
//
// pub static mut OPEN_FILE_ORIGINAL: FnOpenFile = open_file_hook;
//
// #[repr(C)]
// pub struct MicrosoftDiskFileOperator {
//     unk: [u8; 0x38],
//     ptr: *const u16,
//     unk2: usize,
//     unk3: usize,
//     size: usize,
//     unk4: usize,
//     file_handle: usize,
// }
//
// pub unsafe extern "system" fn open_file_hook(
//     lpFileName: &mut MicrosoftDiskFileOperator,
//     param_2: u8,
// ) -> usize {
//     let fileName = U16CStr::from_ptr_str(lpFileName.ptr);
//     if fileName.to_string().unwrap().to_lowercase().contains("game\\data") {
//         println!("OF: {:?}", fileName);
//     }
//
//     OPEN_FILE_ORIGINAL(
//         lpFileName,
//         param_2,
//     )
// }
//
// pub type FnGetFileAttributesW = unsafe extern "system" fn(
//     lpFileName: *mut u16,
// ) -> u32;
//
// pub type FnReadFile = unsafe extern "system" fn(
//     hFile: usize,
//     lpBuffer: *mut u8,
//     nNumberOfBytesToRead: u32,
//     lpNumberOfBytesRead: *mut u32,
//     lpOverlapped: *mut u8,
// ) -> u32;
//
// pub static mut READ_FILE_ORIGINAL: FnReadFile = read_file_hook;
//
// pub unsafe extern "system" fn read_file_hook(
//     hFile: usize,
//     lpBuffer: *mut u8,
//     nNumberOfBytesToRead: u32,
//     lpNumberOfBytesRead: *mut u32,
//     lpOverlapped: *mut u8,
// ) -> u32 {
//     //let fileName = U16CStr::from_ptr_str(lpFileName);
//     //println!("{:?}", fileName);
//
//     READ_FILE_ORIGINAL(
//         hFile,
//         lpBuffer,
//         nNumberOfBytesToRead,
//         lpNumberOfBytesRead,
//         lpOverlapped,
//     )
// }
//
// pub type FnCreateFileW = unsafe extern "system" fn(
//     lpFileName: *const u16,
//     dwDesiredAccess: u32,
//     dwShareMode: u32,
//     lpSecurityAttributes: *const u8,
//     dwCreationDisposition: u32,
//     dwFlagsAndAttributes: u32,
//     hTemplateFile: usize,
// ) -> usize;
//
// pub static mut CREATE_FILE_ORIGINAL: FnCreateFileW = create_file_hook;
//
// pub unsafe extern "system" fn create_file_hook(
//     lpFileName: *const u16,
//     dwDesiredAccess: u32,
//     dwShareMode: u32,
//     lpSecurityAttributes: *const u8,
//     dwCreationDisposition: u32,
//     dwFlagsAndAttributes: u32,
//     hTemplateFile: usize,
// ) -> usize {
//     let fileName = U16CStr::from_ptr_str(lpFileName);
//     if fileName.to_string().unwrap().to_lowercase().contains("regulation") {
//         println!("CF: {:?}", fileName);
//     }
//
//     CREATE_FILE_ORIGINAL(
//         lpFileName,
//         dwDesiredAccess,
//         dwShareMode,
//         lpSecurityAttributes,
//         dwCreationDisposition,
//         dwFlagsAndAttributes,
//         hTemplateFile,
//     )
// }
//
// pub static mut GET_FILE_ATTRIBUTES_ORIGINAL: FnGetFileAttributesW = get_file_attributes_hook;
//
// pub unsafe extern "system" fn get_file_attributes_hook(
//     lpFileName: *mut u16,
// ) -> u32 {
//     let fileName = U16CStr::from_ptr_str(lpFileName);
//     println!("{:?}", fileName);
//     GET_FILE_ATTRIBUTES_ORIGINAL(lpFileName)
// }
//
//
// pub type FnInsertResCap = unsafe extern "C" fn(
//     param_1: usize,
//     param_2: usize,
//     param_3: &FD4ResCap,
//     param_3: u32,
// ) -> &FD4ResCap;
//
// pub static mut INSERT_RES_CAP_ORIGINAL: FnInsertResCap = insert_res_cap_hook;
//
//
// pub unsafe extern "C" fn insert_res_cap_hook(
//     repository: usize,
//     hashable_string: usize,
//     resource_cap: &FD4ResCap,
//     param_3: u32,
// ) -> &FD4ResCap {
//     let cap = INSERT_RES_CAP_ORIGINAL(repository, hashable_string, resource_cap, param_3);
//     // let string = resource_cap
//     //     .resource_string
//     //     .allocated_string
//     //     .string
//     //     .as_ref()
//     //     .to_string()
//     //     .unwrap();
//
//     cap
// }
