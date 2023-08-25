#![feature(naked_functions)]
#![allow(non_snake_case)]

mod HashableString;
mod dl_string;
mod hooks;
mod path_processor;

use crate::hooks::{get_file_hook, hash_path_hook, HASH_PATH_ORIGINAL, hash_path_two_hook, HASH_PATH_TWO_ORIGINAL};
use crate::path_processor::Game::{ArmoredCore6, EldenRing};
use crate::path_processor::{save_dump, Game, ARCHIVES};
use fisherman::hook::builder::HookBuilder;
use fisherman::scanner::signature::Signature;
use fisherman::scanner::simple_scanner::SimpleScanner;
use fisherman::util::{get_module_slice, get_relative_pointer};
use log::*;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Logger, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::*;
use std::fs;
use windows::Win32::Foundation::{HMODULE, MAX_PATH};
#[cfg(feature = "Console")]
use windows::Win32::System::Console::{AllocConsole, AttachConsole};
use windows::Win32::System::LibraryLoader::{GetModuleFileNameA, GetModuleHandleA};
use windows::Win32::System::SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH};

pub static mut ROOT_DIR: String = String::new();
pub static SAVE_PATH: &str = "./log/file_paths.txt";
pub static LOG_PATH: &str = "log/file-logger.log";
static ELDEN_RING_EXE: &str = "eldenring.exe";
static ARMORED_CORE_EXE: &str = "armoredcore6.exe";
static GET_FILE_ER_SIGNATURE: &str = "E8 ?? ?? ?? ?? 48 83 7B 20 08 48 8D 4B 08 72 03 48 8B 09 4C 8B 4B 18 41 b8 05 00 00 00 4D 3B C8";
static GET_FILE_AC_SIGNATURE: &str = "48 89 5c 24 08 48 89 74 24 10 57 48 83 ec 30 48 8b da 49 8b f1 48 83 c2 08 48 8b f9 48 83 7a 18 08";

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
            //init_exit_callback();
            1
        },
        DLL_PROCESS_DETACH => {
            unsafe {
                save_dump();
            }
            1
        }
        _ => 0,
    }
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
    let mut end = name.rfind("\\");
    if end == None {
        end = name.rfind("/");
    }

    let i = end.expect(&format!("Could not find parent directory. {name}"));
    let root_dir = &name[..i + 1];
    ROOT_DIR = root_dir.to_string();

    let base = GetModuleHandleA(None).unwrap().0 as usize;

    let game = get_game();

    set_archives(game);
    let module_slice = get_module_slice(base);
    let signature = get_function_signature(game);
    let offset = SimpleScanner
        .scan(module_slice, &signature)
        .expect("Could not find signature.");

    let callsite = base as isize + offset as isize;
    let addr = get_relative_pointer(callsite, 1, 5) as *const u8 as usize;

    let signature2 = Signature::from_ida_pattern("40 55 57 48 8b ec 48 83 ec 68").unwrap();
    let offset2 = SimpleScanner
        .scan(module_slice, &signature2)
        .expect("Could not find signature.");

    let callsite2 = base as isize + offset as isize;

    HookBuilder::new()
        .add_inline_hook(callsite as usize, hash_path_hook as usize, &mut HASH_PATH_ORIGINAL, None)
        .add_inline_hook(callsite2 as usize, hash_path_two_hook as usize, &mut HASH_PATH_TWO_ORIGINAL, None)
        .build();
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

fn get_function_signature(game: Game) -> Signature {
    match game {
        EldenRing => Signature::from_ida_pattern(GET_FILE_ER_SIGNATURE)
            .expect("Could not parse Elden Ring AoB"),
        ArmoredCore6 => Signature::from_ida_pattern(GET_FILE_AC_SIGNATURE)
            .expect("Could not parse Armored Core 6 AoB"),
    }
}
