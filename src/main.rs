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
use libstyle::{ThemeSet, CustomTheme, ButtonStyle, ThemeFile, mk_app_theme, col_from_string};
mod libstyle;
use iconhelpers::{get_file_icon, get_file_mimetype};
mod iconhelpers;
use confighelpers::*;
mod confighelpers;
use uihelpers::*;
mod uihelpers;
mod kbparser;
mod defaultstate;
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

static RENAMEBTN: Lazy<id::Button> = Lazy::new(id::Button::unique);

struct Narwhal {//contains all application state
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
    themes: ThemeSet,
    theme: ThemeType,
    typemode: Option<String>,
    rename_id: text_input::Id,
    show_keybinds: bool,
    anims: Timeline,
    icntheme: String,
    icnsize: u16,
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
fn foldercmp(a: &DirEntry, b: &DirEntry, folders_first: bool) -> std::cmp::Ordering {//compare folders, returning an ordering
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
    async fn regen_uifiles(&mut self) {
        let mut items_flushed = 0;
        let max_iter = self.desired_cols * self.desired_rows;
        let mut futures = Vec::with_capacity(max_iter as usize);
        let mut names = Vec::with_capacity(max_iter as usize);
        let mut selectedvals = Vec::with_capacity(max_iter as usize);
        let mut originalindeces = Vec::with_capacity(max_iter as usize);
        let mut all_changes = vec![];
        let exec = iced::executor::Default::new().unwrap();
        self.uifiles = Vec::with_capacity(max_iter as usize);
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
                futures.push(exec.spawn(get_file_icon(self.icon_cache.clone(), path.clone(), self.icntheme.clone(), self.icnsize)));//spawn all file icon fetching futures
                names.push(name);
                selectedvals.push(selected);
                originalindeces.push(i);
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
            let uifile = UIFile { name: names[i].clone(), original_index: originalindeces[i], selected: selectedvals[i], icon: icon };//construct the UIFile and push it onto the vec
            self.uifiles.push(uifile);
        }
        for change in all_changes {//for every change, push it onto the cache
            self.icon_cache.extend(change.into_iter());
        }
        self.typemode = None;
    }
    fn regen_files(&mut self) {//rebuild filelist
        self.files = vec![];
        let read_output = match fs::read_dir(self.currentpath.clone()) {
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
                            Command::new("xdg-open").arg(path.to_string_lossy().to_string()).spawn().expect("oops");
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
                block_on(self.regen_uifiles());
            }
            None => {
                self.last_clicked_file = Some(index);
                block_on(self.regen_uifiles());
            }
        }
    }
    fn go_back_directory(&mut self) {//pop an entry off of the current path, regenerate the filelist and UIFiles
        self.currentpath.pop();
        self.regen_files();
        sort_file_by_type(&mut self.files, self.sorttype.clone());
        self.last_clicked_file = None;
        block_on(self.regen_uifiles());
    }
    fn change_sort(&mut self, reverse: bool) {//cycle through sort modes
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
        block_on(self.regen_uifiles());
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
        sort_file_by_type(&mut self.files, self.sorttype.clone());
        self.last_clicked_file = None;
        block_on(self.regen_uifiles());
    }
    fn mv_file(&mut self) {//move a file to another location, no sanity checks
        let target = self.mv_target.clone().unwrap();
        let path = self.currentpath.to_string_lossy().to_string();
        Command::new("mv").arg(target).arg(path).output().unwrap();
        self.mv_target = None;
        self.regen_files();
        sort_file_by_type(&mut self.files, self.sorttype.clone());
        self.last_clicked_file = None;
        block_on(self.regen_uifiles());
    }
    fn cp_file(&mut self) {//copy a file to another location, no sanity checks
        let target = self.cp_target.clone().unwrap();
        let path = self.currentpath.to_string_lossy().to_string();
        Command::new("cp").arg(target).arg(path).output().unwrap();
        self.cp_target = None;
        self.regen_files();
        sort_file_by_type(&mut self.files, self.sorttype.clone());
        self.last_clicked_file = None;
        block_on(self.regen_uifiles());
    }
    fn touch(&mut self) {
        let path = format!("{}/NewFile", self.currentpath.to_string_lossy().to_string());
        Command::new("touch").arg(path).output().unwrap();
        self.regen_files();
        sort_file_by_type(&mut self.files, self.sorttype.clone());
        self.last_clicked_file = None;
        block_on(self.regen_uifiles());
    }
    fn mkdir(&mut self) {
        let path = format!("{}/NewFolder", self.currentpath.to_string_lossy().to_string());
        Command::new("mkdir").arg(path).output().unwrap();
        self.regen_files();
        sort_file_by_type(&mut self.files, self.sorttype.clone());
        self.last_clicked_file = None;
        block_on(self.regen_uifiles());
    }
    fn rename(&mut self) {
        let src_path = self.files[self.last_clicked_file.unwrap()].path().to_string_lossy().to_string();
        let dest_path = format!("{}/{}", self.currentpath.to_string_lossy().to_string(), self.typemode.clone().unwrap());
        Command::new("mv").arg(src_path).arg(dest_path).output().unwrap();
        self.regen_files();
        sort_file_by_type(&mut self.files, self.sorttype.clone());
        self.last_clicked_file = None;
        block_on(self.regen_uifiles());
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
            input.sort_by(|a, b| foldercmp(a, b, false))
        }
        SortType::Folders => {
            input.sort_by(|a, b| foldercmp(a, b, true))
        }
    }
}

