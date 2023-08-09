use crate::Narwhal;
use crate::confighelpers::get_theme_file;
use crate::sort_file_by_type;
use std::{env, fs};
use toml;
use crate::CacheFile;
use std::collections::HashMap;
use crate::get_cache_home;
use crate::get_config_home;
use crate::Config;
use crate::decode_sort;
use crate::get_set_theme;
use iced::Color;
use crate::CustomTheme;
use crate::ThemeSet;
use crate::ButtonStyle;
use iced::futures::executor::block_on;

impl Default for Narwhal {
    fn default() -> Self {
        let current_dir = match env::current_dir() {
            Ok(x) => x,
            Err(x) => panic!("{}", x)
        };
        let cache_home = format!("{}/NarwhalFM", get_cache_home());
        let cache_text = fs::read_to_string(cache_home);
        let cache_struct: CacheFile = match cache_text {
            Ok(x) => toml::from_str(&x).unwrap(),
            Err(..) => CacheFile { contents: HashMap::new() }
        };
        let config_home = format!("{}/Oceania/NarwhalFM.toml", get_config_home());
        let config_text = fs::read_to_string(config_home);
        let config_struct: Config = match config_text {
            Ok(x) => toml::from_str(&x).unwrap(),
            Err(..) => Config { sort_mode: "Folder".to_string(), show_hidden: false, bookmarks: vec![] }
        };
        let mut finalstruct = Narwhal { 
            files: vec![], 
            currentpath: current_dir, 
            sorttype: decode_sort(config_struct.sort_mode), 
            desired_cols: 5, 
            show_hidden: config_struct.show_hidden, 
            desired_rows: 5, 
            last_clicked_file: None, 
            uifiles: vec![], 
            icon_cache: cache_struct.contents.clone(), 
            bookmarked_dirs: config_struct.bookmarks.clone(), 
            deletion_confirmation: false, 
            mv_target: None, 
            cp_target: None,
            theme: get_set_theme(),
            themes: ThemeSet {
                light: CustomTheme {
                    application: iced::theme::Palette {
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
                },
                dark: CustomTheme { // TODO: set dark theme properly
                    application: iced::theme::Palette {
                        background: Color::from_rgb8(0x00, 0x19, 0x36),
                        text: Color::from_rgb8(0xE0, 0xF5, 0xFF),
                        primary: Color::from_rgb8(0x00, 0xCD, 0xB6),
                        success: Color::from_rgb8(1, 1, 1),
                        danger: Color::from_rgb8(0xC5, 0x3A, 0x00),
                    },
                    sidebar: ButtonStyle { 
                        border_radius: 2.0,
                        txt_color: Color::from_rgb8( 0xE0, 0xF5, 0xFF),
                        bg_color: Some(Color::from_rgb8(0x00, 0x20, 0x46)),
                        border_color: Color::from_rgb8(0, 0, 0),
                        border_width: 0.0,
                        shadow_offset: iced::Vector {x: 0.0, y: 0.0}
                    },
                    secondary: ButtonStyle {
                        border_radius: 2.0,
                        txt_color: Color::from_rgb8(0xE0, 0xF5, 0xFF),
                        bg_color: Some(Color::from_rgb8(0x00, 0x29, 0x58)),
                        border_color: Color::from_rgb8(0, 0, 0),
                        border_width: 0.0,
                        shadow_offset: iced::Vector {x: 0.0, y: 0.0}
                    },
                },
                custom: get_theme_file()
            }
        };
        finalstruct.regen_files();
        sort_file_by_type(&mut finalstruct.files, finalstruct.sorttype.clone());
        block_on(finalstruct.regen_uifiles());
        finalstruct
    }
}