use crate::descriptor::CommandId;
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    pub timestamp_ms: u128,
    pub command_id: CommandId,
}

pub struct CrashJournal {
    file_path: Option<PathBuf>,
    in_memory: Vec<JournalEntry>,
}

impl Default for CrashJournal {
    fn default() -> Self {
        Self::new_in_memory()
    }
}

impl CrashJournal {
    pub fn new_in_memory() -> Self {
        Self {
            file_path: None,
            in_memory: Vec::new(),
        }
    }

    pub fn new_with_file(path: impl Into<PathBuf>) -> Self {
        Self {
            file_path: Some(path.into()),
            in_memory: Vec::new(),
        }
    }

    pub fn record(&mut self, command_id: &CommandId) {
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);

        let entry = JournalEntry {
            timestamp_ms,
            command_id: command_id.clone(),
        };

        if let Some(path) = &self.file_path {
            if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
                if let Ok(line) = ron::to_string(&entry) {
                    let _ = writeln!(file, "{}", line);
                }
            }
        }

        self.in_memory.push(entry);
    }

    pub fn entries(&self) -> &[JournalEntry] {
        &self.in_memory
    }

    pub fn clear(&mut self) {
        self.in_memory.clear();
        if let Some(path) = &self.file_path {
            let _ = std::fs::remove_file(path);
        }
    }
}
