use std::{fs, path::PathBuf};

use crate::error::{AppError, AppResult};

pub struct FileNavigator {
    pub entries: Vec<PathBuf>,
    pub selected_index: usize,
}

impl FileNavigator {
    pub fn new(path: &str) -> AppResult<Self> {
        let entries_iter =
            fs::read_dir(path).map_err(|_| AppError::DirReadError(path.to_owned()))?;
        let mut entries: Vec<PathBuf> = entries_iter
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .collect();
        entries.sort();
        Ok(Self {
            entries,
            selected_index: 0,
        })
    }

    pub fn next(&mut self) {
        if self.selected_index < self.entries.len().saturating_sub(1) {
            self.selected_index += 1;
        }
    }

    pub fn previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn selected(&self) -> Option<&PathBuf> {
        self.entries.get(self.selected_index)
    }
}
