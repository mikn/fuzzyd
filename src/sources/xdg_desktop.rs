use walkdir::WalkDir;
use xdg::BaseDirectories;
use std::fs::File;
use std::io::{BufRead, BufReader};
use regex::Regex;
use crate::fuzzy::FuzzyItem;
use lazy_static::lazy_static;
use crate::sources::SourceFinder;
use rayon::prelude::*;
use std::path::PathBuf;
use std::collections::HashMap;

pub struct XdgDesktopFinder;

impl SourceFinder for XdgDesktopFinder {
    fn find_entries(&self) -> Vec<FuzzyItem> {
        let icon = "  ".to_string();
        let xdg_dirs = BaseDirectories::new().expect("Failed to get XDG directories");
        let mut data_dirs: Vec<PathBuf> = xdg_dirs.get_data_dirs();

        // Add the user-specific directory
        data_dirs.push(xdg_dirs.get_data_home());

        data_dirs.par_iter()
            .flat_map(|dir| {
                let applications_dir = dir.join("applications");
                WalkDir::new(applications_dir)
                    .into_iter()
                    .par_bridge()
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().extension().map_or(false, |ext| ext == "desktop"))
                    .flat_map(|entry| parse_desktop_file(entry.path(), &icon))
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    fn source_order(&self) -> usize {
        0 // Desktop entries have higher priority
    }
}

fn parse_desktop_file(path: &std::path::Path, icon: &str) -> Vec<FuzzyItem> { // Add icon parameter
    let file = match File::open(path) {
        Ok(file) => file,
        Err(_) => return Vec::new(), // Return an empty vector if file can't be opened
    };
    let reader = BufReader::new(file);

    let mut items = Vec::new();
    let mut current_section = String::new();
    let mut main_item = HashMap::new();
    let mut action_items = HashMap::new();

    for line in reader.lines() {
        let line = match line {
            Ok(line) => line,
            Err(_) => continue, // Skip lines that can't be read
        };
        if line.starts_with('[') && line.ends_with(']') {
            current_section = line[1..line.len() - 1].to_string();
        } else if !line.trim().is_empty() {
            let parts: Vec<&str> = line.splitn(2, '=').collect();
            if parts.len() == 2 {
                let key = parts[0].trim();
                let value = parts[1].trim();
                if current_section == "Desktop Entry" {
                    main_item.insert(key.to_string(), value.to_string());
                } else if current_section.starts_with("Desktop Action") {
                    action_items
                        .entry(current_section.clone())
                        .or_insert_with(HashMap::new)
                        .insert(key.to_string(), value.to_string());
                }
            }
        }
    }

    // Create main FuzzyItem
    if let Some(name) = main_item.get("Name") {
        if let Some(exec) = main_item.get("Exec") {
            if let Some(item) = create_fuzzy_item(
                name,
                exec,
                main_item.get("Comment").unwrap_or(&String::new()),
                main_item.get("Icon").unwrap_or(&String::new()),
                path,
                "Desktop Entry",
                icon, // Pass icon
            ) {
                items.push(item);
            }
        }
    }

    // Create FuzzyItems for Desktop Actions
    for (section, action) in action_items {
        if let Some(name) = action.get("Name") {
            if let Some(exec) = action.get("Exec") {
                if let Some(item) = create_fuzzy_item(
                    name,
                    exec,
                    action.get("Comment").unwrap_or(&String::new()),
                    main_item.get("Icon").unwrap_or(&String::new()),
                    path,
                    &section,
                    icon, // Pass icon
                ) {
                    items.push(item);
                }
            }
        }
    }

    items
}

fn create_fuzzy_item(name: &str, exec: &str, description: &str, icon: &str, path: &std::path::Path, section: &str, source_icon: &str) -> Option<FuzzyItem> { // Add source_icon parameter
    let source_path = if section.starts_with("Desktop Action") {
        format!("{}:{}", path.to_str().unwrap_or(""), section)
    } else {
        path.to_str().unwrap_or("").to_string()
    };

    let (description, search_desc) = if description.is_empty() {
        ("No description".to_string(), false)
    } else {
        (description.to_string(), true)
    };

    let parsed_exec = parse_exec(exec, icon);

    Some(FuzzyItem {
        display: if section.starts_with("Desktop Action") {
            format!("{} ({})", name, section.strip_prefix("Desktop Action ").unwrap_or(section))
        } else {
            name.to_string()
        },
        exec: parsed_exec,
        priority: 2,
        source_order: 0,
        description,
        source_path,
        search_desc,
        icon: source_icon.to_string(), // Add this line
    })
}

pub fn parse_exec(exec: &str, icon: &str) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"%[fFuUdDnNvmck]").unwrap();
    }
    
    let parsed = RE.replace_all(exec, "");
    let parsed = parsed.replace("%i", &format!("--icon {}", icon));
    let parsed = parsed.trim();

    // Strip surrounding quotes if present
    let parsed = parsed.strip_prefix('"').unwrap_or(parsed);
    let parsed = parsed.strip_suffix('"').unwrap_or(parsed);

    parsed.to_string()
}

impl XdgDesktopFinder {
    pub fn new() -> Self {
        XdgDesktopFinder
    }
}