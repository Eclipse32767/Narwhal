use crate::Narwhal;
use crate::sort_file_by_type;
use std::str::FromStr;
use std::{env, fs};
use cosmic_time::Timeline;
use iced::widget::text_input;
use toml;
use crate::CacheFile;
use std::collections::HashMap;
use crate::get_cache_home;
use crate::get_config_home;
use crate::Config;
use crate::decode_sort;
use iced::Color;

use iced::futures::executor::block_on;
use iced_style::theme;
use oceania_style::{ButtonStyle, get_set_theme, ListStyle, make_custom_theme, MenuStyle, ThemeCustom, ThemeSet};

impl Default for Narwhal {
    fn default() -> Self {
        let current_dir = match env::current_dir() {//collect the user's working directory
            Ok(x) => x,
            Err(x) => panic!("{}", x)
        };
        let cache_home = format!("{}/NarwhalFM", get_cache_home());//collect the cache from the cache file
        let cache_text = fs::read_to_string(cache_home);
        let cache_struct: CacheFile = match cache_text {
            Ok(x) => toml::from_str(&x).unwrap(),
            Err(..) => CacheFile { contents: HashMap::new() }
        };
        let config_home = format!("{}/Oceania/NarwhalFM.toml", get_config_home());//collect the config options from the config file
        let config_text = fs::read_to_string(config_home);
        let config_struct: Config = match config_text {
            Ok(x) => toml::from_str(&x).unwrap(),
            Err(..) => Config { sort_mode: "Folder".to_string(), show_hidden: false, bookmarks: vec![], icn_theme: String::from_str("Adwaita").unwrap(), icn_size: 32 }
        };
        let mut final_struct = Narwhal {//build a struct with only config options injected
            files: vec![], 
            current_path: current_dir,
            sort_type: decode_sort(config_struct.sort_mode),
            desired_cols: 5, 
            show_hidden: config_struct.show_hidden, 
            desired_rows: 5, 
            last_clicked_file: None, 
            ui_files: vec![],
            icon_cache: cache_struct.contents.clone(), 
            bookmarked_dirs: config_struct.bookmarks.clone(), 
            deletion_confirmation: false, 
            mv_target: None, 
            cp_target: None,
            theme: get_set_theme(),
            type_mode: None,
            rename_id: text_input::Id::unique(),
            show_keybinds: false,
            anims: Timeline::new(),
            icn_theme: config_struct.icn_theme.clone(),
            icn_size: config_struct.icn_size,
            show_file_options: true,
            themes: ThemeSet {
            light: ThemeCustom {
                application: theme::Palette {
                    background: Color::from_rgb8(0xE0, 0xF5, 0xFF),
                    text: Color::from_rgb8(0x00, 0x19, 0x36),
                    primary: Color::from_rgb8(0x00, 0x77, 0xFF),
                    success: Color::from_rgb8(0x00, 0xCB, 0x40),
                    danger: Color::from_rgb8(0xFF, 0x4C, 0x00),
                },
                sidebar: ButtonStyle {
                    border_radius: 2.0,
                    txt_color: Color::from_rgb8(0x00, 0x19, 0x36),
                    bg_color: Color::from_rgb8(0xD2, 0xF0, 0xFF),
                    border_color: Color::from_rgb8(0, 0, 0),
                    border_width: 0.0,
                    shadow_offset: iced::Vector { x: 0.0, y: 0.0 }
                },
                secondary: ButtonStyle {
                    border_radius: 2.0,
                    txt_color: Color::from_rgb8(0x00, 0x20, 0x46),
                    bg_color: Color::from_rgb8(0xC6, 0xEC, 0xFF),
                    border_color: Color::from_rgb8(0, 0, 0),
                    border_width: 0.0,
                    shadow_offset: iced::Vector { x: 0.0, y: 0.0 }
                },
                list: ListStyle {
                    txt_color: Color::from_rgb8(0x00, 0x19, 0x36),
                    bg_color: Color::from_rgb8(0xE0, 0xF5, 0xFF),
                    handle_color: Color::from_rgb8(0x00, 0x19, 0x36),
                    border_radius: 5.0,
                    border_width: 2.0,
                    border_color: Color::from_rgb8(0x00, 0x19, 0x36),
                    menu: MenuStyle {
                        txt_color: Color::from_rgb8(0x00, 0x19, 0x36),
                        bg_color: Color::from_rgb8(0xE0, 0xF5, 0xFF),
                        border_radius: 5.0,
                        border_width: 2.0,
                        border_color: Color::from_rgb8(0x00, 0x19, 0x36),
                        sel_txt_color: Color::from_rgb8(0x00, 0x19, 0x36),
                        sel_bg_color: Color::from_rgb8(0x00, 0xF1, 0xD6),
                    }
                }
            },
            dark: ThemeCustom { // TODO: set dark theme properly
                application: theme::Palette {
                    background: Color::from_rgb8(0x00, 0x19, 0x36),
                    text: Color::from_rgb8(0xE0, 0xF5, 0xFF),
                    primary: Color::from_rgb8(0x00, 0xAB, 0xE1),
                    success: Color::from_rgb8(0x00, 0xA9, 0x35),
                    danger: Color::from_rgb8(0xC5, 0x3A, 0x00),
                },
                sidebar: ButtonStyle {
                    border_radius: 2.0,
                    txt_color: Color::from_rgb8(0xE0, 0xF5, 0xFF),
                    bg_color: Color::from_rgb8(0x00, 0x20, 0x46),
                    border_color: Color::from_rgb8(0, 0, 0),
                    border_width: 0.0,
                    shadow_offset: iced::Vector { x: 0.0, y: 0.0 }
                },
                secondary: ButtonStyle {
                    border_radius: 2.0,
                    txt_color: Color::from_rgb8(0xE0, 0xF5, 0xFF),
                    bg_color: Color::from_rgb8(0x00, 0x29, 0x58),
                    border_color: Color::from_rgb8(0, 0, 0),
                    border_width: 0.0,
                    shadow_offset: iced::Vector { x: 0.0, y: 0.0 }
                },
                list: ListStyle {
                    txt_color: Color::from_rgb8(0xE0, 0xF5, 0xFF),
                    bg_color: Color::from_rgb8(0x00, 0x29, 0x58),
                    handle_color: Color::from_rgb8(0xE0, 0xF5, 0xFF),
                    border_radius: 5.0,
                    border_width: 2.0,
                    border_color: Color::from_rgb8(0xE0, 0xF5, 0xFF),
                    menu: MenuStyle {
                        txt_color: Color::from_rgb8(0xE0, 0xF5, 0xFF),
                        bg_color: Color::from_rgb8(0xE0, 0xF5, 0xFF),
                        border_radius: 5.0,
                        border_width: 2.0,
                        border_color: Color::from_rgb8(0xE0, 0xF5, 0xFF),
                        sel_txt_color: Color::from_rgb8(0xE0, 0xF5, 0xFF),
                        sel_bg_color: Color::from_rgb8(0x00, 0xCD, 0xB6),
                    }
                }
            },
            custom: make_custom_theme()
        },
        };
        final_struct.regen_files();//generate file list
        sort_file_by_type(&mut final_struct.files, final_struct.sort_type.clone());//sort file list
        block_on(final_struct.regen_ui_files());//regenerate ui files
        final_struct
    }
}