use crate::fs::{DirEntry, scan_dir_async};
use std::path::PathBuf;
use ratatui::widgets::ListState;
use crossbeam_channel::{unbounded, Receiver};

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
    rx: Receiver<Vec<DirEntry>>,
}

impl App {
    pub fn new(path: &str) -> App {
        let (tx, rx) = unbounded();
        scan_dir_async(PathBuf::from(path), tx);

        let mut list_state = ListState::default();
        list_state.select(Some(0));

        App {
            current_path: PathBuf::from(path),
            root_path: PathBuf::from(path),
            entries: vec![],
            list_state,
            scan_state: ScanState::Scanning,
            spinner_tick: 0,
            rx,
        }
    }

    pub fn tick(&mut self) {
        if let Ok(entries) = self.rx.try_recv() {
            self.entries = entries;
            self.scan_state = ScanState::Done;
            if !self.entries.is_empty() {
                self.list_state.select(Some(0));
            }
        }
        self.spinner_tick = self.spinner_tick.wrapping_add(1);
    }

    pub fn navigate_into(&mut self) {
        if matches!(self.scan_state, ScanState::Scanning) {
            return;
        }
        if let Some(i) = self.list_state.selected() {
            if i < self.entries.len() && self.entries[i].is_dir {
                let path = self.entries[i].path.clone();
                let (tx, rx) = unbounded();
                self.rx = rx;
                self.scan_state = ScanState::Scanning;
                self.current_path = path.clone();
                self.entries = vec![];
                scan_dir_async(path, tx);
                self.list_state.select(Some(0));
            }
        }
    }

    pub fn navigate_back(&mut self) {
        if self.current_path == self.root_path {
            return;
        }
        if let Some(parent) = self.current_path.parent() {
            let parent_path = parent.to_path_buf();
            let (tx, rx) = unbounded();
            self.rx = rx;
            self.scan_state = ScanState::Scanning;
            self.entries = vec![];
            scan_dir_async(parent_path.clone(), tx);
            self.current_path = parent_path;
            self.list_state.select(Some(0));
        }
    }

    pub fn move_up(&mut self) {
        if self.entries.is_empty() { return; }
        let i = match self.list_state.selected() {
            Some(i) => if i == 0 { self.entries.len() - 1 } else { i - 1 },
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn move_down(&mut self) {
        if self.entries.is_empty() { return; }
        let i = match self.list_state.selected() {
            Some(i) => if i >= self.entries.len() - 1 { 0 } else { i + 1 },
            None => 0,
        };
        self.list_state.select(Some(i));
    }
}