use std::env;
use which::which;
use crate::fuzzy::FuzzyItem;
use crate::sources::SourceFinder;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::fs;
use std::os::unix::fs::PermissionsExt;

pub struct PathFinder;

impl SourceFinder for PathFinder {
    fn find_entries(&self) -> Vec<FuzzyItem> {
        let icon = "  ".to_string(); // Add this line
        env::var_os("PATH")
            .map(|paths| {
                // Step 1: Deduplicate PATH directories
                let unique_dirs: HashSet<_> = env::split_paths(&paths)
                    .filter_map(|path| fs::canonicalize(path).ok())
                    .collect();
                
                // Step 2: Collect and deduplicate executables from unique directories
                let executables: HashMap<String, PathBuf> = unique_dirs.par_iter()
                    .flat_map(|dir| {
                        fs::read_dir(dir)
                            .into_iter()
                            .flatten()
                            .filter_map(|entry| {
                                let entry = entry.ok()?;
                                let path = entry.path();
                                if path.is_file() && which(&path).is_ok() {
                                    // Resolve the real path of the executable
                                    let real_path = fs::canonicalize(&path).ok()?;
                                    Some((path.file_name()?.to_str()?.to_string(), real_path))
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>()
                    })
                    .collect();

                // Step 3: Create FuzzyItems from unique executables
                executables.into_iter()
                    .filter_map(|(name, path)| {
                        if is_executable(path.to_str().unwrap_or("")) {
                            Some(FuzzyItem {
                                display: name,
                                exec: format!("\"{}\"", path.to_str().unwrap()),
                                priority: 1,
                                source_order: 1,
                                description: "Executable in PATH".to_string(),
                                source_path: path.to_str().unwrap().to_string(),
                                search_desc: false,
                                icon: icon.clone(), // Add this line
                            })
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    fn source_order(&self) -> usize {
        1 // PATH executables have lower priority
    }
}

fn is_executable(path: &str) -> bool {
    if let Ok(metadata) = fs::metadata(path) {
        let permissions = metadata.permissions();
        let mode = permissions.mode();
        // Check if the file is executable by the owner
        mode & 0o100 != 0
    } else {
        false
    }
}

impl PathFinder {
    pub fn new() -> Self {
        PathFinder
    }
}