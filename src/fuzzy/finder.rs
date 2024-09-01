use super::history::History;
use std::path::PathBuf;
use std::collections::HashMap;
use super::scorer::match_and_score;

pub struct FuzzyFinder {
    items: Vec<FuzzyItem>,
    history: History,
}

#[derive(Clone)]
pub struct FuzzyItem {
    pub display: String,
    pub exec: String,
    pub priority: u8,
    pub source_order: usize,
    pub description: String,
    pub source_path: String,
    pub search_desc: bool,
    pub icon: String,
}

impl FuzzyFinder {
    pub fn new(history_file: Option<PathBuf>) -> Self {
        FuzzyFinder {
            items: Vec::new(),
            history: History::new(history_file),
        }
    }

    pub fn add_items(&mut self, new_items: Vec<FuzzyItem>) {
        let mut unique_items: HashMap<String, FuzzyItem> = HashMap::new();

        // First, add existing items to the HashMap
        for item in self.items.drain(..) {
            unique_items.entry(item.exec.clone())
                .and_modify(|existing| {
                    if item.source_order < existing.source_order {
                        *existing = item.clone();
                    }
                })
                .or_insert(item);
        }

        // Then, add new items, respecting source order
        for item in new_items {
            unique_items.entry(item.exec.clone())
                .and_modify(|existing| {
                    if item.source_order < existing.source_order {
                        *existing = item.clone();
                    }
                })
                .or_insert(item);
        }

        self.items = unique_items.into_values().collect();
    }

    pub fn find(&self, query: &str) -> Vec<(f64, &FuzzyItem)> {
        let query = query.to_lowercase();
        
        // If the query is empty, return all items sorted by priority and history
        if query.is_empty() {
            return self.items
                .iter()
                .map(|item| {
                    let history_boost = self.history.get_count(&item.exec) as f64 * 10.0;
                    let priority_score = item.priority as f64;
                    (priority_score + history_boost, item)
                })
                .collect();
        }

        // Rest of the method remains the same for non-empty queries
        let mut matches: Vec<_> = self.items
            .iter()
            .filter_map(|item| {
                let mut total_score = 0.0;

                // Always score the display field
                total_score += match_and_score(&item.display, &query).unwrap_or(0.0) as f64;

                // Score exec only if it's different from display
                if item.exec != item.display {
                    total_score += (match_and_score(&item.exec, &query).unwrap_or(0.0).max(0.0) as f64) * 0.8;
                }

                // Score description only if search_desc is true
                if item.search_desc {
                    total_score += (match_and_score(&item.description, &query).unwrap_or(0.0).max(0.0) as f64) * 0.6;
                }

                // Score source_path only if it's different from exec
                if item.source_path != item.exec {
                    total_score += (match_and_score(&item.source_path, &query).unwrap_or(0.0).max(0.0) as f64) * 0.4;
                }

                if total_score > 0.0 {
                    let history_boost = self.history.get_count(&item.exec) as f64 * 10.0;
                    Some(((total_score + history_boost) * item.priority as f64, item))
                } else {
                    None
                }
            })
            .collect();
        matches.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        matches
    }

    pub fn record_usage(&mut self, command: &str) {
        self.history.record_usage(command);
    }

    pub fn item_count(&self) -> usize {
        self.items.len()
    }
}