use crate::SAVE_PATH;
use lazy_static::lazy_static;
use log::warn;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

#[derive(Copy, Clone)]
pub enum Game {
    EldenRing,
    ArmoredCore6,
}

lazy_static! {
    pub static ref REGEX: Regex = Regex::new(r"^([a-zA-Z0-9]*):([^:]*)$").unwrap();
}

pub static mut STRINGS: OnceLock<Mutex<Vec<String>>> = OnceLock::new();
unsafe fn get_vec_mut() -> &'static mut Mutex<Vec<String>> {
    if let Some(vec) = STRINGS.get_mut() {
        return vec;
    }

    STRINGS.get_or_init(|| Mutex::new(vec![]));
    return STRINGS.get_mut().unwrap();
}

pub unsafe fn add_to_vector(string: String) {
    let vec = get_vec_mut().get_mut().unwrap();
    #[cfg(feature = "Console")]
    println!("{string}");
    vec.push(string);
}

pub static mut FILES: OnceLock<HashMap<String, Vec<String>>> = OnceLock::new();

unsafe fn get_hashmap_mut() -> &'static mut HashMap<String, Vec<String>> {
    if let Some(hashmap) = FILES.get_mut() {
        return hashmap;
    }

    FILES.get_or_init(|| init_hashmap(SAVE_PATH));
    return FILES.get_mut().unwrap();
}
pub unsafe fn process_file_paths() {
    let vec = get_vec_mut().get_mut().unwrap();
    let strings = vec.clone();
    vec.clear();
    save_paths_in_vector(strings);
}
pub static mut ARCHIVES: &[&str] = &[];
unsafe fn save_paths_in_vector(strings: Vec<String>) {
    let hashmap = get_hashmap_mut();
    let re = &REGEX;
    for string in strings {
        match re.captures(&string) {
            Some(c) => {
                if c.len() != 3 {
                    warn!("capture len incorrect. {}\n{}", c.len(), string);
                    continue;
                }
                let key = c[1].to_lowercase().to_string();
                if !ARCHIVES.contains(&&key[..]) {
                    continue;
                }
                let val = c[2].to_string();
                match hashmap.get_mut(&key) {
                    None => {
                        hashmap.insert(key, vec![val]);
                        ()
                    }
                    Some(v) => v.push(val),
                }
            }
            None => {
                warn!("Failed to match: {string}");
                continue;
            }
        };
    }
}

pub unsafe fn init_hashmap(path: &str) -> HashMap<String, Vec<String>> {
    let mut hashmap: HashMap<String, Vec<String>> = HashMap::new();

    match fs::read_to_string(path) {
        Ok(f) => {
            let lines = f.lines();
            let mut archive = "".to_string();
            for line in lines {
                if line.starts_with('#') {
                    archive = line[1..].to_string();
                    continue;
                }

                add_to_hashmap(&archive, line, &mut hashmap);
            }
        }
        Err(e) => warn!("file list not found: {e}"),
    };

    return hashmap;
}

pub unsafe fn merge_dicts(path: &str) {
    let hashmap = FILES.get_mut().unwrap();

    match fs::read_to_string(path) {
        Ok(f) => {
            let lines = f.lines();
            let mut archive = "".to_string();
            for line in lines {
                if line.starts_with('#') {
                    archive = line[1..].to_string();
                    continue;
                }

                add_to_hashmap(&archive, line, hashmap);
            }
        }
        Err(e) => warn!("file list not found: {e}"),
    };
}

pub fn add_to_hashmap(archive: &String, line: &str, hashmap: &mut HashMap<String, Vec<String>>) {
    match hashmap.get_mut(archive) {
        Some(v) => v.push(line.to_string()),
        None => {
            let vec = vec![line.to_string()];
            hashmap.insert(archive.clone(), vec);
        }
    }
}

pub unsafe fn save_dump() {
    let path = PathBuf::from(SAVE_PATH);
    let folder_path = path
        .parent()
        .expect(&format!("Could not parse path {SAVE_PATH}"));

    fs::create_dir_all(folder_path).expect(&format!("Could not create path {folder_path:?}"));

    let mut hashmap = match FILES.get_mut() {
        Some(h) => h,
        None => return,
    };
    let mut string = String::new();
    let mut sd = String::new();
    for (key, val) in hashmap.iter_mut() {
        val.sort();
        val.dedup();

        if key == "sd" {
            sd.push_str(&format!("#{}\n", key));
            sd.push_str(&val.join("\n"));
            sd.push('\n');
            continue;
        }
        string.push_str(&format!("#{}\n", key));
        string.push_str(&val.join("\n"));
        string.push('\n');
    }

    string.push_str(&sd);
    fs::write(SAVE_PATH, string).unwrap();
}

#[cfg(test)]
mod tests {
    use crate::path_processor::{init_hashmap, merge_dicts, save_dump, FILES};
    use std::fs;

    #[test]
    fn save_hashmap() {
        unsafe {
            let hashmap =
                init_hashmap(r"G:\Steam\steamapps\common\ELDEN RING\Game\dump\file_paths.txt");
            FILES.set(hashmap).unwrap();

            save_dump();
        }
    }

    #[test]
    fn merge_hashmap() {
        unsafe {
            let hashmap = init_hashmap(r".\dumps\file_paths.txt");
            FILES.set(hashmap).unwrap();

            let files = fs::read_dir(r".\dumps\new\").unwrap();

            for file in files {
                merge_dicts(file.unwrap().path().to_str().unwrap());
            }
            save_dump();
        }
    }
}
