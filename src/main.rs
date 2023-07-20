use iced::{Application, Result, Settings, executor, Length};
use iced::widget::{Button, Text, Row, Column, Container, svg, Rule};
use iced::theme;
use iced_style::Theme;
use serde_derive::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fs::{DirEntry, Metadata};
use std::{env, fs, vec};
use std::path::PathBuf;
use std::process::Command;
use freedesktop_icons::lookup;
use xdg_utils::query_mime_info;
use toml;

fn main() -> Result {
    let mut settings = Settings::default();
    settings.exit_on_close_request =  false;
    Narwhal::run(settings)
}

const EST_LENGTH: u32 = 84;
const EST_HEIGHT: u32 = 104;
const FONT_SIZE: u16 = 16;
const SPACING: u16 = 10;
const MAX_LENGTH: usize = 10;
const SIDEBAR_WIDTH: u16 = 100;
const THEME: &str = "Adwaita";
const IMAGE_SCALE: u16 = 64;
const RULE_WIDTH: u16 = 5;

struct Narwhal {
    files: Vec<DirEntry>,
    currentpath: PathBuf,
    sorttype: SortType,
    desired_cols: u32,
    desired_rows: u32,
    show_hidden: bool,
    last_clicked_file: Option<usize>,
    uifiles: Vec<UIFile>,
    icon_cache: HashMap<String, String>,
    bookmarked_dirs: Vec<BookmarkDir>,
    deletion_confirmation: bool,
    mv_target: Option<String>,
    cp_target: Option<String>,
}
fn get_cache_home() -> String {
    match env::var("XDG_CACHE_HOME") {
        Ok(x) => x,
        Err(..) => match env::var("HOME") {
            Ok(x) => format!("{x}/.cache"),
            Err(..) => panic!("bailing out, you're on your own")
        }
    }
}
#[derive(Serialize, Deserialize, Clone)]
struct CachedIcon {
    path: String,
    icon: String
}
#[derive(Serialize, Deserialize, Clone)]
struct BookmarkDir {
    name: String,
    path: String
}
#[derive(Serialize, Deserialize, Clone)]
struct Config {
    sort_mode: String,
    show_hidden: bool,
    bookmarks: Vec<BookmarkDir>
}
#[derive(Serialize, Deserialize, Clone)]
struct CacheFile {
    contents: HashMap<String, String>
}
fn encode_sort(sort_type: SortType) -> String {
    match sort_type {
        SortType::Alphabetical => "Alphabetical".to_string(),
        SortType::Reverse => "Reverse".to_string(),
        SortType::Folders => "Folders".to_string(),
        SortType::Files => "Files".to_string(),
    }
}
fn decode_sort(sort_type: String) -> SortType {
    let test = String::as_str(&sort_type);
    match test {
        "Alphabetical" => SortType::Alphabetical,
        "Reverse" => SortType::Reverse,
        "Folders" => SortType::Folders,
        "Files" => SortType::Files,
        &_ => SortType::Folders
    }
}
fn get_config_home() -> String {
    match env::var("XDG_CONFIG_HOME") {
        Ok(x) => x,
        Err(..) => match env::var("HOME") {
            Ok(x) => format!("{x}/.config"),
            Err(..) => panic!("bailing out, you're on your own")
        }
    }
}
#[derive(Debug, Clone)]
enum Message {
    FileClicked(usize),
    GoBack,
    SortChanged,
    HiddenChanged,
    KeyboardUpdate(iced::keyboard::Event),
    WindowUpdate(iced::window::Event),
    BookmarkCurrent,
    BookmarkClicked(usize),
    DeleteClicked,
    MvClicked,
    CpClicked,
}

#[derive(PartialEq)]
enum FileType {
    Folder,
    File,
    Link
}
#[derive(Clone)]
struct UIFile {
    name: String,
    original_index: usize,
    selected: bool,
    icon: String,
}

