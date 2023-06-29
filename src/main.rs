use iced::{Application, Result, Settings, executor, Length};
use iced::widget::{Button, Text, Row, Column, Container, svg};
use iced::theme;
use iced_style::Theme;
use serde_derive::{Serialize, Deserialize};
use std::fs::{DirEntry, Metadata};
use std::{env, fs, vec};
use std::path::{PathBuf};
use std::process::{Command};
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
const SIDEBAR_WIDTH: u16 = 75;

struct Narwhal {
    files: Vec<DirEntry>,
    currentpath: PathBuf,
    sorttype: SortType,
    desired_cols: u32,
    desired_rows: u32,
    show_hidden: bool,
    last_clicked_file: Option<usize>,
    uifiles: Vec<UIFile>,
    icon_cache: Vec<CachedIcon>,
    bookmarked_dirs: Vec<BookmarkDir>
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
struct BookmarkDir {
    name: String,
    path: String
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
}
#[derive(Serialize, Deserialize)]
struct FlushCache {
    icons: Vec<CachedIcon>
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
    let image = svg(handle);
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
fn cacheless_get_file_icon(filetype: FileType, path: String) -> String {
    match filetype {
        FileType::File => {
            let mut mimetype = get_file_mimetype(path).replace("/", "-");
            match lookup(&mimetype).with_cache().with_size(64).with_theme("breeze").find() {
                Some(x) => x.to_string_lossy().to_string(),
                None => {
                    println!("{mimetype}");
                    mimetype = clean_bad_mime(mimetype);
                    match lookup(&mimetype).with_cache().with_size(64).with_theme("breeze").find() {
                        Some(x) => x.to_string_lossy().to_string(),
                        None => format!("{}/resources/text-rust.svg", env!("CARGO_MANIFEST_DIR"))
                    }
                }
            }
        }
        FileType::Folder => {
            lookup("folder").with_cache().with_size(64).with_theme("breeze").find().unwrap().to_string_lossy().to_string()
        }
        FileType::Link => {
            format!("{}/resources/text-rust.svg", env!("CARGO_MANIFEST_DIR"))
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
                let metadata = match self.files[i].metadata() {
                    Ok(x) => x,
                    Err(x) => panic!("{}", x)
                };
                let selected = match self.last_clicked_file {
                    Some(value) => value == i,
                    None => false
                };
                let icon = self.get_file_icon(get_file_type(metadata.clone()), path.clone());
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
    fn get_file_icon(&mut self, filetype: FileType, path: String) -> String {
        let mut icon_out = None;
        for cached_value in &self.icon_cache {
            if path == cached_value.path {
                icon_out = Some(cached_value.icon.clone());
                break;
            }
        }
        match icon_out {
            Some(icon) => icon,
            None => {
                let output = cacheless_get_file_icon(filetype, path.clone());
                self.icon_cache.push(CachedIcon { path: path, icon: output.clone() });
                output
            }
        }
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
        let read_output = match fs::read_dir(current_dir) {
            Ok(x) => x,
            Err(x) => panic!("{}", x)
        };
        let mut filelist = vec![];
        for path in read_output {
            filelist.push(path.unwrap());
        }
        sort_file_by_type(&mut filelist, SortType::Alphabetical);
        let current_dir = match env::current_dir() {
            Ok(x) => x,
            Err(x) => panic!("{}", x)
        };
        let mut uifiles = vec![];
        for i in 0..filelist.len() {
            let name = filelist[i].file_name().to_string_lossy().to_string();
            let path = filelist[i].path().to_string_lossy().to_string();
            let metadata = match filelist[i].metadata() {
                Ok(x) => x,
                Err(x) => panic!("{}", x)
            };
            let selected = false;
            let icon = cacheless_get_file_icon(get_file_type(metadata.clone()), path.clone());
            let uifile = UIFile { name: name, original_index: i, selected: selected, icon: icon };
            uifiles.push(uifile);
        }
        let cache_home = format!("{}/NarwhalFM", get_cache_home());
        let cache_text = fs::read_to_string(cache_home);
        let cache_struct: FlushCache = match cache_text {
            Ok(x) => toml::from_str(&x).unwrap(),
            Err(..) => FlushCache { icons: vec![] }
        };
        Narwhal { files: filelist, currentpath: current_dir, sorttype: SortType::Alphabetical, desired_cols: 5, show_hidden: true, desired_rows: 5, last_clicked_file: None, uifiles: uifiles, icon_cache: cache_struct.icons.clone(), bookmarked_dirs: vec![]}
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
                match self.last_clicked_file {
                    Some(value) => {
                        if value == x {
                            let filetype = get_file_type(self.files[x].metadata().expect("this should never happen"));
                            match filetype {
                                FileType::File => {
                                    let filename = self.files[x].path().display().to_string();
                                    Command::new("open").arg(filename).spawn().expect("oops");
                                }
                                FileType::Folder => {
                                    self.currentpath.push(tempfiles[x].clone());
                                    self.regen_files();
                                    sort_file_by_type(&mut self.files, self.sorttype.clone());
                                }
                                FileType::Link => {

                                }
                            }
                            self.last_clicked_file = None;
                            self.regen_uifiles();
                        } else {
                            self.last_clicked_file = Some(x);
                            self.regen_uifiles();
                        }
                    }
                    None => {
                        self.last_clicked_file = Some(x);
                        self.regen_uifiles();
                    }
                }
                iced::Command::none()
            },
            Message::GoBack => {
                self.currentpath.pop();
                self.regen_files();
                sort_file_by_type(&mut self.files, self.sorttype.clone());
                self.last_clicked_file = None;
                self.regen_uifiles();
                iced::Command::none()
            },
            Message::SortChanged => {
                self.sorttype = match self.sorttype {
                    SortType::Alphabetical => SortType::Reverse,
                    SortType::Reverse => SortType::Folders,
                    SortType::Folders => SortType::Files,
                    SortType::Files => SortType::Alphabetical,
                };
                sort_file_by_type(&mut self.files, self.sorttype.clone());
                self.last_clicked_file = None;
                self.regen_uifiles();
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
            Message::KeyboardUpdate(_kb_event) => {
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
                        let yes = FlushCache { icons: self.icon_cache.clone() };
                        let cached_contents = toml::to_string(&yes).unwrap();
                        let cache_home = format!("{}/NarwhalFM", get_cache_home());
                        fs::write(cache_home, cached_contents).unwrap();
                        iced::window::close()
                    },
                    iced::window::Event::Focused => {iced::Command::none()},
                    iced::window::Event::Unfocused => {iced::Command::none()},
                    iced::window::Event::FileHovered(_) => {iced::Command::none()},
                    iced::window::Event::FileDropped(_) => {iced::Command::none()},
                    iced::window::Event::FilesHoveredLeft => {iced::Command::none()},
                }
            }
        }
    }
    fn view(&self) -> iced::Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
        let back_btn = Button::new("Back").on_press(Message::GoBack).width(SIDEBAR_WIDTH);
        let sort_btn = Button::new("Sort").on_press(Message::SortChanged).width(SIDEBAR_WIDTH);
        let hidden_btn = Button::new("Hidden").on_press(Message::HiddenChanged).width(SIDEBAR_WIDTH);
        let bookmark_btn = Button::new("Bookmark").on_press(Message::BookmarkCurrent).width(SIDEBAR_WIDTH);
        let mut function_buttons = Column::new().push(back_btn).push(sort_btn).push(hidden_btn).push(bookmark_btn);
        for i in 0..self.bookmarked_dirs.len() {
            let btn_text = Text::new(self.bookmarked_dirs[i].name.clone());
            let btn = Button::new(btn_text).on_press(Message::BookmarkClicked(i)).width(SIDEBAR_WIDTH);
            function_buttons = function_buttons.push(btn)
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
        let row_test = Row::new().push(function_buttons).push(file_listing).spacing(SPACING);
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