impl Application for Narwhal {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();
    fn new(_flags: ()) -> (Self, iced::Command<Self::Message>) {//initialize program
        let mut narwhal = Self::default();
        use cosmic_time::button;
        let unmitosis = chain![RENAMEBTN, 
            button(Duration::ZERO).width(Length::Fixed(75.0)).height(Length::Fixed(TOP_HEIGHT as f32)),
            button(Duration::from_millis(500)).width(Length::Fixed(75.0)).height(Length::Fixed(TOP_HEIGHT as f32)),
            button(Duration::from_millis(500)).width(Length::Fixed(1000.0)).height(Length::Fixed(TOP_HEIGHT as f32)),
        ];
        narwhal.anims.set_chain(unmitosis).start();
        (
            narwhal,
            iced::Command::none()
        )
    }
    fn title(&self) -> String {//Window title
        String::from("Narwhal File Manager")
    }
    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {//decide what to do based on message
        let mut tempfiles: Vec<String> = vec![];
        for file in &self.files {
            let temp = file.file_name().to_string_lossy().to_string();
            tempfiles.push(temp);
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
            Message::SortChanged => {//change sortmode
                self.change_sort(false);
                iced::Command::none()
            }
            Message::HiddenChanged => {//change hidden flag
                self.show_hidden = !self.show_hidden;
                block_on(self.regen_uifiles());
                iced::Command::none()
            }
            Message::BookmarkCurrent => {//bookmark or unbookmark current dir
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
            Message::BookmarkClicked(index) => {//go to the bookmark's chosen dir
                self.currentpath = PathBuf::from(self.bookmarked_dirs[index].path.clone());
                self.regen_files();
                sort_file_by_type(&mut self.files, self.sorttype.clone());
                self.last_clicked_file = None;
                block_on(self.regen_uifiles());
                iced::Command::none()
            }
            Message::KeyboardUpdate(kb_event) => {//send to keyboard parser
                self.kbparse(kb_event)
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
                            block_on(self.regen_uifiles());
                        }
                        iced::Command::none()
                    },
                    iced::window::Event::RedrawRequested(_) => {iced::Command::none()},
                    iced::window::Event::CloseRequested => {//write cache and config, then close
                        let yes = CacheFile { contents: self.icon_cache.clone() };
                        let cached_contents = toml::to_string(&yes).unwrap();
                        let cache_home = format!("{}/NarwhalFM", get_cache_home());
                        fs::write(cache_home, cached_contents).unwrap();
                        let config_file = Config { sort_mode: encode_sort(self.sorttype.clone()), show_hidden: self.show_hidden, bookmarks: self.bookmarked_dirs.clone(), icntheme: self.icntheme.clone(), icnsize: self.icnsize };
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
                match &self.typemode {
                    Some(val) => {
                        match self.last_clicked_file {
                            Some(..) => {
                                if val.len() >= 1 {
                                    self.rename()
                                }
                                self.typemode = None
                            }
                            None => {
                                self.typemode = None
                            }
                        }
                        use cosmic_time::button;
                        let unmitosis = chain![RENAMEBTN, 
                            button(Duration::ZERO).width(Length::Fixed(75.0)).height(Length::Fixed(TOP_HEIGHT as f32)),
                            button(Duration::from_millis(500)).width(Length::Fixed(75.0)).height(Length::Fixed(TOP_HEIGHT as f32)),
                            button(Duration::from_millis(500)).width(Length::Fixed(1000.0)).height(Length::Fixed(TOP_HEIGHT as f32)),
                        ];
                        self.anims.set_chain(unmitosis).start();
                        iced::Command::none()
                    },
                    None => {
                        use cosmic_time::button;
                        let mitosis = chain![RENAMEBTN, 
                            button(Duration::ZERO).width(Length::Fixed(1000.0)).height(Length::Fixed(TOP_HEIGHT as f32)),
                            button(Duration::from_millis(500)).width(Length::Fixed(1000.0)).height(Length::Fixed(TOP_HEIGHT as f32)),
                            button(Duration::from_millis(500)).width(Length::Fixed(75.0)).height(Length::Fixed(TOP_HEIGHT as f32)),
                        ];
                        self.anims.set_chain(mitosis).start();
                        self.typemode = Some(String::default());
                        text_input::focus(self.rename_id.clone())
                    }
                }
            }
            Message::RenameUpdate(x) => {
                self.typemode = Some(x);
                iced::Command::none()
            }
            Message::Tick(now) => {
                self.anims.now(now);
                iced::Command::none()
            }
            Message::NoOp => {
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
        let back_btn = string_button(translated[0].clone(), SPECIAL_FONT_SIZE).on_press(Message::GoBack(1)).height(TOP_HEIGHT).style(current_theme.secondary.mk_theme());
        let sort_btn = string_button(translated[1].clone(), SPECIAL_FONT_SIZE).on_press(Message::SortChanged).height(TOP_HEIGHT).style(current_theme.secondary.mk_theme());
        let delete_btn = if self.deletion_confirmation {
            string_button(translated[2].clone(), SPECIAL_FONT_SIZE).on_press(Message::DeleteClicked).height(TOP_HEIGHT).style(theme::Button::Destructive)
        } else {
            string_button(translated[2].clone(), SPECIAL_FONT_SIZE).on_press(Message::DeleteClicked).height(TOP_HEIGHT).style(current_theme.secondary.mk_theme())
        };
        let mv_btn = match self.mv_target {
            Some(..) => string_button(translated[3].clone(), SPECIAL_FONT_SIZE).on_press(Message::MvClicked),
            None => string_button(translated[4].clone(), SPECIAL_FONT_SIZE).on_press(Message::MvClicked).height(TOP_HEIGHT).style(current_theme.secondary.mk_theme())
        };
        let cp_btn = match self.cp_target {
            Some(..) => string_button(translated[5].clone(), SPECIAL_FONT_SIZE).on_press(Message::CpClicked),
            None => string_button(translated[6].clone(), SPECIAL_FONT_SIZE).on_press(Message::CpClicked).height(TOP_HEIGHT).style(current_theme.secondary.mk_theme())
        };
        let hidden_btn = string_button(translated[7].clone(), SPECIAL_FONT_SIZE).height(TOP_HEIGHT).on_press(Message::HiddenChanged).style(current_theme.secondary.mk_theme());
        let bookmark_btn = string_button(translated[8].clone(), SPECIAL_FONT_SIZE).height(TOP_HEIGHT).on_press(Message::BookmarkCurrent).style(current_theme.secondary.mk_theme());
        let touch_btn = string_button(translated[9].clone(), SPECIAL_FONT_SIZE).width(SIDEBAR_WIDTH).on_press(Message::MkFile).style(current_theme.sidebar.mk_theme());
        let mkdir_btn = string_button(translated[10].clone(), SPECIAL_FONT_SIZE).width(SIDEBAR_WIDTH).on_press(Message::MkDir).style(current_theme.sidebar.mk_theme());
        //let function_cap = Button::new("").width(5000).height(TOP_HEIGHT).style(current_theme.secondary.mk_theme());
        let rename_btn = anim!(RENAMEBTN, &self.anims, Text::new(translated[11].clone()).size(SPECIAL_FONT_SIZE)).height(Length::Fixed(TOP_HEIGHT as f32)).on_press(Message::RenameToggle).style(current_theme.secondary.mk_theme());
        let mut function_buttons = Row::new().push(back_btn).push(sort_btn).push(hidden_btn).push(bookmark_btn).push(delete_btn).push(mv_btn).push(cp_btn).push(rename_btn);
        let txt = match &self.typemode {
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
        let mut temprow = Row::new();
        let mut filebtnfutures = vec![];
        for i in 0..self.uifiles.len() {
            filebtnfutures.push(self.uifiles[i].render());
        }
        let mut test = block_on(join_all(filebtnfutures));
        for i in 0..test.len() {
            let full = test.remove(0);
            if i % self.desired_cols as usize == 0 {
                file_listing = file_listing.push(temprow);
                temprow = Row::new().spacing(SPACING);
            }
            temprow = temprow.push(full);
        }
        file_listing = file_listing.push(temprow);
        let mut pathbar = Row::new();
        let chars: Vec<char> = self.currentpath.to_string_lossy().to_string().chars().collect();
        let mut pathentries = vec![];
        let pathcap = Button::new("").height(TOP_HEIGHT).width(10000).style(current_theme.sidebar.mk_theme()).on_press(Message::NoOp);
        let mut entries = 0;
        for character in chars {
            if character == '/' {
                pathentries.push(String::default());
                entries = pathentries.len() - 1;
            } else {
                pathentries[entries] = format!("{}{}", pathentries[entries], character);
            }
        }
        let mut iterations = 0;
        for entry in pathentries {
            pathbar = pathbar.push(Button::new(Text::new("/")).on_press(Message::NoOp).style(current_theme.sidebar.mk_theme()).height(TOP_HEIGHT));
            pathbar = pathbar.push(Button::new(Text::new(entry)).on_press(Message::GoBack(entries - iterations)).style(current_theme.sidebar.mk_theme()).height(TOP_HEIGHT));
            iterations = iterations + 1;
        }
        pathbar = pathbar.push(pathcap);
        //return render commands
        let ruleh = Rule::horizontal(RULE_WIDTH);
        let ruleh2 = Rule::horizontal(RULE_WIDTH);
        let fillspace = Space::new(10, Length::Fill);
        let rulev = Rule::vertical(RULE_WIDTH);
        let col_test = Column::new().push(function_buttons).push(ruleh).push(file_listing).push(fillspace).push(ruleh2).push(pathbar);
        let row_test = Row::new().push(bookmark_buttons).push(rulev).push(col_test);
        Container::new(row_test).width(Length::Fill).height(Length::Fill).into()
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
    fn theme(&self) -> Self::Theme {//send in the selected application theme
        match self.theme {
            ThemeType::Light => mk_app_theme(self.themes.light.application.clone()),
            ThemeType::Dark => mk_app_theme(self.themes.dark.application.clone()),
            ThemeType::Custom => mk_app_theme(self.themes.custom.application.clone()),
        }
    }
}