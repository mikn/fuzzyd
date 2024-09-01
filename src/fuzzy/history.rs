use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

pub struct History {
    entries: HashMap<String, u32>,
    file_path: Option<PathBuf>,
}

impl History {
    pub fn new(file_path: Option<PathBuf>) -> Self {
        let entries = file_path.as_ref().map_or_else(HashMap::new, Self::load_from_file);
        History { entries, file_path }
    }

    fn load_from_file(file_path: &PathBuf) -> HashMap<String, u32> {
        let mut entries = HashMap::new();
        if let Ok(file) = File::open(file_path) {
            let reader = BufReader::new(file);
            for line in reader.lines().filter_map(Result::ok) {
                let parts: Vec<&str> = line.split('\t').collect();
                if parts.len() == 2 {
                    if let Ok(count) = parts[1].parse::<u32>() {
                        entries.insert(parts[0].to_string(), count);
                    }
                }
            }
        }
        entries
    }

    pub fn record_usage(&mut self, command: &str) {
        if self.file_path.is_some() {
            let count = self.entries.entry(command.to_string()).or_insert(0);
            *count += 1;
            self.save_to_file();
        }
    }

    pub fn get_count(&self, command: &str) -> u32 {
        if self.file_path.is_some() {
            *self.entries.get(command).unwrap_or(&0)
        } else {
            0
        }
    }

    fn save_to_file(&self) {
        if let Some(file_path) = &self.file_path {
            if let Ok(mut file) = OpenOptions::new().write(true).truncate(true).create(true).open(file_path) {
                for (command, count) in &self.entries {
                    writeln!(file, "{}\t{}", command, count).ok();
                }
            }
        }
    }
}