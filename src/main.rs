extern crate clap;
use clap::{Arg, App};

extern crate termion;
use termion::color;

#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;
extern crate serde;
use serde::de::{self, Visitor, Deserialize, Deserializer};

use std::path;
use std::env;
use std::fs;
use std::fmt;
use std::ffi;

use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone, Copy)]
enum Verbosity {
    Quiet,
    Warn,
    Debug,
}

type Options = HashMap<String, String>;

#[derive(Debug)]
struct Config {
    files: Options,
    file_aliases: Options,
    folders: Options,
    folder_aliases: Options,
    colors: HashMap<ColorType, RealColor>,
}

#[derive(Hash, Debug, PartialEq, Eq, Clone, Copy)]
enum ColorType {
    UnrecognizedFile,
    RecognizedFile,
    Dir,
    DeadLink,
    Link,
    Write,
    Read,
    Exec,
    NoAccess,
    DayOld,
    HourOld,
    NoModifier,
    Report,
    User,
    Tree,
    Empty,
    Normal,
}

struct ColorTypeVisitor;
impl Visitor for ColorTypeVisitor {
    type Value = ColorType;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("one of unrecognized_file, recognized_file, dir, dead_link, link, write, read, exec, no_access, day_old, hour_old, no_modifier, report, user, tree, empty, normal")
    }

    fn visit_str<E>(self, value: &str) -> Result<ColorType, E>
        where E: de::Error
    {
        match value {
            "unrecognized_file" => Ok(ColorType::UnrecognizedFile),
            "recognized_file" => Ok(ColorType::RecognizedFile),
            "dir" => Ok(ColorType::Dir),
            "dead_link" => Ok(ColorType::DeadLink),
            "link" => Ok(ColorType::Link),
            "write" => Ok(ColorType::Write),
            "read" => Ok(ColorType::Read),
            "exec" => Ok(ColorType::Exec),
            "no_access" => Ok(ColorType::NoAccess),
            "day_old" => Ok(ColorType::DayOld),
            "hour_old" => Ok(ColorType::HourOld),
            "no_modifier" => Ok(ColorType::NoModifier),
            "report" => Ok(ColorType::Report),
            "user" => Ok(ColorType::User),
            "tree" => Ok(ColorType::Tree),
            "empty" => Ok(ColorType::Empty),
            "normal" => Ok(ColorType::Normal),
            _ => Err(E::custom(format!("Unknown ColorType: {}", value)))
        }
    }
}

impl Deserialize for ColorType {
    fn deserialize<D>(deserializer: D) -> Result<ColorType, D::Error>
        where D: Deserializer
    {
        deserializer.deserialize_str(ColorTypeVisitor)
    }
}

#[derive(Hash, Debug, PartialEq, Eq, Clone, Copy)]
enum RealColor {
    Yellow,
    Green,
    Blue,
    Red,
    Cyan,
    Magenta,
    Grey,
    White,
    Black,
}

struct RealColorVisitor;
impl Visitor for RealColorVisitor {
    type Value = RealColor;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("one of yellow, green, blue, red, cyan, magenta, grey, white")
    }

    fn visit_str<E>(self, value: &str) -> Result<RealColor, E>
        where E: de::Error
    {
        match value {
            "yellow" => Ok(RealColor::Yellow),
            "green" => Ok(RealColor::Green),
            "blue" => Ok(RealColor::Blue),
            "red" => Ok(RealColor::Red),
            "cyan" => Ok(RealColor::Cyan),
            "magenta" => Ok(RealColor::Magenta),
            "grey" => Ok(RealColor::Grey),
            "white" => Ok(RealColor::White),
            _ => Err(E::custom(format!("Unknown RealColor: {}", value)))
        }
    }
}

impl Deserialize for RealColor {
    fn deserialize<D>(deserializer: D) -> Result<RealColor, D::Error>
        where D: Deserializer
    {
        deserializer.deserialize_str(RealColorVisitor)
    }
}

#[derive(Debug)]
struct Action {
    verbosity: Verbosity,
    directory: path::PathBuf,
    config: Config,
}

