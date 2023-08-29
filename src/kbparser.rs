use std::path::PathBuf;

use cosmic_time::{chain, Duration};
use iced::{futures::executor::block_on, widget::text_input, Length};

use crate::{Narwhal, confighelpers::BookmarkDir, sort_file_by_type, RENAMEBTN, TOP_HEIGHT};

impl Narwhal {
    pub fn kbparse(&mut self, kb_event: iced::keyboard::Event) -> iced::Command<<Narwhal as iced::Application>::Message> {
        let mut return_command = iced::Command::none();
        match kb_event {
            iced::keyboard::Event::KeyPressed { key_code, modifiers } => {
                match &self.typemode {
                    Some(val) => {
                        if key_code == iced::keyboard::KeyCode::Escape {
                            self.typemode = None;
                            use cosmic_time::button;
                            let unmitosis = chain![RENAMEBTN, 
                                button(Duration::ZERO).width(Length::Fixed(75.0)).height(Length::Fixed(TOP_HEIGHT as f32)),
                                button(Duration::from_millis(500)).width(Length::Fixed(75.0)).height(Length::Fixed(TOP_HEIGHT as f32)),
                                button(Duration::from_millis(500)).width(Length::Fixed(1000.0)).height(Length::Fixed(TOP_HEIGHT as f32)),
                            ];
                            self.anims.set_chain(unmitosis).start();
                        } else if key_code == iced::keyboard::KeyCode::Enter {
                            match  self.last_clicked_file {
                                Some(..) => {
                                    if val.len() >= 1 {
                                        self.rename()
                                    }
                                    self.typemode = None;
                                }
                                None => {
                                    self.typemode = None;
                                }
                            }
                            use cosmic_time::button;
                            let unmitosis = chain![RENAMEBTN, 
                                button(Duration::ZERO).width(Length::Fixed(75.0)).height(Length::Fixed(TOP_HEIGHT as f32)),
                                button(Duration::from_millis(500)).width(Length::Fixed(75.0)).height(Length::Fixed(TOP_HEIGHT as f32)),
                                button(Duration::from_millis(500)).width(Length::Fixed(1000.0)).height(Length::Fixed(TOP_HEIGHT as f32)),
                            ];
                            self.anims.set_chain(unmitosis).start();
                        }
                    },
                    None => {
                if key_code == iced::keyboard::KeyCode::Left {//move the cursor to the left, wrapping around if necessary
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
                    block_on(self.regen_uifiles());
                } else if key_code == iced::keyboard::KeyCode::Right {//move the cursor to the right, wrapping if necessary
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
                    block_on(self.regen_uifiles());
                } else if key_code == iced::keyboard::KeyCode::Down {//move the cursor down, wrap if necessary
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
                    block_on(self.regen_uifiles());
                } else if key_code == iced::keyboard::KeyCode::Up {//move cursor up, wrap if necessary
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
                    block_on(self.regen_uifiles());
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
                    block_on(self.regen_uifiles());
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
                } else if key_code == iced::keyboard::KeyCode::B && modifiers.shift() {//bookmark or unbookmark current dir
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
                } else if key_code == iced::keyboard::KeyCode::Key1 && self.bookmarked_dirs.len() > 0 {//activate bookmarkdir 1
                    self.currentpath = PathBuf::from(self.bookmarked_dirs[0].path.clone());
                    self.regen_files();
                    sort_file_by_type(&mut self.files, self.sorttype.clone());
                    self.last_clicked_file = None;
                    block_on(self.regen_uifiles());
                } else if key_code == iced::keyboard::KeyCode::Key2 && self.bookmarked_dirs.len() > 1 {//activate bookmarkdir 2
                    self.currentpath = PathBuf::from(self.bookmarked_dirs[1].path.clone());
                    self.regen_files();
                    sort_file_by_type(&mut self.files, self.sorttype.clone());
                    self.last_clicked_file = None;
                    block_on(self.regen_uifiles());
                } else if key_code == iced::keyboard::KeyCode::Key3 && self.bookmarked_dirs.len() > 2 {//activate bookmarkdir 3
                    self.currentpath = PathBuf::from(self.bookmarked_dirs[2].path.clone());
                    self.regen_files();
                    sort_file_by_type(&mut self.files, self.sorttype.clone());
                    self.last_clicked_file = None;
                    block_on(self.regen_uifiles());
                } else if key_code == iced::keyboard::KeyCode::Key4 && self.bookmarked_dirs.len() > 3 {//activate bookmarkdir 4
                    self.currentpath = PathBuf::from(self.bookmarked_dirs[3].path.clone());
                    self.regen_files();
                    sort_file_by_type(&mut self.files, self.sorttype.clone());
                    self.last_clicked_file = None;
                    block_on(self.regen_uifiles());
                } else if key_code == iced::keyboard::KeyCode::Key5 && self.bookmarked_dirs.len() > 4 {//activate bookmarkdir 5
                    self.currentpath = PathBuf::from(self.bookmarked_dirs[4].path.clone());
                    self.regen_files();
                    sort_file_by_type(&mut self.files, self.sorttype.clone());
                    self.last_clicked_file = None;
                    block_on(self.regen_uifiles());
                } else if key_code == iced::keyboard::KeyCode::Key6 && self.bookmarked_dirs.len() > 5 {//activate bookmarkdir 6
                    self.currentpath = PathBuf::from(self.bookmarked_dirs[5].path.clone());
                    self.regen_files();
                    sort_file_by_type(&mut self.files, self.sorttype.clone());
                    self.last_clicked_file = None;
                    block_on(self.regen_uifiles());
                } else if key_code == iced::keyboard::KeyCode::Key7 && self.bookmarked_dirs.len() > 6 {//activate bookmarkdir 7
                    self.currentpath = PathBuf::from(self.bookmarked_dirs[6].path.clone());
                    self.regen_files();
                    sort_file_by_type(&mut self.files, self.sorttype.clone());
                    self.last_clicked_file = None;
                    block_on(self.regen_uifiles());
                } else if key_code == iced::keyboard::KeyCode::Key8 && self.bookmarked_dirs.len() > 7 {//activate bookmarkdir 8
                    self.currentpath = PathBuf::from(self.bookmarked_dirs[7].path.clone());
                    self.regen_files();
                    sort_file_by_type(&mut self.files, self.sorttype.clone());
                    self.last_clicked_file = None;
                    block_on(self.regen_uifiles());
                } else if key_code == iced::keyboard::KeyCode::Key9 && self.bookmarked_dirs.len() > 8 {//activate bookmarkdir 9
                    self.currentpath = PathBuf::from(self.bookmarked_dirs[8].path.clone());
                    self.regen_files();
                    sort_file_by_type(&mut self.files, self.sorttype.clone());
                    self.last_clicked_file = None;
                    block_on(self.regen_uifiles());
                } else if key_code == iced::keyboard::KeyCode::Key0 && self.bookmarked_dirs.len() > 9 {//activate bookmarkdir 10
                    self.currentpath = PathBuf::from(self.bookmarked_dirs[9].path.clone());
                    self.regen_files();
                    sort_file_by_type(&mut self.files, self.sorttype.clone());
                    self.last_clicked_file = None;
                    block_on(self.regen_uifiles());
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
                    self.typemode = Some(String::default());
                    use cosmic_time::button;
                    let mitosis = chain![RENAMEBTN, 
                        button(Duration::ZERO).width(Length::Fixed(1000.0)).height(Length::Fixed(TOP_HEIGHT as f32)),
                        button(Duration::from_millis(500)).width(Length::Fixed(1000.0)).height(Length::Fixed(TOP_HEIGHT as f32)),
                        button(Duration::from_millis(500)).width(Length::Fixed(75.0)).height(Length::Fixed(TOP_HEIGHT as f32)),
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