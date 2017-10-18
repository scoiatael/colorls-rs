extern crate termion;
extern crate serde;
extern crate unicode_segmentation;
extern crate num_iter;

use std::path;
use std::fs;

mod colors;
mod formatter;
use self::formatter::{Entry,get_attr};
pub use self::formatter::{Formatter,EntryConfig,ShortFormat,LongFormat};
mod tabulator;
pub use self::tabulator::{Tabulator,Config,PlanningTabulator,NaiveTabulator};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Verbosity {
    Quiet,
    Warn,
    Debug,
}

#[derive(Debug)]
pub struct Action {
    pub verbosity: Verbosity,
    pub directory: path::PathBuf,
    pub config: Config,
    pub tabulator: Box<Tabulator>,
}

pub fn run(action : Action) {
    if action.verbosity != Verbosity::Quiet {
        println!("Looking at {}", action.directory.display());

    }
    let dirs = fs::read_dir(action.directory).unwrap();
    let config = action.config;
    let mut ls : Vec<Entry> = dirs.map(|dir| {
        let path = dir.unwrap().path();
        Entry { path: path.clone(), attr: get_attr(&config.entry, &path) }
    }).collect();
    ls.sort_unstable();
    let rows = action.tabulator.tabulate(&config, ls);
    for items in rows {
        for item in items {
            print!("{}", item);
        }
        println!("");
    }
}
