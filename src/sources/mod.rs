mod xdg_desktop;
mod path;

pub use xdg_desktop::XdgDesktopFinder;
pub use path::PathFinder;
use crate::fuzzy::FuzzyItem;
use clap::ValueEnum;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum Source {
    Desktop,
    Path,
}

impl Source {
    pub fn get_finder(&self) -> Box<dyn SourceFinder> {
        match self {
            Source::Desktop => Box::new(XdgDesktopFinder::new()),
            Source::Path => Box::new(PathFinder::new()),
        }
    }
}

pub trait SourceFinder: Send + Sync {
    fn find_entries(&self) -> Vec<FuzzyItem>;
    fn source_order(&self) -> usize;
}

// Implement Send and Sync for XdgDesktopFinder and PathFinder
unsafe impl Send for XdgDesktopFinder {}
unsafe impl Sync for XdgDesktopFinder {}
unsafe impl Send for PathFinder {}
unsafe impl Sync for PathFinder {}