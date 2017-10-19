extern crate clap;
use clap::{Arg, App};

extern crate termion;
use termion::terminal_size;

extern crate serde_yaml;

use std::env;
use std::path;

extern crate colorls;
use colorls::*;

fn main() {
    let matches = App::new("ColorLs")
        .version("0.1.0")
        .author("scoiatael <czapl.luk+git@gmail.com>")
        .about("List information about the FILEs (the current directory by default).")
        .arg(Arg::with_name("long")
             .long("long")
             .short("l")
             .help("Prints using long format"))
        .arg(Arg::with_name("naive")
             .long("naive")
             .short("n")
             .help("Prints using naive tabulator"))
        .arg(Arg::with_name("v")
             .short("v")
             .multiple(true)
             .help("Sets the level of verbosity"))
        .arg(Arg::with_name("FILE")
             .required(false)
             .index(1))
        .get_matches();

    let verbosity = match matches.occurrences_of("v") {
        0 => Verbosity::Quiet,
        1 => Verbosity::Warn,
        2 | _ =>  Verbosity::Debug,
    };
    let tabulator : Box<Tabulator> = match matches.occurrences_of("naive") {
        0 => Box::new(BinsearchTabulator),
        1 => Box::new(PlanningTabulator),
        _ => Box::new(NaiveTabulator),
    };
    let formatter : Box<Formatter> = match matches.occurrences_of("long") {
        0 => Box::new(ShortFormat),
        1 | _ =>  Box::new(LongFormat),
    };

    let file_icons = serde_yaml::from_str(include_str!("default_config/files.yaml")).unwrap();
    let folder_icons = serde_yaml::from_str(include_str!("default_config/folders.yaml")).unwrap();
    let file_aliases = serde_yaml::from_str(include_str!("default_config/file_aliases.yaml")).unwrap();
    let folder_aliases = serde_yaml::from_str(include_str!("default_config/folder_aliases.yaml")).unwrap();
    let colors = serde_yaml::from_str(include_str!("default_config/dark_colors.yaml")).unwrap();
    let cdir_path = env::current_dir().unwrap();
    let dir = matches.value_of("FILE").unwrap_or_else(|| cdir_path.to_str().unwrap());
    let path = path::PathBuf::from(dir);
    let width = terminal_size().unwrap().0 as usize;
    let action = Action {
        verbosity: verbosity,
        directory: path,
        config: Config {
            max_width: width,
            formatter,
            entry: EntryConfig {
                files: file_icons,
                folders: folder_icons,
                file_aliases,
                colors,
                folder_aliases,
                width,
            }
        },
        tabulator,
    };

    if verbosity == Verbosity::Debug {
        println!("{:?}", action);

    }
    run(action);
}
