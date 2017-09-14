extern crate clap;
use clap::{Arg, App};

extern crate termion;
use termion::color;

extern crate serde_derive;
extern crate serde_yaml;

use std::path;
use std::env;
use std::fs;

use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone, Copy)]
enum Verbosity {
    Quiet,
    Warn,
    Debug,
}

#[derive(Debug)]
struct Icons {
    file: HashMap<String, String>,
}

#[derive(Debug)]
struct Action {
    verbosity: Verbosity,
    directory: path::PathBuf,
    icons: Icons,
}

struct Attr {
    icon: String,
    color: color::AnsiValue,
}

fn get_attr(icons : &Icons, _ : &path::Path) -> Attr {
    return Attr { icon: icons.file.get("windows").unwrap().clone(), color: color::AnsiValue::rgb(2,2,5) }
}

fn run(action : Action) {
    if action.verbosity != Verbosity::Quiet {
        println!("Looking at {}", action.directory.display());

    }
    let dirs = fs::read_dir(action.directory).unwrap();

    for dir in dirs {
        let path = dir.unwrap().path();
        let attr = get_attr(&action.icons, &path);
        let name = path.display();
        println!("{icon}  {color}{name}{reset}",
                 name = name,
                 icon = attr.icon,
                 color = color::Fg(attr.color),
                 reset = color::Fg(color::Reset),
        )
    }
}

fn main() {
     let matches = App::new("ColorLs")
                          .version("0.1.0")
                        .author("scoiatael <czapl.luk+git@gmail.com>")
                          .about("list directory contents")
                          .arg(Arg::with_name("v")
                               .short("v")
                               .multiple(true)
                               .help("Sets the level of verbosity"))
                          .get_matches();

    let mut verbosity = Verbosity::Quiet;
    match matches.occurrences_of("v") {
        0 => (),
        1 => verbosity = Verbosity::Warn,
        2 => verbosity = Verbosity::Debug,
        3 | _ => println!("Can't be more verbose!"),
    }

    let file_icons = serde_yaml::from_str(include_str!("default_config/files.yaml")).unwrap();
    let cdir = env::current_dir().unwrap();
    let action = Action {
        verbosity: verbosity,
        directory: cdir,
        icons: Icons {
            file: file_icons,
        },
    };

    if verbosity == Verbosity::Debug {
        println!("{:?}", action);

    }
    run(action);
}
