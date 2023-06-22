use iced::{Application, Result, Settings, executor, Length};
use iced::widget::{Button, Text, Row, Column, Container, svg};
use iced_style::Theme;
use std::fs::{DirEntry, Metadata};
use std::{env, fs};
use std::path::{PathBuf};
use std::process::Command;

fn main() -> Result {
    Narwhal::run(Settings::default())
}


struct Narwhal {
    files: Vec<DirEntry>,
    currentpath: PathBuf,
    sorttype: SortType
}

#[derive(Debug, Clone)]
enum Message {
    FileClicked(usize),
    GoBack,
    SortChanged,
}

#[derive(PartialEq)]
enum FileType {
    Folder,
    File,
    Link
}
#[derive(Clone)]
enum SortType {
    Alphabetical,
    Reverse,
    Folders,
    Files,
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
        Narwhal { files: filelist, currentpath: current_dir, sorttype: SortType::Alphabetical}
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
                        sort_file_by_type(&mut self.files, self.sorttype.clone())
                    }
                    FileType::Link => {

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
                sort_file_by_type(&mut self.files, self.sorttype.clone())
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
        }
        iced::Command::none()
    }
    fn view(&self) -> iced::Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
        let back_btn = Button::new("Back").on_press(Message::GoBack);
        let sort_btn = Button::new("Sort").on_press(Message::SortChanged);
        let mut file_listing = Column::new();
        for i in 0..self.files.len() {
            let filetype = get_file_type(self.files[i].metadata().expect("this should never happen"));
            let file_icon = match filetype {
                FileType::File => format!("{}/resources/text-x-generic.svg", env!("CARGO_MANIFEST_DIR")),
                FileType::Folder => format!("{}/resources/folder-blue.svg", env!("CARGO_MANIFEST_DIR")),
                FileType::Link => format!("{}/resources/folder-blue.svg", env!("CARGO_MANIFEST_DIR")),
            };
            let handle = svg::Handle::from_path(file_icon);
            let image = svg(handle);
            let filename = self.files[i].file_name().to_string_lossy().to_string();
            let text = Text::new(filename);
            let button = Button::new(image).on_press(Message::FileClicked(i));
            let full = Column::new().push(button).push(text);
            file_listing = file_listing.push(full)
        }
        let function_buttons = Column::new().push(back_btn).push(sort_btn);
        let row_test = Row::new().push(function_buttons).push(file_listing);
        Container::new(row_test).width(Length::Fill).height(Length::Fill).into()
    }
}