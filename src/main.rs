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
        match message {

        }
        iced::Command::none()
    }
    fn view(&self) -> iced::Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
        let btn_test = Button::new("test");
        let mut unrolled_files = String::from("");
        for file in &self.files {
            unrolled_files = format!("{unrolled_files} \n {}", file.file_name().to_string_lossy())
        }
        let txt_test = Text::new(unrolled_files);
        let row_test = Row::new().push(btn_test).push(txt_test);
        let column_test = Column::new().push(row_test);
        Container::new(column_test).width(Length::Fill).height(Length::Fill).into()
    }
}