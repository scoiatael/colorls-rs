extern crate clap;
use clap::{Arg, App};

extern crate termion;
use termion::color;

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
        formatter.write_str("one of yellow, green, blue, red, cyan, magenta, grey, white, black")
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
            "black" => Ok(RealColor::Black),
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
    printer: Box<LsPrinter>,
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

struct ColorWrapper(pub Box<color::Color>);

fn color_for(config : &Config, color : &ColorType) -> ColorWrapper {
   let boxed : Box<color::Color> = match config.colors.get(color).unwrap_or(&RealColor::Grey) {
       &RealColor::Yellow => Box::new(color::Yellow),
        &RealColor::Green => Box::new(color::Green),
        &RealColor::Blue => Box::new(color::Blue),
        &RealColor::Red => Box::new(color::Red),
        &RealColor::Cyan => Box::new(color::Cyan),
        &RealColor::Magenta => Box::new(color::Magenta),
        &RealColor::Grey => Box::new(color::AnsiValue::rgb(2,2,2)),
        &RealColor::White => Box::new(color::AnsiValue::rgb(0,0,0)),
        &RealColor::Black => Box::new(color::AnsiValue::rgb(5,5,5)),
   };
    ColorWrapper(boxed)
}

struct LsEntry {
    path: path::PathBuf,
    attr: Attr,
}

type LsEntries = Vec<LsEntry>;

trait LsPrinter: fmt::Debug {
    fn print(&self, &Config, LsEntries);
}

impl color::Color for ColorWrapper {
    #[inline]
    fn write_fg(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (*self.0).write_fg(f)
    }

    #[inline]
    fn write_bg(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (*self.0).write_bg(f)
    }
}

#[derive(Debug)]
struct LongFormat {}
impl LsPrinter for LongFormat {
    fn print(&self, config : &Config, ls : Vec<LsEntry>) {
        for l in &ls {
            let name = l.path.display();
            println!("{icon}  {color}{name}{reset}",
                     name = name,
                     icon = l.attr.icon,
                     color = color::Fg(color_for(config, &l.attr.color)),
                     reset = color::Fg(color::Reset),
            )
        }
    }
}

#[derive(Debug)]
struct ShortFormat {}
impl LsPrinter for ShortFormat {
    fn print(&self, config : &Config, ls : Vec<LsEntry>) {
        for l in &ls {
            let name = l.path.file_name().unwrap().to_str().unwrap();
            println!("{icon}  {color}{name}{reset}",
                     name = name,
                     icon = l.attr.icon,
                     color = color::Fg(color_for(config, &l.attr.color)),
                     reset = color::Fg(color::Reset),
            )
        }
    }
}

fn run(action : Action) {
    if action.verbosity != Verbosity::Quiet {
        println!("Looking at {}", action.directory.display());

    }
    let dirs = fs::read_dir(action.directory).unwrap();
    let mut ls = Vec::new();

    for dir in dirs {
        let path = dir.unwrap().path();
        let attr = get_attr(&action.config, &path);
        ls.push(LsEntry { path : path, attr : attr});
    }
    action.printer.print(&action.config, ls)
}

fn main() {
    let matches = App::new("ColorLs")
        .version("0.1.0")
        .author("scoiatael <czapl.luk+git@gmail.com>")
        .about("List information about the FILEs (the current directory by default).")
        .arg(Arg::with_name("long")
             .long("long")
             .short("l")
             .help("Prints using long format"))
        .arg(Arg::with_name("v")
             .short("v")
             .multiple(true)
             .help("Sets the level of verbosity"))
        .arg(Arg::with_name("FILE")
             .required(false)
             .index(1))
        .get_matches();

    let mut verbosity = Verbosity::Quiet;
    match matches.occurrences_of("v") {
        0 => (),
        1 => verbosity = Verbosity::Warn,
        2 => verbosity = Verbosity::Debug,
        3 | _ => println!("Can't be more verbose!"),
    }
    let mut printer : Box<LsPrinter> = Box::new(ShortFormat{});
    match matches.occurrences_of("long") {
        1 => printer = Box::new(LongFormat{}),
        _ => (),
    }

    let file_icons = serde_yaml::from_str(include_str!("default_config/files.yaml")).unwrap();
    let folder_icons = serde_yaml::from_str(include_str!("default_config/folders.yaml")).unwrap();
    let file_aliases = serde_yaml::from_str(include_str!("default_config/file_aliases.yaml")).unwrap();
    let folder_aliases = serde_yaml::from_str(include_str!("default_config/folder_aliases.yaml")).unwrap();
    let colors = serde_yaml::from_str(include_str!("default_config/dark_colors.yaml")).unwrap();
    let cdir_path = env::current_dir().unwrap();
    let dir = matches.value_of("FILE").unwrap_or_else(|| cdir_path.to_str().unwrap());
    let path = path::PathBuf::from(dir);
    let action = Action {
        verbosity: verbosity,
        directory: path,
        config: Config {
            files: file_icons,
            file_aliases: file_aliases,
            folders: folder_icons,
            folder_aliases: folder_aliases,
            colors: colors,
        },
        printer: printer,
    };

    if verbosity == Verbosity::Debug {
        println!("{:?}", action);

    }
    run(action);
}