fn ui_file_to_btn<'a>(lazy: UIFile) -> Column<'a, Message> {
    let file_icon = lazy.icon.clone();
    let handle = svg::Handle::from_path(file_icon);
    let image = svg(handle).height(IMAGE_SCALE).width(IMAGE_SCALE);
    let text = Text::new(clip_file_name(lazy.name.clone())).size(FONT_SIZE);
    let button = if lazy.selected {
        Button::new(image).on_press(Message::FileClicked(lazy.original_index))
    } else {
        Button::new(image).on_press(Message::FileClicked(lazy.original_index)).style(theme::Button::Text)
    };
    Column::new().push(button).push(text).align_items(iced::Alignment::Center)
}

#[derive(Clone)]
enum SortType {
    Alphabetical,
    Reverse,
    Folders,
    Files,
}
fn clean_bad_mime(mime: String) -> String {
    let substrings_type: Vec<&str> = mime.split("-").collect();
    let category = substrings_type[0].to_string();
    if category == "application".to_string() {
        String::from("application-x-executable")
    } else if mime == "inode-directory" {
        String::from("folder")
    } else {
        format!("{}-x-generic", category)
    }
}
fn get_file_mimetype(path: String) -> String {
    let raw_data = match query_mime_info(path) {
        Ok(x) => x,
        Err(x) => panic!("{}", x)  
    };
    match std::str::from_utf8(&raw_data) {
        Ok(x) => x.to_string(),
        Err(e) => panic!("{}", e)
    }
}
fn get_file_type(metadata: Metadata) -> FileType {
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
fn clip_file_name(name: String) -> String {
    let usename: Vec<char> = name.clone().chars().collect();
    let mut newvec: Vec<char> = vec![];
    let len = MAX_LENGTH;
    if name.chars().count() > len {
        for i in 0..len {
            newvec.push(usename[i]);
        }
        let newstr = newvec.into_iter().collect::<String>();
        format!("{newstr}...")
    } else {
        name
    }  
}
fn cacheless_get_file_icon(path: String) -> String {
    let mut mimetype = get_file_mimetype(path.clone()).replace("/", "-");
    if mimetype == "inode-directory" {
        String::from("folder")
    } else {
        match lookup(&mimetype).with_cache().with_size(32).with_theme(&THEME).find() {
            Some(..) => mimetype,
            None => {
                mimetype = clean_bad_mime(mimetype);
                match lookup(&mimetype).with_cache().with_size(32).with_theme(&THEME).find() {
                    Some(..) => mimetype,
                    None => format!("text-x-generic")
                }
            }
        }
    }
}
fn foldercmp(a: &DirEntry, b: &DirEntry, folders_first: bool) -> std::cmp::Ordering {
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
    fn regen_uifiles(&mut self) {
        let mut items_flushed = 0;
        let max_iter = self.desired_cols * self.desired_rows;
        self.uifiles = vec![];
        for i in 0..self.files.len() {
            if items_flushed == max_iter {
                break;
            }
            let name = self.files[i].file_name().to_string_lossy().to_string();
            let chars: Vec<char> = name.chars().collect();
            if !self.show_hidden && chars[0] == '.' {
            } else {
                let path = self.files[i].path().to_string_lossy().to_string();
                let selected = match self.last_clicked_file {
                    Some(value) => value == i,
                    None => false
                };
                let icon = self.get_file_icon(path.clone());
                let uifile = UIFile { name: name, original_index: i, selected: selected, icon: icon };
                self.uifiles.push(uifile);
                items_flushed = items_flushed + 1;
            }
        }
    }
    fn regen_files(&mut self) {
        self.files = vec![];
        let read_output = match fs::read_dir(self.currentpath.clone()) {
            Ok(x) => x,
            Err(x) => panic!("{}", x),
        };
        for path in read_output {
            self.files.push(path.unwrap())
        }
    }
    fn get_file_icon(&mut self, path: String) -> String {
        let icon_out = self.icon_cache.get(&path);
        match icon_out {
            Some(icon) => match lookup(icon).with_cache().with_size(32).with_theme(THEME).find() {
                Some(x) => x.to_string_lossy().to_string(),
                None => {
                    let newicon = clean_bad_mime(icon.clone());
                    match lookup(&newicon).with_cache().with_size(32).with_theme(THEME).find() {
                        Some(x) => x.to_string_lossy().to_string(),
                        None => format!("{}/resources/text-rust.svg", env!("CARGO_MANIFEST_DIR"))
                    }
                }
            }
            None => {
                let output = cacheless_get_file_icon(path.clone());
                self.icon_cache.insert(path, output.clone());
                lookup(&output).with_cache().with_size(32).with_theme(&THEME).find().unwrap().to_string_lossy().to_string()
            }
        }
    }
    fn interact_selected_entry(&mut self, index: usize) {
        match self.last_clicked_file {
            Some(x) => {
                if x == index {
                    let metadata = self.files[x].metadata().unwrap();
                    if metadata.is_symlink() {
                        let mime = get_file_mimetype(self.files[x].path().to_string_lossy().to_string()).replace("/", "-");
                        let path = fs::read_link(self.files[x].path().to_string_lossy().to_string()).unwrap();
                        if mime == "inode-directory" {
                            let pathtxt = path.to_string_lossy().to_string();
                            let pathchars: Vec<char> = pathtxt.chars().collect();
                            let cleanpath = if pathchars[0] == '/' {
                                pathtxt
                            } else {
                                format!("/{}", pathtxt)
                            };
                            self.currentpath = PathBuf::from(cleanpath);
                            println!("{}", self.currentpath.to_string_lossy());
                            self.regen_files();
                            sort_file_by_type(&mut self.files, self.sorttype.clone());
                        } else {
                            Command::new("open").arg(path.to_string_lossy().to_string()).spawn().expect("oops");
                        }
                    } else {
                        let mime = get_file_mimetype(self.files[x].path().to_string_lossy().to_string()).replace("/", "-");
                        if mime == "inode-directory" {
                            let filename = self.files[x].path().display().to_string();
                            self.currentpath.push(filename);
                            println!("{}", self.currentpath.to_string_lossy());
                            self.regen_files();
                            sort_file_by_type(&mut self.files, self.sorttype.clone());
                        } else {
                            let filename = self.files[x].path().display().to_string();
                            Command::new("open").arg(filename).spawn().expect("oops");
                        }
                    }
                    self.last_clicked_file = None;
                } else {
                    self.last_clicked_file = Some(index);
                }
                self.regen_uifiles();
            }
            None => {
                self.last_clicked_file = Some(index);
                self.regen_uifiles();
            }
        }
    }
    fn go_back_directory(&mut self) {
        self.currentpath.pop();
        self.regen_files();
        sort_file_by_type(&mut self.files, self.sorttype.clone());
        self.last_clicked_file = None;
        self.regen_uifiles();
    }
    fn change_sort(&mut self, reverse: bool) {
        if reverse {
            self.sorttype = match self.sorttype {
                SortType::Alphabetical => SortType::Files,
                SortType::Reverse => SortType::Alphabetical,
                SortType::Folders => SortType::Reverse,
                SortType::Files => SortType::Folders,
            };
        } else {
            self.sorttype = match self.sorttype {
                SortType::Alphabetical => SortType::Reverse,
                SortType::Reverse => SortType::Folders,
                SortType::Folders => SortType::Files,
                SortType::Files => SortType::Alphabetical,
            }; 
        }
        sort_file_by_type(&mut self.files, self.sorttype.clone());
        self.last_clicked_file = None;
        self.regen_uifiles();
    }
    fn rm_file(&mut self, index: usize) {
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
        sort_file_by_type(&mut self.files, self.sorttype.clone());
        self.last_clicked_file = None;
        self.regen_uifiles();
    }
    fn mv_file(&mut self) {
        let target = self.mv_target.clone().unwrap();
        let path = self.currentpath.to_string_lossy().to_string();
        Command::new("mv").arg(target).arg(path).output().unwrap();
        self.mv_target = None;
        self.regen_files();
        sort_file_by_type(&mut self.files, self.sorttype.clone());
        self.last_clicked_file = None;
        self.regen_uifiles();
    }
    fn cp_file(&mut self) {
        let target = self.cp_target.clone().unwrap();
        let path = self.currentpath.to_string_lossy().to_string();
        Command::new("cp").arg(target).arg(path).output().unwrap();
        self.cp_target = None;
        self.regen_files();
        sort_file_by_type(&mut self.files, self.sorttype.clone());
        self.last_clicked_file = None;
        self.regen_uifiles();
    }
}
fn sort_file_by_type(input: &mut Vec<DirEntry>, sort_type: SortType) {
    match sort_type {
        SortType::Alphabetical => {
            input.sort_by(|a, b| a.file_name().to_string_lossy().to_string().partial_cmp( &b.file_name().to_string_lossy().to_string()).unwrap())
        }
        SortType::Reverse => {
            input.sort_by(|a, b| b.file_name().to_string_lossy().to_string().partial_cmp( &a.file_name().to_string_lossy().to_string()).unwrap())
        }
        SortType::Files => {
            input.sort_by(|a, b| foldercmp(a, b, false))
        }
        SortType::Folders => {
            input.sort_by(|a, b| foldercmp(a, b, true))
        }
    }
}

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
        let config_home = format!("{}/NarwhalFM.toml", get_config_home());
        let config_text = fs::read_to_string(config_home);
        let config_struct: Config = match config_text {
            Ok(x) => toml::from_str(&x).unwrap(),
            Err(..) => Config { sort_mode: "Folder".to_string(), show_hidden: false, bookmarks: vec![] }
        };
        let mut finalstruct = Narwhal { files: vec![], currentpath: current_dir, sorttype: decode_sort(config_struct.sort_mode), desired_cols: 5, show_hidden: config_struct.show_hidden, desired_rows: 5, last_clicked_file: None, uifiles: vec![], icon_cache: cache_struct.contents.clone(), bookmarked_dirs: config_struct.bookmarks.clone(), deletion_confirmation: false, mv_target: None, cp_target: None};
        finalstruct.regen_files();
        finalstruct.regen_uifiles();
        finalstruct
    }
}

