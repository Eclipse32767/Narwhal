use iced::{Application, Result, Settings, executor, Length};
use iced::widget::{Button, Text, Row, Column, Container};
use iced_style::Theme;
use std::fs::DirEntry;
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
}

impl Default for Narwhal {
    fn default() -> Self {
        let current_dir = match env::current_dir() {
            Ok(x) => x,
            Err(..) => panic!()
        };
        let read_output = match fs::read_dir(current_dir) {
            Ok(x) => x,
            Err(..) => panic!()
        };
        let mut filelist = vec![];
        for path in read_output {
            filelist.push(path.unwrap());
        }
        let current_dir = match env::current_dir() {
            Ok(x) => x,
            Err(..) => panic!()
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
                self.currentpath.push(tempfiles[x].clone());
                println!("{}", tempfiles[x].clone());
                self.files = vec![];
                let read_output = match fs::read_dir(self.currentpath.clone()) {
                    Ok(x) => x,
                    Err(y) => panic!("{}", y),
                };
                for path in read_output {
                    self.files.push(path.unwrap())
                }
            },
        }
        iced::Command::none()
    }
    fn view(&self) -> iced::Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
        let btn_test = Button::new("test");
        let mut file_listing = Column::new();
        for i in 0..self.files.len() {
            let filename = self.files[i].file_name();
            let file_string: String = filename.to_string_lossy().to_string();
            let text = Text::new(file_string);
            let button = Button::new(text).on_press(Message::FileClicked(i));
            file_listing = file_listing.push(button)
        }
        let row_test = Row::new().push(btn_test).push(file_listing);
        let column_test = Column::new().push(row_test);
        Container::new(column_test).width(Length::Fill).height(Length::Fill).into()
    }
}