use std::env;
use serde_derive::{Serialize, Deserialize};
use std::collections::HashMap;

pub fn get_cache_home() -> String { //get cache directory in compliance with XDG directories
    match env::var("XDG_CACHE_HOME") {
        Ok(x) => x,
        Err(..) => match env::var("HOME") {
            Ok(x) => format!("{x}/.cache"),
            Err(..) => panic!("bailing out, you're on your own")
        }
    }
}

pub fn encode_sort(sort_type: SortType) -> String {//convert a sort type to a string
    match sort_type {
        SortType::Alphabetical => "Alphabetical".to_string(),
        SortType::Reverse => "Reverse".to_string(),
        SortType::Folders => "Folders".to_string(),
        SortType::Files => "Files".to_string(),
    }
}
pub fn decode_sort(sort_type: String) -> SortType {//convert a string into a sort type
    let test = String::as_str(&sort_type);
    match test {
        "Alphabetical" => SortType::Alphabetical,
        "Reverse" => SortType::Reverse,
        "Folders" => SortType::Folders,
        "Files" => SortType::Files,
        &_ => SortType::Folders
    }
}
pub fn get_config_home() -> String {//get the user's config home, in compliance with XDG directories
    match env::var("XDG_CONFIG_HOME") {
        Ok(x) => x,
        Err(..) => match env::var("HOME") {
            Ok(x) => format!("{x}/.config"),
            Err(..) => panic!("bailing out, you're on your own")
        }
    }
}

#[derive(PartialEq)]
pub enum FileType {//enum for what types a file may possess
    Folder,
    File,
    Link
}
#[derive(Serialize, Deserialize, Clone)]
pub struct BookmarkDir {//struct representation of a bookmarked location
    pub name: String,
    pub path: String
}
#[derive(Serialize, Deserialize, Clone)]
pub struct Config {//struct representation of a config file
    pub sort_mode: String,
    pub show_hidden: bool,
    pub bookmarks: Vec<BookmarkDir>,
    pub icn_theme: String,
    pub icn_size: u16
}
#[derive(Serialize, Deserialize, Clone)]
pub struct CacheFile {//struct representation of the cache file
    pub contents: HashMap<String, String>
}
#[derive(Clone)]
pub enum SortType { //enum representing what sort type is preferred
    Alphabetical,
    Reverse,
    Folders,
    Files,
}
#[derive(Serialize, Deserialize)]
pub struct CuttlefishCfg {//struct used in collecting the user's preferred theme
    pub theme: String
}
