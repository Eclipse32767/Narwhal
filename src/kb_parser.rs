use std::path::PathBuf;

use cosmic_time::{chain, Duration};
use iced::{futures::executor::block_on, widget::text_input, Length};

use crate::{Narwhal, config_helpers::BookmarkDir, sort_file_by_type, RENAME_BTN, TOP_HEIGHT};

impl Narwhal {
    pub fn kb_parse(&mut self, kb_event: iced::keyboard::Event) -> iced::Command<<Narwhal as iced::Application>::Message> {
        let mut return_command = iced::Command::none();
        match kb_event {
            iced::keyboard::Event::KeyPressed { key_code, modifiers } => {
                match &self.type_mode {
                    Some(val) => {
                        if key_code == iced::keyboard::KeyCode::Escape {
                            self.type_mode = None;
                            use cosmic_time::button;
                            let un_mitosis = chain![RENAME_BTN,
                                button(Duration::ZERO).width(Length::Fixed(0.0)).height(Length::Fixed(TOP_HEIGHT as f32)),
                                button(Duration::from_millis(500)).width(Length::Fixed(0.0)).height(Length::Fixed(TOP_HEIGHT as f32)),
                                button(Duration::from_millis(500)).width(Length::Fixed(1000.0)).height(Length::Fixed(TOP_HEIGHT as f32)),
                            ];
                            self.anims.set_chain(un_mitosis).start();
                        } else if key_code == iced::keyboard::KeyCode::Enter {
                            match  self.last_clicked_file {
                                Some(..) => {
                                    if val.len() >= 1 {
                                        self.rename()
                                    }
                                    self.type_mode = None;
                                }
                                None => {
                                    self.type_mode = None;
                                }
                            }
                            use cosmic_time::button;
                            let un_mitosis = chain![RENAME_BTN,
                                button(Duration::ZERO).width(Length::Fixed(0.0)).height(Length::Fixed(TOP_HEIGHT as f32)),
                                button(Duration::from_millis(500)).width(Length::Fixed(0.0)).height(Length::Fixed(TOP_HEIGHT as f32)),
                                button(Duration::from_millis(500)).width(Length::Fixed(1000.0)).height(Length::Fixed(TOP_HEIGHT as f32)),
                            ];
                            self.anims.set_chain(un_mitosis).start();
                        }
                    },
                    None => {
                if key_code == iced::keyboard::KeyCode::Left {//move the cursor to the left, wrapping around if necessary
                    let mut old_index = match self.ui_files.len() {
                        0 => 0,
                        _ => self.ui_files.len() - 1
                    };
                    for i in 0..self.ui_files.len() {
                        match self.last_clicked_file {
                            Some(x) => {
                                if self.ui_files[i].original_index == x && i != 0{
                                    old_index = i - 1;
                                    break;
                                }
                            },
                            None => {},
                        }
                    }
                    if self.ui_files.len() > 0 {
                        let index = self.ui_files[old_index].original_index;
                        self.last_clicked_file = Some(index);
                    }
                    block_on(self.regen_ui_files());
                } else if key_code == iced::keyboard::KeyCode::Right {//move the cursor to the right, wrapping if necessary
                    let mut old_index = 0;
                    for i in 0..self.ui_files.len() {
                        match self.last_clicked_file {
                            Some(x) => {
                                if self.ui_files[i].original_index == x && i != self.ui_files.len() - 1 {
                                    old_index = i + 1;
                                    break;
                                }
                            },
                            None => {},
                        }
                    }
                    if self.ui_files.len()>0 {
                    let index = self.ui_files[old_index].original_index;
                    self.last_clicked_file = Some(index);
                    }
                    block_on(self.regen_ui_files());
                } else if key_code == iced::keyboard::KeyCode::Down {//move the cursor down, wrap if necessary
                    let mut old_index = None;
                    for i in 0..self.ui_files.len() {
                        match self.last_clicked_file {
                            Some(x) => {
                                if self.ui_files[i].original_index == x {
                                    old_index = Some(i);
                                    break;
                                }
                            },
                            None => {},
                        }
                    }
                    old_index = match old_index {
                        Some(x) => {
                            if self.desired_cols as usize + x < self.ui_files.len() {
                                Some(x + self.desired_cols as usize)
                            } else {
                                Some(x % self.desired_cols as usize)
                            }
                        }
                        None => {
                            Some(0)
                        }
                    };
                    if self.ui_files.len()>0 {
                        let index = self.ui_files[old_index.unwrap()].original_index;
                        self.last_clicked_file = Some(index);
                    }
                    block_on(self.regen_ui_files());
                } else if key_code == iced::keyboard::KeyCode::Up {//move cursor up, wrap if necessary
                    let mut old_index = None;
                    for i in 0..self.ui_files.len() {
                        match self.last_clicked_file {
                            Some(x) => {
                                if self.ui_files[i].original_index == x {
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
                                let length = (self.ui_files.len() - 1) as u32;
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
                    if self.ui_files.len() > 0 {
                        let index = self.ui_files[old_index.unwrap()].original_index;
                        self.last_clicked_file = Some(index);
                    }
                    block_on(self.regen_ui_files());
                } else if key_code == iced::keyboard::KeyCode::Enter {//interact with hovered entry
                    match self.last_clicked_file {
                        Some(x) => self.interact_selected_entry(x),
                        None => {}
                    }
                } else if key_code == iced::keyboard::KeyCode::Backspace {//equivalent to cd ..
                    self.go_back_directory();
                } else if key_code == iced::keyboard::KeyCode::S && modifiers == iced::keyboard::Modifiers::SHIFT {//cycle sort mode forwards
                    self.change_sort(true);
                } else if key_code == iced::keyboard::KeyCode::S {//cycle sort mode backwards
                    self.change_sort(false);
                } else if key_code == iced::keyboard::KeyCode::H {//toggle hidden files
                    self.show_hidden = !self.show_hidden;
                    block_on(self.regen_ui_files());
                } else if key_code == iced::keyboard::KeyCode::Minus && modifiers == iced::keyboard::Modifiers::SHIFT {//delete files
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
                } else if key_code == iced::keyboard::KeyCode::B && modifiers.shift() {//bookmark or un-bookmark current dir
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
                } else if key_code == iced::keyboard::KeyCode::Key1 && self.bookmarked_dirs.len() > 0 {//activate bookmark dir 1
                    self.current_path = PathBuf::from(self.bookmarked_dirs[0].path.clone());
                    self.regen_files();
                    sort_file_by_type(&mut self.files, self.sort_type.clone());
                    self.last_clicked_file = None;
                    block_on(self.regen_ui_files());
                } else if key_code == iced::keyboard::KeyCode::Key2 && self.bookmarked_dirs.len() > 1 {//activate bookmark dir 2
                    self.current_path = PathBuf::from(self.bookmarked_dirs[1].path.clone());
                    self.regen_files();
                    sort_file_by_type(&mut self.files, self.sort_type.clone());
                    self.last_clicked_file = None;
                    block_on(self.regen_ui_files());
                } else if key_code == iced::keyboard::KeyCode::Key3 && self.bookmarked_dirs.len() > 2 {//activate bookmark dir 3
                    self.current_path = PathBuf::from(self.bookmarked_dirs[2].path.clone());
                    self.regen_files();
                    sort_file_by_type(&mut self.files, self.sort_type.clone());
                    self.last_clicked_file = None;
                    block_on(self.regen_ui_files());
                } else if key_code == iced::keyboard::KeyCode::Key4 && self.bookmarked_dirs.len() > 3 {//activate bookmark dir 4
                    self.current_path = PathBuf::from(self.bookmarked_dirs[3].path.clone());
                    self.regen_files();
                    sort_file_by_type(&mut self.files, self.sort_type.clone());
                    self.last_clicked_file = None;
                    block_on(self.regen_ui_files());
                } else if key_code == iced::keyboard::KeyCode::Key5 && self.bookmarked_dirs.len() > 4 {//activate bookmark dir 5
                    self.current_path = PathBuf::from(self.bookmarked_dirs[4].path.clone());
                    self.regen_files();
                    sort_file_by_type(&mut self.files, self.sort_type.clone());
                    self.last_clicked_file = None;
                    block_on(self.regen_ui_files());
                } else if key_code == iced::keyboard::KeyCode::Key6 && self.bookmarked_dirs.len() > 5 {//activate bookmark dir 6
                    self.current_path = PathBuf::from(self.bookmarked_dirs[5].path.clone());
                    self.regen_files();
                    sort_file_by_type(&mut self.files, self.sort_type.clone());
                    self.last_clicked_file = None;
                    block_on(self.regen_ui_files());
                } else if key_code == iced::keyboard::KeyCode::Key7 && self.bookmarked_dirs.len() > 6 {//activate bookmark dir 7
                    self.current_path = PathBuf::from(self.bookmarked_dirs[6].path.clone());
                    self.regen_files();
                    sort_file_by_type(&mut self.files, self.sort_type.clone());
                    self.last_clicked_file = None;
                    block_on(self.regen_ui_files());
                } else if key_code == iced::keyboard::KeyCode::Key8 && self.bookmarked_dirs.len() > 7 {//activate bookmark dir 8
                    self.current_path = PathBuf::from(self.bookmarked_dirs[7].path.clone());
                    self.regen_files();
                    sort_file_by_type(&mut self.files, self.sort_type.clone());
                    self.last_clicked_file = None;
                    block_on(self.regen_ui_files());
                } else if key_code == iced::keyboard::KeyCode::Key9 && self.bookmarked_dirs.len() > 8 {//activate bookmark dir 9
                    self.current_path = PathBuf::from(self.bookmarked_dirs[8].path.clone());
                    self.regen_files();
                    sort_file_by_type(&mut self.files, self.sort_type.clone());
                    self.last_clicked_file = None;
                    block_on(self.regen_ui_files());
                } else if key_code == iced::keyboard::KeyCode::Key0 && self.bookmarked_dirs.len() > 9 {//activate bookmark dir 10
                    self.current_path = PathBuf::from(self.bookmarked_dirs[9].path.clone());
                    self.regen_files();
                    sort_file_by_type(&mut self.files, self.sort_type.clone());
                    self.last_clicked_file = None;
                    block_on(self.regen_ui_files());
                } else if key_code == iced::keyboard::KeyCode::M {//move files around
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
                } else if key_code == iced::keyboard::KeyCode::C {//copy files
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
                } else if key_code == iced::keyboard::KeyCode::N && modifiers.shift() {//mkdir
                    self.mkdir();
                } else if key_code == iced::keyboard::KeyCode::N {//touch
                    self.touch();
                } else if key_code == iced::keyboard::KeyCode::R {//enter rename mode
                    self.type_mode = Some(String::default());
                    use cosmic_time::button;
                    let mitosis = chain![RENAME_BTN,
                        button(Duration::ZERO).width(Length::Fixed(1000.0)).height(Length::Fixed(TOP_HEIGHT as f32)),
                        button(Duration::from_millis(500)).width(Length::Fixed(1000.0)).height(Length::Fixed(TOP_HEIGHT as f32)),
                        button(Duration::from_millis(500)).width(Length::Fixed(0.0)).height(Length::Fixed(TOP_HEIGHT as f32)),
                    ];
                    self.anims.set_chain(mitosis).start();
                    return_command = text_input::focus(self.rename_id.clone())
                }
            }
            }
            }
            iced::keyboard::Event::KeyReleased { key_code: _, modifiers: _ } => {},
            iced::keyboard::Event::CharacterReceived(_) => {},
            iced::keyboard::Event::ModifiersChanged(modifiers) => {
                self.show_keybinds = modifiers.control();
            },
        }
        return_command
    }
}