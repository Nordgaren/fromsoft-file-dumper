#![feature(naked_functions)]
#![allow(non_snake_case)]

mod HashableString;
mod config;
mod dl_string;
mod exception_handler;
mod game_consts;
mod hooks;
mod path_processor;

use crate::config::*;
use crate::exception_handler::{
    init_exception_handler, AddVectoredExceptionHandler_hook, RemoveVectoredExceptionHandler_hook,
    ADD_VECTORED_EXCEPTION_HANDLER_ORGINAL, REMOVE_VECTORED_EXCEPTION_HANDLER_ORGINAL,
};
use crate::game_consts::*;
use crate::hooks::{
    hash_path_hook, hash_path_two_hook, HASH_PATH_ORIGINAL, HASH_PATH_TWO_ORIGINAL,
};
use crate::path_processor::Game::{ArmoredCore6, EldenRing};
use crate::path_processor::{process_file_paths, save_dump, Game, ARCHIVES};
use fisherman::hook::builder::HookBuilder;
use fisherman::scanner::signature::Signature;
use fisherman::scanner::simple_scanner::SimpleScanner;
use fisherman::util::get_module_slice;
use log::*;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Logger, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::*;
use std::sync::atomic::Ordering;
use std::{fs, mem, thread};
use windows::Win32::Foundation::{HMODULE, MAX_PATH};
#[cfg(feature = "Console")]
use windows::Win32::System::Console::{AllocConsole, AttachConsole};
use windows::Win32::System::LibraryLoader::{GetModuleFileNameA, GetModuleHandleA};
use windows::Win32::System::SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH};

#[no_mangle]
#[allow(unused)]
pub extern "stdcall" fn DllMain(hinstDLL: isize, dwReason: u32, lpReserved: *mut usize) -> i32 {
    match dwReason {
        DLL_PROCESS_ATTACH => unsafe {
            #[cfg(feature = "Console")]
            {
                AllocConsole();
                AttachConsole(u32::MAX);
            }
            init_logs(LOG_PATH);
            let path = init(hinstDLL);
            init_hooks(&path);
            init_loop();
            init_exception_handler();
            //init_exit_callback();
            1
        },
        DLL_PROCESS_DETACH => {
            unsafe {
                shutdown();
            }
            1
        }
        _ => 0,
    }
}

unsafe fn shutdown() {
    if END.load(Ordering::Relaxed) {
        return;
    }
    END.store(true, Ordering::Relaxed);
    process_file_paths();
    save_dump();
}

unsafe fn init_loop() {
    thread::spawn(|| loop {
        thread::sleep(SLEEP_DURATION);
        if END.load(Ordering::Relaxed) {
            break;
        }
        process_file_paths();
        save_dump();
    });
}

pub fn init_logs(file: &str) {
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d(%Y-%m-%d %H:%M:%S)} | {({l}):5.5} | {f}:{L} — {m}{n}",
        )))
        .build();

    let file = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d(%Y-%m-%d %H:%M:%S)} | {({l}):5.5} | {f}:{L} — {m}{n}",
        )))
        .build(file)
        .expect("Could not init file appender");

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(Appender::builder().build("file", Box::new(file)))
        .logger(Logger::builder().build("fromsoft-file-print", LevelFilter::Trace))
        .logger(Logger::builder().build("file", LevelFilter::Trace))
        .build(
            Root::builder()
                .appender("stdout")
                .appender("file")
                .build(LevelFilter::Trace),
        )
        .expect("Could not init log config");

    init_config(config).unwrap();

    log_panics::init();
}

unsafe fn init(hinstDLL: isize) -> String {
    let mut buffer = [0u8; MAX_PATH as usize + 1];
    let name_size = GetModuleFileNameA(HMODULE(hinstDLL), &mut buffer) as usize;
    let name = &buffer[..name_size];
    let name_str = std::str::from_utf8(name).expect("Could not parse name from GetModuleFileNameA");
    name_str.to_string()
}

unsafe fn init_hooks(name: &str) {
    let base = GetModuleHandleA(None).unwrap().0 as usize;

    let module_slice = get_module_slice(base);

    let signature = Signature::from_ida_pattern(HASH_FILE_ONE).unwrap();
    let offset = SimpleScanner
        .scan(module_slice, &signature)
        .expect("Could not find signature.");
    let hash_path_one = base as isize + offset as isize;

    let signature2 = Signature::from_ida_pattern(HASH_FILE_TWO).unwrap();
    let offset2 = SimpleScanner
        .scan(module_slice, &signature2)
        .expect("Could not find signature.");
    let hash_path_two = base as isize + offset2 as isize;

    let hook = HookBuilder::new()
        .add_inline_hook(
            hash_path_one as usize,
            hash_path_hook as usize,
            &mut HASH_PATH_ORIGINAL,
            None,
        )
        .add_inline_hook(
            hash_path_two as usize,
            hash_path_two_hook as usize,
            &mut HASH_PATH_TWO_ORIGINAL,
            None,
        )
        .add_iat_hook(
            "KERNEL32.dll",
            "AddVectoredExceptionHandler",
            AddVectoredExceptionHandler_hook as usize,
        )
        .add_iat_hook(
            "KERNEL32.dll",
            "RemoveVectoredExceptionHandler",
            RemoveVectoredExceptionHandler_hook as usize,
        )
        .build();

    ADD_VECTORED_EXCEPTION_HANDLER_ORGINAL = mem::transmute(
        hook.get_original_func_addr_iat("AddVectoredExceptionHandler")
            .unwrap(),
    );
    REMOVE_VECTORED_EXCEPTION_HANDLER_ORGINAL = mem::transmute(
        hook.get_original_func_addr_iat("RemoveVectoredExceptionHandler")
            .unwrap(),
    );
}

fn set_archives(game: Game) {
    unsafe {
        match game {
            EldenRing => ARCHIVES = &["data0", "data1", "data2", "data3", "sd"],
            ArmoredCore6 => ARCHIVES = &["data0", "data1", "data2", "data3", "sd"],
        }
    }
}

fn get_game() -> Game {
    if fs::read(format!("./{ELDEN_RING_EXE}")).is_ok() {
        return EldenRing;
    }

    if fs::read(format!("./{ARMORED_CORE_EXE}")).is_ok() {
        return ArmoredCore6;
    }

    panic!("Could not find game");
}
