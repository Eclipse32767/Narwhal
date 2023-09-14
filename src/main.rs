#![deny(unsafe_code)]
use iced::futures::executor::block_on;
use iced::futures::future::join_all;
use iced::{Application, Result, Settings, executor, Length, Event};
use iced::widget::{Button, Text, Row, Column, Container, Rule, text_input, TextInput, Space};
use iced::theme;
use iced_style::Theme;
use std::collections::HashMap;
use std::fs::{DirEntry, Metadata};
use std::{fs, vec};
use std::path::PathBuf;
use std::process::Command;
use toml;
use gettextrs::*;
use gettextrs::gettext as tr;
use lib_style::{ThemeSet, CustomTheme, ButtonStyle, ThemeFile, mk_app_theme, col_from_string};
mod lib_style;
use icon_helpers::{get_file_icon, get_file_mimetype};
mod icon_helpers;
use config_helpers::*;
mod config_helpers;
use ui_helpers::*;
mod ui_helpers;
mod kb_parser;
mod default_state;
use cosmic_time::{
    self, anim, chain, id, Duration, Instant, once_cell::sync::Lazy, Timeline,
};

fn main() -> Result {
    let _ = textdomain("NarwhalFM");
    let _ = bind_textdomain_codeset("NarwhalFM", "UTF-8");

    let mut settings = Settings::default();
    settings.exit_on_close_request =  false;
    Narwhal::run(settings)
}

const EST_LENGTH: u32 = 84;
const EST_HEIGHT: u32 = 104;
const FONT_SIZE: u16 = 12;
const SPECIAL_FONT_SIZE: u16 = 14;
const SPACING: u16 = 10;
const MAX_LENGTH: usize = 10;
const SIDEBAR_WIDTH: u16 = 120;
const IMAGE_SCALE: u16 = 64;
const RULE_WIDTH: u16 = 1;
const TOP_HEIGHT: u16 = 30;

static RENAME_BTN: Lazy<id::Button> = Lazy::new(id::Button::unique);
static MENU_BTN: Lazy<id::Button> = Lazy::new(id::Button::unique);
static MENU_BTN_LONG: Lazy<id::Button> = Lazy::new(id::Button::unique);

struct Narwhal {//contains all application state
    files: Vec<DirEntry>,
    current_path: PathBuf,
    sort_type: SortType,
    desired_cols: u32,
    desired_rows: u32,
    show_hidden: bool,
    last_clicked_file: Option<usize>,
    ui_files: Vec<UIFile>,
    icon_cache: HashMap<String, String>,
    bookmarked_dirs: Vec<BookmarkDir>,
    deletion_confirmation: bool,
    mv_target: Option<String>,
    cp_target: Option<String>,
    themes: ThemeSet,
    theme: ThemeType,
    type_mode: Option<String>,
    rename_id: text_input::Id,
    show_keybinds: bool,
    anims: Timeline,
    icn_theme: String,
    icn_size: u16,
    show_file_options: bool,
}

#[derive(Debug, Clone)]
pub enum Message {//enum representing button events
    FileClicked(usize),
    GoBack(usize),
    SortChanged,
    HiddenChanged,
    KeyboardUpdate(iced::keyboard::Event),
    WindowUpdate(iced::window::Event),
    BookmarkCurrent,
    BookmarkClicked(usize),
    DeleteClicked,
    MvClicked,
    CpClicked,
    MkFile,
    MkDir,
    RenameToggle,
    RenameUpdate(String),
    Tick(Instant),
    ToggleMenu,
    NoOp,
}
fn get_file_type(metadata: Metadata) -> FileType {//collects the filetype from metadata
    if metadata.is_dir() {
        FileType::Folder
    } else if metadata.is_file() {
        FileType::File
    } else if metadata.is_symlink() {
        FileType::Link
    } else {
        FileType::File
    }
}
fn clip_file_name(name: String) -> String {//shorten the file name
    let use_name: Vec<char> = name.clone().chars().collect();
    let mut new_vec: Vec<char> = vec![];
    let len = MAX_LENGTH;
    if name.chars().count() > len {
        for i in 0..len {
            new_vec.push(use_name[i]);
        }
        let new_str = new_vec.into_iter().collect::<String>();
        format!("{new_str}...")
    } else {
        name
    }  
}
fn folder_cmp(a: &DirEntry, b: &DirEntry, folders_first: bool) -> std::cmp::Ordering {//compare folders, returning an ordering
    let a_metadata = a.metadata().unwrap();
    let b_metadata = b.metadata().unwrap();
    let a_type = get_file_type(a_metadata);
    let b_type = get_file_type(b_metadata);
    if a_type == b_type {
        a.file_name().to_string_lossy().to_string().partial_cmp(&b.file_name().to_string_lossy().to_string()).unwrap()
    } else {
        if folders_first {
            if a_type == FileType::Folder {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            }
        } else {
            if a_type == FileType::Folder {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Less
            }
        }
    }
}

