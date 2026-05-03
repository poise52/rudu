use crate::fs::{build_index_async, list_dir, DirEntry, SizeIndex};
use crossbeam_channel::{unbounded, Receiver};
use ratatui::widgets::ListState;
use std::fs;
use std::path::PathBuf;

#[derive(PartialEq)]
pub enum ScanState {
    Scanning,
    Done,
}

pub struct App {
    pub current_path: PathBuf,
    pub root_path: PathBuf,
    pub entries: Vec<DirEntry>,
    pub list_state: ListState,
    pub scan_state: ScanState,
    pub spinner_tick: u8,
    index: Option<SizeIndex>,
    rx: Receiver<SizeIndex>,
}

impl App {
    pub fn new(path: &str) -> App {
        let root_path = fs::canonicalize(path).unwrap_or_else(|_| PathBuf::from(path));
        let (tx, rx) = unbounded();
        build_index_async(root_path.clone(), tx);

        let mut list_state = ListState::default();
        list_state.select(Some(0));

        App {
            current_path: root_path.clone(),
            root_path,
            entries: vec![],
            list_state,
            scan_state: ScanState::Scanning,
            spinner_tick: 0,
            index: None,
            rx,
        }
    }

    pub fn tick(&mut self) {
        if let Ok(index) = self.rx.try_recv() {
            self.index = Some(index);
            self.scan_state = ScanState::Done;
            self.reload_entries();
            if !self.entries.is_empty() {
                self.list_state.select(Some(0));
            }
        }
        self.spinner_tick = self.spinner_tick.wrapping_add(1);
    }

    fn reload_entries(&mut self) {
        if let Some(ref idx) = self.index {
            self.entries = list_dir(idx.as_ref(), &self.current_path);
        }
    }

    pub fn refresh_scan(&mut self) {
        let (tx, rx) = unbounded();
        self.rx = rx;
        self.scan_state = ScanState::Scanning;
        self.index = None;
        self.entries.clear();
        build_index_async(self.root_path.clone(), tx);
        self.list_state.select(Some(0));
    }

    pub fn navigate_into(&mut self) {
        if matches!(self.scan_state, ScanState::Scanning) {
            return;
        }
        if self.index.is_none() {
            return;
        }
        if let Some(i) = self.list_state.selected() {
            if i < self.entries.len() && self.entries[i].is_dir {
                self.current_path = self.entries[i].path.clone();
                self.reload_entries();
                self.list_state.select(Some(0));
            }
        }
    }

    pub fn navigate_back(&mut self) {
        if matches!(self.scan_state, ScanState::Scanning) {
            return;
        }
        if self.index.is_none() {
            return;
        }
        if self.current_path == self.root_path {
            return;
        }
        if let Some(parent) = self.current_path.parent() {
            self.current_path = parent.to_path_buf();
            self.reload_entries();
            self.list_state.select(Some(0));
        }
    }

    pub fn move_up(&mut self) {
        if self.entries.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.entries.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn move_down(&mut self) {
        if self.entries.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.entries.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }
}
