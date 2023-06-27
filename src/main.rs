use iced::{Application, Result, Settings, executor, Length};
use iced::widget::{Button, Text, Row, Column, Container, svg};
use iced::theme;
use iced_style::Theme;
use std::fs::{DirEntry, Metadata};
use std::{env, fs};
use std::path::{PathBuf};
use std::process::Command;
use freedesktop_icons::lookup;
use xdg_utils::query_mime_info;

fn main() -> Result {
    Narwhal::run(Settings::default())
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
    uifiles: Vec<UIFile>
}

#[derive(Debug, Clone)]
enum Message {
    FileClicked(usize),
    GoBack,
    SortChanged,
    HiddenChanged,
    KeyboardUpdate(iced::keyboard::Event),
    WindowUpdate(iced::window::Event)
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
    path: String,
    metadata: Metadata,
    original_index: usize,
    selected: bool
}

fn ui_file_to_btn<'a>(lazy: UIFile) -> Column<'a, Message> {
    let filetype = get_file_type(lazy.metadata.clone());
    let file_icon = get_file_icon(filetype, lazy.path.clone());
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
fn get_file_icon(filetype: FileType, path: String) -> String {
    match filetype {
        FileType::File => {
            let mimetype = query_mime_info(path).map_err(|_| ()).map(|bytes| String::from_utf8_lossy(&bytes).into_owned());
            let mut fixed_type = match mimetype {
                Ok(x) => x.replace("/", "-"),
                Err(..) => panic!("lol")
            };
            match lookup(&fixed_type).with_cache().with_size(64).with_theme("breeze").find() {
                Some(x) => x.to_string_lossy().to_string(),
                None => {
                    println!("{fixed_type}");
                    fixed_type = clean_bad_mime(fixed_type);
                    match lookup(&fixed_type).with_cache().with_size(64).with_theme("breeze").find() {
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
        Narwhal { files: filelist, currentpath: current_dir, sorttype: SortType::Alphabetical, desired_cols: 5, show_hidden: true, desired_rows: 5, last_clicked_file: None, uifiles: vec![]}
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
                                    println!("{}", tempfiles[x].clone());
                                    self.files = vec![];
                                    let read_output = match fs::read_dir(self.currentpath.clone()) {
                                        Ok(x) => x,
                                        Err(x) => panic!("{}", x),
                                    };
                                    for path in read_output {
                                        self.files.push(path.unwrap())
                                    }
                                    sort_file_by_type(&mut self.files, self.sorttype.clone());
                                    self.last_clicked_file = None
                                }
                                FileType::Link => {

                                }
                            }
                        } else {
                            self.last_clicked_file = Some(x)
                        }
                    }
                    None => {
                        self.last_clicked_file = Some(x)
                    }
                }
            },
            Message::GoBack => {
                self.currentpath.pop();
                self.files = vec![];
                let read_output = match fs::read_dir(self.currentpath.clone()) {
                    Ok(x) => x,
                    Err(x) => panic!("{}", x),
                };
                for path in read_output {
                    self.files.push(path.unwrap())
                }
                sort_file_by_type(&mut self.files, self.sorttype.clone());
                self.last_clicked_file = None
            },
            Message::SortChanged => {
                self.sorttype = match self.sorttype {
                    SortType::Alphabetical => SortType::Reverse,
                    SortType::Reverse => SortType::Folders,
                    SortType::Folders => SortType::Files,
                    SortType::Files => SortType::Alphabetical,
                };
                sort_file_by_type(&mut self.files, self.sorttype.clone())
            }
            Message::HiddenChanged => {
                self.show_hidden = !self.show_hidden;
            }
            Message::KeyboardUpdate(_kb_event) => {

            }
            Message::WindowUpdate(win_event) => {
                match win_event {
                    iced::window::Event::Moved { x: _, y: _ } => {},
                    iced::window::Event::Resized { width, height } => {
                        let adjusted_width = width - SIDEBAR_WIDTH as u32;
                        self.desired_cols = adjusted_width / EST_LENGTH;
                        let adjusted_height = height;
                        self.desired_rows = adjusted_height / EST_HEIGHT;
                    },
                    iced::window::Event::RedrawRequested(_) => {},
                    iced::window::Event::CloseRequested => {},
                    iced::window::Event::Focused => {},
                    iced::window::Event::Unfocused => {},
                    iced::window::Event::FileHovered(_) => {},
                    iced::window::Event::FileDropped(_) => {},
                    iced::window::Event::FilesHoveredLeft => {},
                }
            }
        }
        iced::Command::none()
    }
    fn view(&self) -> iced::Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
        let back_btn = Button::new("Back").on_press(Message::GoBack).width(SIDEBAR_WIDTH);
        let sort_btn = Button::new("Sort").on_press(Message::SortChanged).width(SIDEBAR_WIDTH);
        let hidden_btn = Button::new("Hidden").on_press(Message::HiddenChanged).width(SIDEBAR_WIDTH);
        let mut file_listing = Column::new();
        let mut temprow = Row::new();
        let mut rows_entered = 0;
        let mut newfiles = vec![];
        if self.show_hidden {
            for i in 0..self.files.len() {
                let filename = self.files[i].file_name().to_string_lossy().to_string();
                let directory = self.currentpath.to_string_lossy().to_string();
                let filepath = format!("{directory}/{filename}");
                let metadata = self.files[i].metadata().expect("uh oh");
                let selected = match self.last_clicked_file {
                    Some(value) => i == value,
                    None => false
                };
                let lazy = UIFile {name: filename.clone(), path: filepath, metadata: metadata, original_index: i, selected: selected};
                newfiles.push(lazy);
            }
        } else {
            for i in 0..self.files.len() {
                let filename = self.files[i].file_name().to_string_lossy().to_string();
                let directory = self.currentpath.to_string_lossy().to_string();
                let filepath = format!("{directory}/{filename}");
                let metadata = self.files[i].metadata().expect("uh oh");
                let selected = match self.last_clicked_file {
                    Some(value) => i == value,
                    None => false
                };
                let lazy = UIFile {name: filename.clone(), path: filepath, metadata: metadata, original_index: i, selected: selected};
                let chars_vec: Vec<char> = filename.chars().collect();
                if chars_vec[0] != '.' {
                    newfiles.push(lazy);
                }
            }
        }
        for i in 0..newfiles.len() {
            if self.desired_rows >= rows_entered {
                let full = ui_file_to_btn(newfiles[i].clone());
                if i % self.desired_cols as usize == 0 {
                    file_listing = file_listing.push(temprow);
                    temprow = Row::new().spacing(SPACING);
                    rows_entered = rows_entered + 1;
                }
                if self.desired_rows != rows_entered-1 {
                    temprow = temprow.push(full);
                }
            }
        }
        file_listing = file_listing.push(temprow);
        let function_buttons = Column::new().push(back_btn).push(sort_btn).push(hidden_btn);
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