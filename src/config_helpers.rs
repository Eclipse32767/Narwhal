use std::env;
use std::fs;
use iced_style::theme::Palette;
use iced::Color;
use serde_derive::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::ThemeFile;
use crate::CustomTheme;
use crate::ButtonStyle;
use crate::col_from_string;
pub fn get_cache_home() -> String { //get cache directory in compliance with XDG directories
    match env::var("XDG_CACHE_HOME") {
        Ok(x) => x,
        Err(..) => match env::var("HOME") {
            Ok(x) => format!("{x}/.cache"),
            Err(..) => panic!("bailing out, you're on your own")
        }
    }
}
pub fn get_theme_file() -> CustomTheme {
    let home = get_config_home();//read theme file
    match fs::read_to_string(format!("{home}/Oceania/theme.toml")) {
        Ok(file) => { // if it succeeds, construct a theme according to file
            let theme_file: ThemeFile = toml::from_str(&file).unwrap();
            CustomTheme {
                application: Palette {
                    background: col_from_string(theme_file.bg_color1.clone()),
                    text: col_from_string(theme_file.txt_color.clone()),
                    primary: col_from_string(theme_file.blue.clone()),
                    success: col_from_string(theme_file.green.clone()),
                    danger: col_from_string(theme_file.red.clone()),
                },
                sidebar: ButtonStyle {
                    border_radius: 2.0,
                    txt_color: col_from_string(theme_file.txt_color.clone()),
                    bg_color: Some(col_from_string(theme_file.bg_color2.clone())),
                    border_color: Color::from_rgb8(0, 0, 0),
                    border_width: 0.0,
                    shadow_offset: iced::Vector {x: 0.0, y: 0.0}
                },
                secondary: ButtonStyle {
                    border_radius: 2.0,
                    txt_color: col_from_string(theme_file.txt_color.clone()),
                    bg_color: Some(col_from_string(theme_file.bg_color3.clone())),
                    border_color: Color::from_rgb8(0, 0, 0),
                    border_width: 0.0,
                    shadow_offset: iced::Vector {x: 0.0, y: 0.0}
                },
            }
        }
        Err(..) => {
            CustomTheme {//if it fails, use the light theme
                application: Palette {
                    background: Color::from_rgb8(0xE0, 0xF5, 0xFF),
                    text: Color::from_rgb8(0x00, 0x19, 0x36),
                    primary: Color::from_rgb8(0x00, 0xF1, 0xD6),
                    success: Color::from_rgb8(0xFF, 0x4C, 0x00),
                    danger: Color::from_rgb8(0xFF, 0x4C, 0x00),
                },
                sidebar: ButtonStyle { 
                    border_radius: 2.0,
                    txt_color: Color::from_rgb8( 0x00, 0x19, 0x36),
                    bg_color: Some(Color::from_rgb8(0xD2, 0xF0, 0xFF)),
                    border_color: Color::from_rgb8(0, 0, 0),
                    border_width: 0.0,
                    shadow_offset: iced::Vector {x: 0.0, y: 0.0}
                },
                secondary: ButtonStyle {
                    border_radius: 2.0,
                    txt_color: Color::from_rgb8(0x00, 0x20, 0x46),
                    bg_color: Some(Color::from_rgb8(0xC6, 0xEC, 0xFF)),
                    border_color: Color::from_rgb8(0, 0, 0),
                    border_width: 0.0,
                    shadow_offset: iced::Vector {x: 0.0, y: 0.0}
                },
            }
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
pub enum ThemeType {//enum for selecting the desired theme
    Light,
    Dark,
    Custom
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
pub fn get_set_theme() -> ThemeType {//function to retrieve the user's theme preference
    let home = format!("{}/Oceania/cfg.toml", get_config_home());
    match fs::read_to_string(home) {
        Ok(x) => {
            let cfg: CuttlefishCfg = toml::from_str(&x).unwrap();
            let theme_str = cfg.theme.clone();
            if theme_str == String::from("dark") {
                ThemeType::Dark
            } else if theme_str == String::from("custom") {
                ThemeType::Custom
            } else {
                ThemeType::Light
            }
        }
        Err(..) => ThemeType::Light
    }
}