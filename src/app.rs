use crate::fs::DirEntry;
use std::path::PathBuf;
use ratatui::widgets::ListState;

pub struct App {
    pub current_path: PathBuf,
    pub entries: Vec<DirEntry>,
    pub list_state: ListState,
}

impl App {
    pub fn new(path: &str) -> App {
        let entries = crate::fs::scan_dir(path);
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        App {
            current_path: PathBuf::from(path),
            entries,
            list_state,
        }
    }

    pub fn navigate_into(&mut self) {
        if let Some(i) = self.list_state.selected() {
            let selected = &self.entries[i];
            if selected.path.is_dir() {
                self.current_path = selected.path.clone();
                self.entries = crate::fs::scan_dir(self.current_path.to_str().unwrap());
                self.list_state.select(Some(0));
            }
        }
    }

    pub fn navigate_back(&mut self) {
        if let Some(parent) = self.current_path.parent() {
            self.current_path = parent.to_path_buf();
            self.entries = crate::fs::scan_dir(self.current_path.to_str().unwrap());
            self.list_state.select(Some(0));
        }
    }

    pub fn move_up(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => if i == 0 { self.entries.len() - 1 } else { i - 1 },
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn move_down(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => if i == self.entries.len() - 1 { 0 } else { i + 1 },
            None => 0,
        };
        self.list_state.select(Some(i));
    }
}