impl Application for Narwhal {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();
    fn new(_flags: ()) -> (Self, iced::Command<Self::Message>) {
        (
            Self::default(),
            iced::Command::none()
        )
    }
    fn title(&self) -> String {
        String::from("Narwhal File Manager")
    }
    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        let mut tempfiles: Vec<String> = vec![];
        for file in &self.files {
            let temp = file.file_name().to_string_lossy().to_string();
            tempfiles.push(temp);
        };
        match message {
            Message::FileClicked(x) => {
                self.interact_selected_entry(x);
                iced::Command::none()
            },
            Message::GoBack => {
                self.go_back_directory();
                iced::Command::none()
            },
            Message::SortChanged => {
                self.change_sort(false);
                iced::Command::none()
            }
            Message::HiddenChanged => {
                self.show_hidden = !self.show_hidden;
                self.regen_uifiles();
                iced::Command::none()
            }
            Message::BookmarkCurrent => {
                let dir = self.currentpath.to_string_lossy().to_string();
                let paths: Vec<&str> = dir.split('/').into_iter().collect();
                let name = paths[paths.len()-1].to_string();
                let bookmark = BookmarkDir { name: name, path: dir };
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
            Message::BookmarkClicked(index) => {
                self.currentpath = PathBuf::from(self.bookmarked_dirs[index].path.clone());
                self.regen_files();
                sort_file_by_type(&mut self.files, self.sorttype.clone());
                self.last_clicked_file = None;
                self.regen_uifiles();
                iced::Command::none()
            }
            Message::KeyboardUpdate(kb_event) => {
                match kb_event {
                    iced::keyboard::Event::KeyPressed { key_code, modifiers } => {
                        if key_code == iced::keyboard::KeyCode::Left {
                            let mut old_index = self.uifiles.len() - 1;
                            for i in 0..self.uifiles.len() {
                                match self.last_clicked_file {
                                    Some(x) => {
                                        if self.uifiles[i].original_index == x && i != 0{
                                            old_index = i - 1;
                                            break;
                                        }
                                    },
                                    None => {},
                                }
                            }
                            self.last_clicked_file = Some(self.uifiles[old_index].original_index);
                            self.regen_uifiles();
                        }
                        if key_code == iced::keyboard::KeyCode::Right {
                            let mut old_index = 0;
                            for i in 0..self.uifiles.len() {
                                match self.last_clicked_file {
                                    Some(x) => {
                                        if self.uifiles[i].original_index == x && i != self.uifiles.len() - 1 {
                                            old_index = i + 1;
                                            break;
                                        }
                                    },
                                    None => {},
                                }
                            }
                            self.last_clicked_file = Some(self.uifiles[old_index].original_index);
                            self.regen_uifiles();
                        }
                        if key_code == iced::keyboard::KeyCode::Down {
                            let mut old_index = None;
                            for i in 0..self.uifiles.len() {
                                match self.last_clicked_file {
                                    Some(x) => {
                                        if self.uifiles[i].original_index == x {
                                            old_index = Some(i);
                                            break;
                                        }
                                    },
                                    None => {},
                                }
                            }
                            old_index = match old_index {
                                Some(x) => {
                                    if self.desired_cols as usize + x < self.uifiles.len() {
                                        Some(x + self.desired_cols as usize)
                                    } else {
                                        Some(x % self.desired_cols as usize)
                                    }
                                }
                                None => {
                                    Some(0)
                                }
                            };
                            self.last_clicked_file = Some(self.uifiles[old_index.unwrap()].original_index);
                            self.regen_uifiles();
                        }
                        if key_code == iced::keyboard::KeyCode::Up {
                            let mut old_index = None;
                            for i in 0..self.uifiles.len() {
                                match self.last_clicked_file {
                                    Some(x) => {
                                        if self.uifiles[i].original_index == x {
                                            old_index = Some(i);
                                            break;
                                        }
                                    },
                                    None => {},
                                }
                            }
                            old_index = match old_index {
                                Some(x) => {
                                    if x >= self.desired_cols as usize {
                                        Some(x - self.desired_cols as usize)
                                    } else {
                                        let length = (self.uifiles.len() - 1) as u32;
                                        let offset = x as u32;
                                        let mut final_value = 0;
                                        for i in 0..self.desired_rows {
                                            if i*self.desired_cols+offset > length {
                                                final_value = i-1;
                                                final_value = final_value*self.desired_cols+offset;
                                                break;
                                            }
                                        }
                                        Some(final_value as usize)
                                    }
                                }
                                None => {
                                    Some(0)
                                }
                            };
                            self.last_clicked_file = Some(self.uifiles[old_index.unwrap()].original_index);
                            self.regen_uifiles();
                        }
                        if key_code == iced::keyboard::KeyCode::Enter {
                            match self.last_clicked_file {
                                Some(x) => self.interact_selected_entry(x),
                                None => {}
                            }
                        }
                        if key_code == iced::keyboard::KeyCode::Backspace {
                            self.go_back_directory();
                        }
                        if key_code == iced::keyboard::KeyCode::S && modifiers == iced::keyboard::Modifiers::SHIFT {
                            self.change_sort(true);
                        } else if key_code == iced::keyboard::KeyCode::S {
                            self.change_sort(false);
                        }
                        if key_code == iced::keyboard::KeyCode::H {
                            self.show_hidden = !self.show_hidden;
                            self.regen_uifiles();
                        }
                        if key_code == iced::keyboard::KeyCode::Minus {
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
                        }
                    }
                    iced::keyboard::Event::KeyReleased { key_code: _, modifiers: _ } => {},
                    iced::keyboard::Event::CharacterReceived(_) => {},
                    iced::keyboard::Event::ModifiersChanged(_) => {},
                }
                iced::Command::none()
            }
            Message::WindowUpdate(win_event) => {
                match win_event {
                    iced::window::Event::Moved { x: _, y: _ } => {iced::Command::none()},
                    iced::window::Event::Resized { width, height } => {
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
                            self.regen_uifiles();
                        }
                        iced::Command::none()
                    },
                    iced::window::Event::RedrawRequested(_) => {iced::Command::none()},
                    iced::window::Event::CloseRequested => {
                        let yes = CacheFile { contents: self.icon_cache.clone() };
                        let cached_contents = toml::to_string(&yes).unwrap();
                        let cache_home = format!("{}/NarwhalFM", get_cache_home());
                        fs::write(cache_home, cached_contents).unwrap();
                        let config_file = Config { sort_mode: encode_sort(self.sorttype.clone()), show_hidden: self.show_hidden, bookmarks: self.bookmarked_dirs.clone() };
                        let config_text = toml::to_string(&config_file).unwrap();
                        let config_home = format!("{}/NarwhalFM.toml", get_config_home());
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
            Message::DeleteClicked => {
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
            Message::MvClicked => {
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
            Message::CpClicked => {
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
        }
    }
    fn view(&self) -> iced::Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
        let back_btn = Button::new("Back").on_press(Message::GoBack);
        let sort_btn = Button::new("Sort").on_press(Message::SortChanged);
        let delete_btn = if self.deletion_confirmation {
            Button::new("Delete").on_press(Message::DeleteClicked).style(theme::Button::Destructive)
        } else {
            Button::new("Delete").on_press(Message::DeleteClicked).style(theme::Button::Secondary)
        };
        let mv_btn = match self.mv_target {
            Some(..) => Button::new("Move").on_press(Message::MvClicked),
            None => Button::new("Move").on_press(Message::MvClicked).style(theme::Button::Secondary)
        };
        let cp_btn = match self.cp_target {
            Some(..) => Button::new("Paste").on_press(Message::CpClicked),
            None => Button::new("Copy").on_press(Message::CpClicked).style(theme::Button::Secondary)
        };
        let hidden_btn = Button::new("Hidden").on_press(Message::HiddenChanged);
        let bookmark_btn = Button::new("Bookmark").on_press(Message::BookmarkCurrent);
        let function_buttons = Row::new().push(back_btn).push(sort_btn).push(hidden_btn).push(bookmark_btn).push(delete_btn).push(mv_btn).push(cp_btn);
        let mut bookmark_buttons = Column::new();
        for i in 0..self.bookmarked_dirs.len() {
            let btn_text = Text::new(self.bookmarked_dirs[i].name.clone());
            let btn = Button::new(btn_text).on_press(Message::BookmarkClicked(i)).width(SIDEBAR_WIDTH).style(theme::Button::Text);
            bookmark_buttons = bookmark_buttons.push(btn)
        }
        let mut file_listing = Column::new();
        let mut temprow = Row::new();
        for i in 0..self.uifiles.len() {
            let full = ui_file_to_btn(self.uifiles[i].clone());
            if i % self.desired_cols as usize == 0 {
                file_listing = file_listing.push(temprow);
                temprow = Row::new().spacing(SPACING);
            }
            temprow = temprow.push(full);
        }
        file_listing = file_listing.push(temprow);
        let ruleh = Rule::horizontal(RULE_WIDTH);
        let rulev = Rule::vertical(RULE_WIDTH);
        let col_test = Column::new().push(function_buttons).push(ruleh).push(file_listing);
        let row_test = Row::new().push(bookmark_buttons).push(rulev).push(col_test);
        Container::new(row_test).width(Length::Fill).height(Length::Fill).into()
    }
    fn subscription(&self) -> iced::Subscription<Message> {
        iced::subscription::events_with(
            |event, _| {
                if let iced::Event::Keyboard(keyboard_event) = event {
                    Some(Message::KeyboardUpdate(keyboard_event))
                } else if let iced::Event::Window(window_event) = event{
                    Some(Message::WindowUpdate(window_event))
                } else {
                    None
                }
            }
        )
    }
}