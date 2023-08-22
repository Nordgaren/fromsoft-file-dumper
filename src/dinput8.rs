#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(unused)]

use crate::forward_function;
use core::ffi::c_void;
use paste::paste;
use std::arch::asm;
use std::mem;
use windows::core::PCSTR;
use windows::imp::{GetProcAddress, LoadLibraryA};
use windows::Win32::Foundation::MAX_PATH;
use windows::Win32::System::SystemInformation::GetWindowsDirectoryA;

pub type fnDirectInput8Create = unsafe extern "system" fn(
    hinst: usize,
    dwVersion: u32,
    riidltf: usize,
    ppvOut: *mut usize,
    punkOuter: usize,
) -> u32;
pub type fnDllCanUnloadNow = unsafe extern "system" fn() -> u32;
pub type fnDllGetClassObject =
    unsafe extern "system" fn(rclsid: usize, riid: usize, ppv: usize) -> u32;
pub type fnDllRegisterServer = unsafe extern "system" fn() -> u32;
pub type fnDllUnregisterServer = unsafe extern "system" fn() -> u32;
pub type fnGetdfDIJoystick = unsafe extern "system" fn() -> usize;

forward_function!(DirectInput8Create);
forward_function!(DllCanUnloadNow);
forward_function!(DllGetClassObject);
forward_function!(DllRegisterServer);
forward_function!(DllUnregisterServer);
forward_function!(GetdfDIJoystick);

pub unsafe fn init_dinput8() {
    let mut buffer = [0u8; MAX_PATH as usize + 1];
    let path_size = GetWindowsDirectoryA(Some(&mut buffer)) as usize;
    let dir = &buffer[..path_size];
    let dinput = LoadLibraryA(PCSTR::from_raw(
        format!(
            "{}\\System32\\dinput8.dll\0",
            std::str::from_utf8(dir).unwrap_or_default()
        )
        .as_ptr(),
    ));

    if dinput == 0 {
        panic!("Could not find dinput8.dll.");
    }

    pDirectInput8Create = GetProcAddress(dinput, PCSTR::from_raw(b"DirectInput8Create\0".as_ptr()));
    pDllCanUnloadNow = GetProcAddress(dinput, PCSTR::from_raw(b"DllCanUnloadNow\0".as_ptr()));
    pDllGetClassObject = GetProcAddress(dinput, PCSTR::from_raw(b"DllGetClassObject\0".as_ptr()));
    pDllRegisterServer = GetProcAddress(dinput, PCSTR::from_raw(b"DllRegisterServer\0".as_ptr()));
    pDllUnregisterServer =
        GetProcAddress(dinput, PCSTR::from_raw(b"DllUnregisterServer\0".as_ptr()));
    pGetdfDIJoystick = GetProcAddress(dinput, PCSTR::from_raw(b"GetdfDIJoystick\0".as_ptr()));
}
