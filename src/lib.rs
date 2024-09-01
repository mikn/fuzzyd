pub mod fuzzy;
pub mod ui;
pub mod sources;
pub mod launcher;
pub mod config;
pub mod error;

use fuzzy::FuzzyFinder;
use crate::fuzzy::FuzzyItem;
use sources::Source;
use launcher::SystemdLauncher;
use ui::TerminalUI;
use config::Config;
use error::FuzzydError;
use dirs;
use std::path::PathBuf;
use std::time::Instant;
use rayon::prelude::*;

pub struct Fuzzyd {
    ui: TerminalUI,
    launcher: SystemdLauncher,
    config: Config,
    finder: FuzzyFinder,
}

impl Fuzzyd {
    pub fn new(
        sources: Vec<Source>,
        debug: bool,
        config_path: Option<PathBuf>,
        history_file: Option<PathBuf>,
        dry_run: bool,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let start_time = Instant::now();

        let mut config = match config_path {
            Some(path) => Config::load_from_file(path)?,
            None => Config::load()?,
        };
        config.debug = debug;  // Override the config debug setting with the command-line flag

        let history_file = history_file.or_else(|| {
            config.history.file.as_ref().map(PathBuf::from)
        }).or_else(|| {
            dirs::data_dir().map(|mut path| {
                path.push("fuzzyd");
                std::fs::create_dir_all(&path).ok();
                path.push("fuzzyd.history");
                path
            })
        });

        let mut finder = FuzzyFinder::new(history_file);
        
        // Parallelize the loading of items from different sources
        let source_results: Vec<_> = sources.par_iter().map(|source| {
            let source_start_time = Instant::now();
            let source_finder = source.get_finder();
            let items = source_finder.find_entries();
            let item_count = items.len();
            let source_duration = source_start_time.elapsed();
            (source, items, item_count, source_duration)
        }).collect();

        let mut total_items = 0;
        for (source, items, item_count, source_duration) in source_results {
            finder.add_items(items);
            total_items += item_count;

            if config.debug {
                println!("Loaded {} items from {:?} source in {:?}", item_count, source, source_duration);
            }
        }

        let initialization_time = start_time.elapsed();

        if config.debug {
            println!("Total items added to finder: {}", total_items);
            println!("Total time taken to initialize: {:?}", initialization_time);
        }

        let ui = TerminalUI::new(config.ui.clone(), config.debug);
        let launcher = SystemdLauncher::new(dry_run, &config.systemd_run);

        Ok(Fuzzyd { ui, launcher, config, finder })
    }

    pub fn run(&mut self) -> Result<(), FuzzydError> {
        if self.config.debug {
            println!("Welcome to fuzzyd!");
            println!("Enter your search query or press Esc with an empty query to exit.");
        }

        loop {
            match self.ui.run(&mut self.finder)? {
                Some(item) => {
                    if self.config.debug {
                        println!("Launching: {}", item.exec);
                    }
                    self.finder.record_usage(&item.exec);
                    self.launcher.launch(&item)?;
                    break; // Exit after launching an application
                }
                None => {
                    if self.config.debug {
                        println!("Exiting fuzzyd. Goodbye!");
                    }
                    break;
                }
            }
        }

        Ok(())
    }

    pub fn launch(&mut self, item: &FuzzyItem) -> Result<(), FuzzydError> {
        self.finder.record_usage(&item.exec);
        self.launcher.launch(item)
    }
}