use iced::{Application, Result, Settings, executor, Length};
use iced::widget::{Button, Text, Row, Column, Container, svg};
use iced_style::Theme;
use std::fs::{DirEntry, Metadata};
use std::{env, fs};
use std::path::{PathBuf};

fn main() -> Result {
    Narwhal::run(Settings::default())
}


struct Narwhal {
    files: Vec<DirEntry>,
    currentpath: PathBuf
}

#[derive(Debug, Clone)]
enum Message {
    FileClicked(usize),
    GoBack,
}

enum FileType {
    Folder,
    File,
    Link
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
        let current_dir = match env::current_dir() {
            Ok(x) => x,
            Err(x) => panic!("{}", x)
        };
        Narwhal { files: filelist, currentpath: current_dir }
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
            }
        }
        iced::Command::none()
    }
    fn view(&self) -> iced::Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
        let back_btn = Button::new("Back").on_press(Message::GoBack);
        let mut file_listing = Row::new();
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
        let row_test = Row::new().push(back_btn).push(file_listing);
        let column_test = Column::new().push(row_test);
        Container::new(column_test).width(Length::Fill).height(Length::Fill).into()
    }
}