struct Attr {
    icon: String,
    color: ColorType,
}

fn get_file_attr(conf : &Config, suffix : &str) -> Attr {
    match conf.files.get(suffix) {
        Some(icon) => Attr { icon: icon.clone(), color: ColorType::RecognizedFile },
        None => Attr { icon: conf.files.get("file").unwrap().clone(), color: ColorType::UnrecognizedFile }
    }
}

fn get_file_attr_alias(conf : &Config, suffix : &str) -> Attr {
    match conf.file_aliases.get(suffix) {
        Some(alias) => get_file_attr(conf, alias),
        None => get_file_attr(conf, suffix)
    }
}

fn get_folder_attr(conf : &Config, name : &str) -> Attr {
    match conf.folders.get(name) {
        Some(icon) => Attr { icon: icon.clone(), color: ColorType::Dir },
        None => Attr { icon: conf.files.get("file").unwrap().clone(), color: ColorType::Dir }
    }
}

fn get_folder_attr_alias(conf : &Config, name : &str) -> Attr {
    match conf.folder_aliases.get(name) {
        Some(alias) => get_folder_attr(conf, alias),
        None => get_folder_attr(conf, name)
    }
}

fn filename_without_leading_dot(path : &path::Path) -> String {
    let mut file_name = path.file_name().unwrap().to_str().unwrap().to_string();
    file_name.remove(0);
    file_name
}

fn get_attr(config : &Config, path : &path::Path) -> Attr {
    if path.is_dir() {
        let file_name = path.file_name().unwrap().to_str().unwrap();
        return get_folder_attr_alias(config, file_name)
    } else {
        let filename_without_leading_dot = filename_without_leading_dot(path);
        let default = ffi::OsStr::new(&filename_without_leading_dot);
        let extension = path.extension().unwrap_or(default).to_str().unwrap();
        return get_file_attr_alias(config, extension)
    }
}

fn color_for(config : &Config, color : &ColorType) -> color::AnsiValue {
    match config.colors.get(color).unwrap_or(&RealColor::Grey) {
        &RealColor::Yellow => color::AnsiValue::rgb(0,2,2),
        &RealColor::Green => color::AnsiValue::rgb(0,2,0),
        &RealColor::Blue => color::AnsiValue::rgb(0,0,2),
        &RealColor::Red => color::AnsiValue::rgb(2,0,0),
        &RealColor::Cyan => color::AnsiValue::rgb(2,2,0),
        &RealColor::Magenta => color::AnsiValue::rgb(2,0,2),
        &RealColor::Grey => color::AnsiValue::rgb(2,2,2),
        &RealColor::White => color::AnsiValue::rgb(0,0,0),
        &RealColor::Black => color::AnsiValue::rgb(5,5,5),
    }
}

fn run(action : Action) {
    if action.verbosity != Verbosity::Quiet {
        println!("Looking at {}", action.directory.display());

    }
    let dirs = fs::read_dir(action.directory).unwrap();

    for dir in dirs {
        let path = dir.unwrap().path();
        let attr = get_attr(&action.config, &path);
        let name = path.display();
        println!("{icon}  {color}{name}{reset}",
                 name = name,
                 icon = attr.icon,
                 color = color::Fg(color_for(&action.config, &attr.color)),
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
    let folder_icons = serde_yaml::from_str(include_str!("default_config/folders.yaml")).unwrap();
    let file_aliases = serde_yaml::from_str(include_str!("default_config/file_aliases.yaml")).unwrap();
    let folder_aliases = serde_yaml::from_str(include_str!("default_config/folder_aliases.yaml")).unwrap();
    let colors = serde_yaml::from_str(include_str!("default_config/dark_colors.yaml")).unwrap();
    let cdir = env::current_dir().unwrap();
    let action = Action {
        verbosity: verbosity,
        directory: cdir,
        config: Config {
            files: file_icons,
            file_aliases: file_aliases,
            folders: folder_icons,
            folder_aliases: folder_aliases,
            colors: colors,
        },
    };

    if verbosity == Verbosity::Debug {
        println!("{:?}", action);

    }
    run(action);
}