impl Narwhal {
    async fn regen_ui_files(&mut self) {
        let mut items_flushed = 0;
        let max_iter = self.desired_cols * self.desired_rows;
        let mut futures = Vec::with_capacity(max_iter as usize);
        let mut names = Vec::with_capacity(max_iter as usize);
        let mut selected_vals = Vec::with_capacity(max_iter as usize);
        let mut original_indexes = Vec::with_capacity(max_iter as usize);
        let mut all_changes = vec![];
        let exec = iced::executor::Default::new().unwrap();
        self.ui_files = Vec::with_capacity(max_iter as usize);
        for i in 0..self.files.len() {
            if items_flushed >= max_iter {
                break;
            }
            let name = self.files[i].file_name().to_string_lossy().to_string();
            let chars: Vec<char> = name.chars().collect();
            if !self.show_hidden && chars[0] == '.' {//filter out hidden files if desired
            } else {
                let path = self.files[i].path().to_string_lossy().to_string();
                let selected = match self.last_clicked_file {
                    Some(value) => value == i,
                    None => false
                };
                futures.push(exec.spawn(get_file_icon(self.icon_cache.clone(), path.clone(), self.icn_theme.clone(), self.icn_size)));//spawn all file icon fetching futures
                names.push(name);
                selected_vals.push(selected);
                original_indexes.push(i);
                items_flushed = items_flushed + 1;
            }
        }
        for i in 0..futures.len() {//resolve futures
            let output = futures.remove(0).await.unwrap();//wait for it to finish then collect result
            let icon = output.1;
            match output.0 {
                Some(cache_changes) => {//push any cache changes onto a vec of all needed changes
                    all_changes.push(cache_changes)
                }
                None => {

                }
            }
            let ui_file = UIFile { name: names[i].clone(), original_index: original_indexes[i], selected: selected_vals[i], icon };//construct the UIFile and push it onto the vec
            self.ui_files.push(ui_file);
        }
        for change in all_changes {//for every change, push it onto the cache
            self.icon_cache.extend(change.into_iter());
        }
        self.type_mode = None;
    }
    fn regen_files(&mut self) {//rebuild file list
        self.files = vec![];
        let read_output = match fs::read_dir(self.current_path.clone()) {
            Ok(x) => x,
            Err(x) => panic!("{}", x),
        };
        for path in read_output {
            self.files.push(path.unwrap())
        }
    }
    fn interact_selected_entry(&mut self, index: usize) {//do sanity checks and then interact with the currently hovered entry if all checks pass
        match self.last_clicked_file {
            Some(x) => {
                if x == index {
                    let metadata = self.files[x].metadata().unwrap();
                    if metadata.is_symlink() {
                        let mime = get_file_mimetype(self.files[x].path().to_string_lossy().to_string()).replace("/", "-");
                        let path = fs::read_link(self.files[x].path().to_string_lossy().to_string()).unwrap();
                        if mime == "inode-directory" {
                            let path_txt = path.to_string_lossy().to_string();
                            let path_chars: Vec<char> = path_txt.chars().collect();
                            let clean_path = if path_chars[0] == '/' {
                                path_txt
                            } else {
                                format!("/{}", path_txt)
                            };
                            self.current_path = PathBuf::from(clean_path);
                            println!("{}", self.current_path.to_string_lossy());
                            self.regen_files();
                            sort_file_by_type(&mut self.files, self.sort_type.clone());
                        } else {
                            Command::new("xdg-open").arg(path.to_string_lossy().to_string()).spawn().expect("oops");
                        }
                    } else {
                        let mime = get_file_mimetype(self.files[x].path().to_string_lossy().to_string()).replace("/", "-");
                        if mime == "inode-directory" {
                            let filename = self.files[x].path().display().to_string();
                            self.current_path.push(filename);
                            println!("{}", self.current_path.to_string_lossy());
                            self.regen_files();
                            sort_file_by_type(&mut self.files, self.sort_type.clone());
                        } else {
                            let filename = self.files[x].path().display().to_string();
                            Command::new("xdg-open").arg(filename).spawn().expect("oops");
                        }
                    }
                    self.last_clicked_file = None;
                } else {
                    self.last_clicked_file = Some(index);
                }
                block_on(self.regen_ui_files());
            }
            None => {
                self.last_clicked_file = Some(index);
                block_on(self.regen_ui_files());
            }
        }
    }
    fn go_back_directory(&mut self) {//pop an entry off of the current path, regenerate the file list and UI Files
        self.current_path.pop();
        self.regen_files();
        sort_file_by_type(&mut self.files, self.sort_type.clone());
        self.last_clicked_file = None;
        block_on(self.regen_ui_files());
    }
    fn change_sort(&mut self, reverse: bool) {//cycle through sort modes
        if reverse {
            self.sort_type = match self.sort_type {
                SortType::Alphabetical => SortType::Files,
                SortType::Reverse => SortType::Alphabetical,
                SortType::Folders => SortType::Reverse,
                SortType::Files => SortType::Folders,
            };
        } else {
            self.sort_type = match self.sort_type {
                SortType::Alphabetical => SortType::Reverse,
                SortType::Reverse => SortType::Folders,
                SortType::Folders => SortType::Files,
                SortType::Files => SortType::Alphabetical,
            }; 
        }
        sort_file_by_type(&mut self.files, self.sort_type.clone());
        self.last_clicked_file = None;
        block_on(self.regen_ui_files());
    }
    fn rm_file(&mut self, index: usize) {//remove a file, this function contains less sanity checks and should be used carefully
        let path = self.files[index].path().to_string_lossy().to_string();
        let is_directory = match self.files[index].metadata() {
            Ok(x) => x.is_dir(),
            Err(..) => false,
        };
        if is_directory {
            match fs::remove_dir_all(path) {
                Ok(..) => println!("successfully removed"),
                Err(e) => println!("{e}")
            }
        } else {
            match fs::remove_file(path) {
                Ok(..) => println!("successfully removed"),
                Err(e) => println!("{e}")
            }
        }
        self.regen_files();
        sort_file_by_type(&mut self.files, self.sort_type.clone());
        self.last_clicked_file = None;
        block_on(self.regen_ui_files());
    }
    fn mv_file(&mut self) {//move a file to another location, no sanity checks
        let target = self.mv_target.clone().unwrap();
        let path = self.current_path.to_string_lossy().to_string();
        Command::new("mv").arg(target).arg(path).output().unwrap();
        self.mv_target = None;
        self.regen_files();
        sort_file_by_type(&mut self.files, self.sort_type.clone());
        self.last_clicked_file = None;
        block_on(self.regen_ui_files());
    }
    fn cp_file(&mut self) {//copy a file to another location, no sanity checks
        let target = self.cp_target.clone().unwrap();
        let path = self.current_path.to_string_lossy().to_string();
        Command::new("cp").arg(target).arg(path).output().unwrap();
        self.cp_target = None;
        self.regen_files();
        sort_file_by_type(&mut self.files, self.sort_type.clone());
        self.last_clicked_file = None;
        block_on(self.regen_ui_files());
    }
    fn touch(&mut self) {
        let path = format!("{}/NewFile", self.current_path.to_string_lossy().to_string());
        Command::new("touch").arg(path).output().unwrap();
        self.regen_files();
        sort_file_by_type(&mut self.files, self.sort_type.clone());
        self.last_clicked_file = None;
        block_on(self.regen_ui_files());
    }
    fn mkdir(&mut self) {
        let path = format!("{}/NewFolder", self.current_path.to_string_lossy().to_string());
        Command::new("mkdir").arg(path).output().unwrap();
        self.regen_files();
        sort_file_by_type(&mut self.files, self.sort_type.clone());
        self.last_clicked_file = None;
        block_on(self.regen_ui_files());
    }
    fn rename(&mut self) {
        let src_path = self.files[self.last_clicked_file.unwrap()].path().to_string_lossy().to_string();
        let dest_path = format!("{}/{}", self.current_path.to_string_lossy().to_string(), self.type_mode.clone().unwrap());
        Command::new("mv").arg(src_path).arg(dest_path).output().unwrap();
        self.regen_files();
        sort_file_by_type(&mut self.files, self.sort_type.clone());
        self.last_clicked_file = None;
        block_on(self.regen_ui_files());
    }
}
fn sort_file_by_type(input: &mut Vec<DirEntry>, sort_type: SortType) {//sort files based on the chosen SortType
    match sort_type {
        SortType::Alphabetical => {
            input.sort_by(|a, b| a.file_name().to_string_lossy().to_string().partial_cmp( &b.file_name().to_string_lossy().to_string()).unwrap())
        }
        SortType::Reverse => {
            input.sort_by(|a, b| b.file_name().to_string_lossy().to_string().partial_cmp( &a.file_name().to_string_lossy().to_string()).unwrap())
        }
        SortType::Files => {
            input.sort_by(|a, b| folder_cmp(a, b, false))
        }
        SortType::Folders => {
            input.sort_by(|a, b| folder_cmp(a, b, true))
        }
    }
}

