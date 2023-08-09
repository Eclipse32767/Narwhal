use std::path::PathBuf;

use iced::futures::executor::block_on;

use crate::{Narwhal, confighelpers::BookmarkDir, sort_file_by_type};

impl Narwhal {
    pub fn kbparse(&mut self, kb_event: iced::keyboard::Event) {
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
                    block_on(self.regen_uifiles());
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
                    block_on(self.regen_uifiles());
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
                    block_on(self.regen_uifiles());
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
                    block_on(self.regen_uifiles());
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
                    block_on(self.regen_uifiles());
                }
                if key_code == iced::keyboard::KeyCode::Minus && modifiers == iced::keyboard::Modifiers::SHIFT {
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
                if key_code == iced::keyboard::KeyCode::B && modifiers.shift() {
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
                }
                if key_code == iced::keyboard::KeyCode::Key1 && self.bookmarked_dirs.len() > 0 {
                    self.currentpath = PathBuf::from(self.bookmarked_dirs[0].path.clone());
                    self.regen_files();
                    sort_file_by_type(&mut self.files, self.sorttype.clone());
                    self.last_clicked_file = None;
                    block_on(self.regen_uifiles());
                }
                if key_code == iced::keyboard::KeyCode::Key2 && self.bookmarked_dirs.len() > 1 {
                    self.currentpath = PathBuf::from(self.bookmarked_dirs[1].path.clone());
                    self.regen_files();
                    sort_file_by_type(&mut self.files, self.sorttype.clone());
                    self.last_clicked_file = None;
                    block_on(self.regen_uifiles());
                }
                if key_code == iced::keyboard::KeyCode::Key3 && self.bookmarked_dirs.len() > 2 {
                    self.currentpath = PathBuf::from(self.bookmarked_dirs[2].path.clone());
                    self.regen_files();
                    sort_file_by_type(&mut self.files, self.sorttype.clone());
                    self.last_clicked_file = None;
                    block_on(self.regen_uifiles());
                }
                if key_code == iced::keyboard::KeyCode::Key4 && self.bookmarked_dirs.len() > 3 {
                    self.currentpath = PathBuf::from(self.bookmarked_dirs[3].path.clone());
                    self.regen_files();
                    sort_file_by_type(&mut self.files, self.sorttype.clone());
                    self.last_clicked_file = None;
                    block_on(self.regen_uifiles());
                }
                if key_code == iced::keyboard::KeyCode::Key5 && self.bookmarked_dirs.len() > 4 {
                    self.currentpath = PathBuf::from(self.bookmarked_dirs[4].path.clone());
                    self.regen_files();
                    sort_file_by_type(&mut self.files, self.sorttype.clone());
                    self.last_clicked_file = None;
                    block_on(self.regen_uifiles());
                }
                if key_code == iced::keyboard::KeyCode::Key6 && self.bookmarked_dirs.len() > 5 {
                    self.currentpath = PathBuf::from(self.bookmarked_dirs[5].path.clone());
                    self.regen_files();
                    sort_file_by_type(&mut self.files, self.sorttype.clone());
                    self.last_clicked_file = None;
                    block_on(self.regen_uifiles());
                }
                if key_code == iced::keyboard::KeyCode::Key7 && self.bookmarked_dirs.len() > 6 {
                    self.currentpath = PathBuf::from(self.bookmarked_dirs[6].path.clone());
                    self.regen_files();
                    sort_file_by_type(&mut self.files, self.sorttype.clone());
                    self.last_clicked_file = None;
                    block_on(self.regen_uifiles());
                }
                if key_code == iced::keyboard::KeyCode::Key8 && self.bookmarked_dirs.len() > 7 {
                    self.currentpath = PathBuf::from(self.bookmarked_dirs[7].path.clone());
                    self.regen_files();
                    sort_file_by_type(&mut self.files, self.sorttype.clone());
                    self.last_clicked_file = None;
                    block_on(self.regen_uifiles());
                }
                if key_code == iced::keyboard::KeyCode::Key9 && self.bookmarked_dirs.len() > 8 {
                    self.currentpath = PathBuf::from(self.bookmarked_dirs[8].path.clone());
                    self.regen_files();
                    sort_file_by_type(&mut self.files, self.sorttype.clone());
                    self.last_clicked_file = None;
                    block_on(self.regen_uifiles());
                }
                if key_code == iced::keyboard::KeyCode::Key0 && self.bookmarked_dirs.len() > 9 {
                    self.currentpath = PathBuf::from(self.bookmarked_dirs[9].path.clone());
                    self.regen_files();
                    sort_file_by_type(&mut self.files, self.sorttype.clone());
                    self.last_clicked_file = None;
                    block_on(self.regen_uifiles());
                }
                if key_code == iced::keyboard::KeyCode::M {//mv
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
                }
                if key_code == iced::keyboard::KeyCode::C {//cp
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
                }
            }
            iced::keyboard::Event::KeyReleased { key_code: _, modifiers: _ } => {},
            iced::keyboard::Event::CharacterReceived(_) => {},
            iced::keyboard::Event::ModifiersChanged(_) => {},
        }
    }
}