impl Application for Narwhal {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();
    fn new(_flags: ()) -> (Self, iced::Command<Self::Message>) {//initialize program
        let mut narwhal = Self::default();
        use cosmic_time::button;
        let un_mitosis = chain![RENAME_BTN,
            button(Duration::ZERO).width(Length::Fixed(0.0)).height(Length::Fixed(TOP_HEIGHT as f32)),
            button(Duration::from_millis(500)).width(Length::Fixed(75.0)).height(Length::Fixed(TOP_HEIGHT as f32)),
            button(Duration::from_millis(500)).width(Length::Fixed(1000.0)).height(Length::Fixed(TOP_HEIGHT as f32)),
        ];
        narwhal.anims.set_chain(un_mitosis).start();
        (
            narwhal,
            iced::Command::none()
        )
    }
    fn title(&self) -> String {//Window title
        String::from("Narwhal File Manager")
    }
    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {//decide what to do based on message
        let mut temp_files: Vec<String> = vec![];
        for file in &self.files {
            let temp = file.file_name().to_string_lossy().to_string();
            temp_files.push(temp);
        };
        match message {
            Message::FileClicked(x) => {//a file was clicked, interact it
                self.interact_selected_entry(x);
                iced::Command::none()
            },
            Message::GoBack(x) => {//go back a directory
                for _i in 0..x {
                    self.go_back_directory();
                }
                iced::Command::none()
            },
            Message::SortChanged => {//change sort mode
                self.change_sort(false);
                iced::Command::none()
            }
            Message::HiddenChanged => {//change hidden flag
                self.show_hidden = !self.show_hidden;
                block_on(self.regen_ui_files());
                iced::Command::none()
            }
            Message::BookmarkCurrent => {//bookmark or un-bookmark current dir
                let dir = self.current_path.to_string_lossy().to_string();
                let paths: Vec<&str> = dir.split('/').into_iter().collect();
                let name = paths[paths.len()-1].to_string();
                let bookmark = BookmarkDir { name, path: dir };
                let mut bookmark_already_exists = None;
                for i in 0..self.bookmarked_dirs.len() {
                    if bookmark.path == self.bookmarked_dirs[i].path {
                        bookmark_already_exists = Some(i);
                    }
                }
                match bookmark_already_exists {
                    Some(value) => {
                        self.bookmarked_dirs.remove(value);
                    }
                    None => {
                        self.bookmarked_dirs.push(bookmark);
                    }
                }
                iced::Command::none()
            }
            Message::BookmarkClicked(index) => {//go to the bookmark's chosen dir
                self.current_path = PathBuf::from(self.bookmarked_dirs[index].path.clone());
                self.regen_files();
                sort_file_by_type(&mut self.files, self.sort_type.clone());
                self.last_clicked_file = None;
                block_on(self.regen_ui_files());
                iced::Command::none()
            }
            Message::KeyboardUpdate(kb_event) => {//send to keyboard parser
                self.kb_parse(kb_event)
            }
            Message::WindowUpdate(win_event) => {
                match win_event {
                    iced::window::Event::Moved { x: _, y: _ } => {iced::Command::none()},
                    iced::window::Event::Resized { width, height } => {//calculate appropriate amount of rows and columns
                        let old_cols = self.desired_cols;
                        let old_rows = self.desired_rows;
                        if width > SIDEBAR_WIDTH as u32 {
                            let adjusted_width = width - SIDEBAR_WIDTH as u32;
                            self.desired_cols = adjusted_width / EST_LENGTH;
                        }
                        if self.desired_cols == 0 {
                            self.desired_cols = 1;
                        }
                        if height > EST_HEIGHT {
                            let adjusted_height = height;
                            self.desired_rows = adjusted_height / EST_HEIGHT;
                        }
                        if old_cols == self.desired_cols && old_rows == self.desired_rows {

                        } else {
                            block_on(self.regen_ui_files());
                        }
                        iced::Command::none()
                    },
                    iced::window::Event::RedrawRequested(_) => {iced::Command::none()},
                    iced::window::Event::CloseRequested => {//write cache and config, then close
                        let yes = CacheFile { contents: self.icon_cache.clone() };
                        let cached_contents = toml::to_string(&yes).unwrap();
                        let cache_home = format!("{}/NarwhalFM", get_cache_home());
                        fs::write(cache_home, cached_contents).unwrap();
                        let config_file = Config { sort_mode: encode_sort(self.sort_type.clone()), show_hidden: self.show_hidden, bookmarks: self.bookmarked_dirs.clone(), icn_theme: self.icn_theme.clone(), icn_size: self.icn_size };
                        let config_text = toml::to_string(&config_file).unwrap();
                        let config_home = format!("{}/Oceania/NarwhalFM.toml", get_config_home());
                        fs::write(config_home, config_text).unwrap();
                        iced::window::close()
                    },
                    iced::window::Event::Focused => {iced::Command::none()},
                    iced::window::Event::Unfocused => {iced::Command::none()},
                    iced::window::Event::FileHovered(_) => {iced::Command::none()},
                    iced::window::Event::FileDropped(_) => {iced::Command::none()},
                    iced::window::Event::FilesHoveredLeft => {iced::Command::none()},
                }
            }
            Message::DeleteClicked => {//do sanity checks then rm file
                match self.last_clicked_file {
                    Some(x) => {
                        if self.deletion_confirmation {
                            self.rm_file(x);
                        }
                        self.deletion_confirmation = !self.deletion_confirmation;
                    }
                    None => {
                        self.deletion_confirmation = false;
                    }
                }
                iced::Command::none()
            }
            Message::MvClicked => {//do sanity checks then mv file
                match self.mv_target {
                    Some(..) => {
                        self.mv_file();
                    }
                    None => {
                        match self.last_clicked_file {
                            Some(x) => {
                                let path = self.files[x].path().to_string_lossy().to_string();
                                self.mv_target = Some(path);
                            }
                            None => {
                                self.mv_target = None;
                            }
                        }
                    }
                }
                iced::Command::none()
            }
            Message::CpClicked => {//do sanity checks then cp file
                match self.cp_target {
                    Some(..) => {
                        self.cp_file();
                    }
                    None => {
                        match self.last_clicked_file {
                            Some(x) => {
                                let path = self.files[x].path().to_string_lossy().to_string();
                                self.cp_target = Some(path);
                            }
                            None => {
                                self.cp_target = None;
                            }
                        }
                    }
                }
                iced::Command::none()
            }
            Message::MkDir => {
                self.mkdir();
                iced::Command::none()
            }
            Message::MkFile => {
                self.touch();
                iced::Command::none()
            }
            Message::RenameToggle => {
                match &self.type_mode {
                    Some(val) => {
                        match self.last_clicked_file {
                            Some(..) => {
                                if val.len() >= 1 {
                                    self.rename()
                                }
                                self.type_mode = None
                            }
                            None => {
                                self.type_mode = None
                            }
                        }
                        use cosmic_time::button;
                        let un_mitosis = chain![RENAME_BTN,
                            button(Duration::ZERO).width(Length::Fixed(0.0)).height(Length::Fixed(TOP_HEIGHT as f32)),
                            button(Duration::from_millis(500)).width(Length::Fixed(0.0)).height(Length::Fixed(TOP_HEIGHT as f32)),
                            button(Duration::from_millis(500)).width(Length::Fixed(1000.0)).height(Length::Fixed(TOP_HEIGHT as f32)),
                        ];
                        self.anims.set_chain(un_mitosis).start();
                        iced::Command::none()
                    },
                    None => {
                        use cosmic_time::button;
                        let mitosis = chain![RENAME_BTN,
                            button(Duration::ZERO).width(Length::Fixed(1000.0)).height(Length::Fixed(TOP_HEIGHT as f32)),
                            button(Duration::from_millis(500)).width(Length::Fixed(1000.0)).height(Length::Fixed(TOP_HEIGHT as f32)),
                            button(Duration::from_millis(500)).width(Length::Fixed(0.0)).height(Length::Fixed(TOP_HEIGHT as f32)),
                        ];
                        self.anims.set_chain(mitosis).start();
                        self.type_mode = Some(String::default());
                        text_input::focus(self.rename_id.clone())
                    }
                }
            }
            Message::RenameUpdate(x) => {
                self.type_mode = Some(x);
                iced::Command::none()
            }
            Message::Tick(now) => {
                self.anims.now(now);
                iced::Command::none()
            }
            Message::NoOp => {
                iced::Command::none()
            }
            Message::ToggleMenu => {
                use cosmic_time::button;
                self.show_file_options = !self.show_file_options;
                let grow_options = chain![MENU_BTN,
                    button(Duration::ZERO).width(Length::Fixed(0.0)).height(Length::Fixed(TOP_HEIGHT as f32)),
                    button(Duration::from_millis(500)).width(Length::Fixed(50.0)).height(Length::Fixed(TOP_HEIGHT as f32))
                ];
                let shrink_options = chain![MENU_BTN,
                    button(Duration::ZERO).width(Length::Fixed(50.0)).height(Length::Fixed(TOP_HEIGHT as f32)),
                    button(Duration::from_millis(500)).width(Length::Fixed(0.0)).height(Length::Fixed(TOP_HEIGHT as f32))
                ];
                let grow_options_long = chain![MENU_BTN_LONG,
                    button(Duration::ZERO).width(Length::Fixed(0.0)).height(Length::Fixed(TOP_HEIGHT as f32)),
                    button(Duration::from_millis(500)).width(Length::Fixed(80.0)).height(Length::Fixed(TOP_HEIGHT as f32))
                ];
                let shrink_options_long = chain![MENU_BTN_LONG,
                    button(Duration::ZERO).width(Length::Fixed(80.0)).height(Length::Fixed(TOP_HEIGHT as f32)),
                    button(Duration::from_millis(500)).width(Length::Fixed(0.0)).height(Length::Fixed(TOP_HEIGHT as f32))
                ];
                match self.show_file_options {
                    true => self.anims.set_chain(grow_options).set_chain(grow_options_long),
                    false => self.anims.set_chain(shrink_options).set_chain(shrink_options_long)
                };
                self.anims.start();
                iced::Command::none()
            }
        }
    }
    fn view(&self) -> iced::Element<'_, Self::Message, iced::Renderer<Self::Theme>> {//render code!
        let current_theme = match self.theme {//clone selected theme into current_theme
            ThemeType::Light => self.themes.light.clone(),
            ThemeType::Dark => self.themes.dark.clone(),
            ThemeType::Custom => self.themes.custom.clone(),
        }; 
        let translated = match self.show_keybinds {
            true => [tr("<Backspace>"), tr("<S>"), tr("<Shift+Minus>"), tr("<M>"), tr("<M>"), tr("<C>"), tr("<C>"), tr("<H>"),  tr("<Shift+B>"), tr("<N>"), tr("<Shift+N>"), tr("<R>")],
            false => [tr("Back"), tr("Sort"), tr("Delete"), tr("Move Here"), tr("Move"), tr("Paste"), tr("Copy"), tr("Hidden"), tr("Bookmark"), tr("Make File"), tr("Make Folder"), tr("Rename")]
        };
        // construct top bar
        let option_btn = Button::new("...").height(TOP_HEIGHT).on_press(Message::ToggleMenu).style(current_theme.secondary.mk_theme());
        let back_btn = string_button(translated[0].clone(), SPECIAL_FONT_SIZE).on_press(Message::GoBack(1)).height(TOP_HEIGHT).style(current_theme.secondary.mk_theme());
        let sort_btn = anim!(MENU_BTN, &self.anims, Text::new(translated[1].clone()).size(SPECIAL_FONT_SIZE)).on_press(Message::SortChanged).height(TOP_HEIGHT).style(current_theme.secondary.mk_theme());
        let delete_btn = if self.deletion_confirmation {
            anim!(MENU_BTN_LONG, &self.anims, Text::new(translated[2].clone()).size(SPECIAL_FONT_SIZE)).on_press(Message::DeleteClicked).height(TOP_HEIGHT).style(theme::Button::Destructive)
        } else {
            anim!(MENU_BTN_LONG, &self.anims, Text::new(translated[2].clone()).size(SPECIAL_FONT_SIZE)).on_press(Message::DeleteClicked).height(TOP_HEIGHT).style(current_theme.secondary.mk_theme())
        };
        let mv_btn = match self.mv_target {
            Some(..) => anim!(MENU_BTN, &self.anims, Text::new(translated[3].clone()).size(SPECIAL_FONT_SIZE)).on_press(Message::MvClicked),
            None => anim!(MENU_BTN, &self.anims, Text::new(translated[4].clone()).size(SPECIAL_FONT_SIZE)).on_press(Message::MvClicked).height(TOP_HEIGHT).style(current_theme.secondary.mk_theme())
        };
        let cp_btn = match self.cp_target {
            Some(..) => anim!(MENU_BTN, &self.anims, Text::new(translated[5].clone()).size(SPECIAL_FONT_SIZE)).on_press(Message::CpClicked),
            None => anim!(MENU_BTN, &self.anims, Text::new(translated[6].clone()).size(SPECIAL_FONT_SIZE)).on_press(Message::CpClicked).height(TOP_HEIGHT).style(current_theme.secondary.mk_theme())
        };
        let hidden_btn = anim!(MENU_BTN_LONG, &self.anims, Text::new(translated[7].clone()).size(SPECIAL_FONT_SIZE)).height(TOP_HEIGHT).on_press(Message::HiddenChanged).style(current_theme.secondary.mk_theme());
        let bookmark_btn = anim!(MENU_BTN_LONG, &self.anims, Text::new(translated[8].clone()).size(SPECIAL_FONT_SIZE)).height(TOP_HEIGHT).on_press(Message::BookmarkCurrent).style(current_theme.secondary.mk_theme());
        let touch_btn = anim!(MENU_BTN, &self.anims, Text::new(translated[9].clone()).size(SPECIAL_FONT_SIZE)).width(SIDEBAR_WIDTH).on_press(Message::MkFile).style(current_theme.sidebar.mk_theme());
        let mkdir_btn = anim!(MENU_BTN, &self.anims, Text::new(translated[10].clone()).size(SPECIAL_FONT_SIZE)).width(SIDEBAR_WIDTH).on_press(Message::MkDir).style(current_theme.sidebar.mk_theme());
        //let function_cap = Button::new("").width(5000).height(TOP_HEIGHT).style(current_theme.secondary.mk_theme());
        let rename_btn = string_button(translated[11].clone(), SPECIAL_FONT_SIZE).height(Length::Fixed(TOP_HEIGHT as f32)).on_press(Message::RenameToggle).style(current_theme.secondary.mk_theme());
        let function_cap = anim!(RENAME_BTN, &self.anims, "").height(TOP_HEIGHT).on_press(Message::NoOp).style(current_theme.secondary.mk_theme());
        let mut function_buttons = Row::new().push(back_btn).push(option_btn).push(sort_btn).push(hidden_btn).push(bookmark_btn).push(delete_btn).push(mv_btn).push(cp_btn).push(rename_btn).push(function_cap);
        let txt = match &self.type_mode {
            Some(x) => x.clone(),
            None => String::from("")
        };
        let rename_input = TextInput::new(tr("Placeholder").as_str(), txt.as_str()).on_input(Message::RenameUpdate).size(SPECIAL_FONT_SIZE).id(self.rename_id.clone());
        function_buttons = function_buttons.push(rename_input);
        //construct bookmark column
        let mut bookmark_buttons = Column::new().push(mkdir_btn).push(touch_btn);
        for i in 0..self.bookmarked_dirs.len() {
            let btn_text = match self.show_keybinds {
                false => Text::new(self.bookmarked_dirs[i].name.clone()).size(SPECIAL_FONT_SIZE),
                true => Text::new(format!("<{}>", i+1)).size(SPECIAL_FONT_SIZE),
            };
            let btn = Button::new(btn_text).on_press(Message::BookmarkClicked(i)).width(SIDEBAR_WIDTH).style(current_theme.sidebar.mk_theme());
            bookmark_buttons = bookmark_buttons.push(btn);
        }
        let bookmark_cap = Button::new("").height(5000).width(SIDEBAR_WIDTH).style(current_theme.sidebar.mk_theme()).on_press(Message::NoOp);
        bookmark_buttons = bookmark_buttons.push(bookmark_cap);
        //construct file view
        let mut file_listing = Column::new();
        let mut temp_row = Row::new();
        let mut file_btn_futures = vec![];
        for i in 0..self.ui_files.len() {
            file_btn_futures.push(self.ui_files[i].render());
        }
        let mut test = block_on(join_all(file_btn_futures));
        for i in 0..test.len() {
            let full = test.remove(0);
            if i % self.desired_cols as usize == 0 {
                file_listing = file_listing.push(temp_row);
                temp_row = Row::new().spacing(SPACING);
            }
            temp_row = temp_row.push(full);
        }
        file_listing = file_listing.push(temp_row);
        let mut path_bar = Row::new();
        let chars: Vec<char> = self.current_path.to_string_lossy().to_string().chars().collect();
        let mut path_entries = vec![];
        let path_cap = Button::new("").height(TOP_HEIGHT).width(10000).style(current_theme.sidebar.mk_theme()).on_press(Message::NoOp);
        let mut entries = 0;
        for character in chars {
            if character == '/' {
                path_entries.push(String::default());
                entries = path_entries.len() - 1;
            } else {
                path_entries[entries] = format!("{}{}", path_entries[entries], character);
            }
        }
        let mut iterations = 0;
        for entry in path_entries {
            path_bar = path_bar.push(Button::new(Text::new("/")).on_press(Message::NoOp).style(current_theme.sidebar.mk_theme()).height(TOP_HEIGHT));
            path_bar = path_bar.push(Button::new(Text::new(entry)).on_press(Message::GoBack(entries - iterations)).style(current_theme.sidebar.mk_theme()).height(TOP_HEIGHT));
            iterations = iterations + 1;
        }
        path_bar = path_bar.push(path_cap);
        //return render commands
        let rule_h = Rule::horizontal(RULE_WIDTH);
        let rule_h2 = Rule::horizontal(RULE_WIDTH);
        let fill_space = Space::new(10, Length::Fill);
        let rule_v = Rule::vertical(RULE_WIDTH);
        let col_test = Column::new().push(function_buttons).push(rule_h).push(file_listing).push(fill_space).push(rule_h2).push(path_bar);
        let row_test = Row::new().push(bookmark_buttons).push(rule_v).push(col_test);
        Container::new(row_test).width(Length::Fill).height(Length::Fill).into()
    }
    fn theme(&self) -> Self::Theme {//send in the selected application theme
        match self.theme {
            ThemeType::Light => mk_app_theme(self.themes.light.application.clone()),
            ThemeType::Dark => mk_app_theme(self.themes.dark.application.clone()),
            ThemeType::Custom => mk_app_theme(self.themes.custom.application.clone()),
        }
    }
    fn subscription(&self) -> iced::Subscription<Message> {//listen in on keyboard and window events
        iced::Subscription::batch(vec![
            self.anims.as_subscription::<Event>().map(Message::Tick),
            iced::subscription::events_with(
                |event, _| match event {
                    Event::Keyboard(evt) => Some(Message::KeyboardUpdate(evt)),
                    Event::Window(evt) => Some(Message::WindowUpdate(evt)),
                    _ => None
                }
            )
        ])
    }
}