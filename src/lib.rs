extern crate termion;
use termion::color;

extern crate serde;

extern crate unicode_segmentation;
use unicode_segmentation::UnicodeSegmentation;

use std::path;
use std::fs;
use std::fmt;
use std::ffi;

use std::cmp::max;
use std::cmp::Ordering;
use std::collections::HashMap;

extern crate num_iter;
use num_iter::range_step;

mod colors;
use self::colors::{ColorType, RealColor, ColorWrapper};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Verbosity {
    Quiet,
    Warn,
    Debug,
}

pub type Options = HashMap<String, String>;

#[derive(Debug)]
pub struct Config {
    pub files: Options,
    pub file_aliases: Options,
    pub folders: Options,
    pub folder_aliases: Options,
    pub colors: HashMap<ColorType, RealColor>,
    pub max_width: usize,
    pub printer: Box<EntryPrinter>,
}

#[derive(Debug)]
pub struct Action {
    pub verbosity: Verbosity,
    pub directory: path::PathBuf,
    pub config: Config,
    pub formatter: Box<Formatter>,
}

#[derive(PartialEq, Eq, Clone)]
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
        None => Attr { icon: conf.folders.get("folder").unwrap().clone(), color: ColorType::Dir }
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

#[derive(Eq, Clone)]
pub struct Entry {
    path: path::PathBuf,
    attr: Attr,
}

impl Ord for Entry {
    fn cmp(&self, other: &Entry) -> Ordering {
        self.path.cmp(&other.path)
    }
}

impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Entry) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Entry {
    fn eq(&self, other: &Entry) -> bool {
        self.path == other.path
    }
}

pub struct EntryPrinterConfig {
    width : usize,
}

pub trait EntryPrinter: fmt::Debug {
    fn format(&self, &Config, &EntryPrinterConfig, &Entry) -> String;
    fn predict(&self, &Entry) -> usize;
}

#[derive(Debug)]
pub struct LongFormat {}
impl EntryPrinter for LongFormat {
    fn format(&self, config : &Config, ep_config : &EntryPrinterConfig, entry : &Entry) -> String {
            let name = entry.path.display();
            let width = ep_config.width - 2;
            format!("{icon} {color}{name:<width$}{reset}",
                     name = name,
                     icon = entry.attr.icon,
                     color = color::Fg(color_for(config, &entry.attr.color)),
                     reset = color::Fg(color::Reset),
                     width = width,
            )
    }

    fn predict(&self, entry : &Entry) -> usize {
        strlen(&format!("{}", entry.path.display())) + 2 // Icon + space
    }
}

#[derive(Debug)]
pub struct ShortFormat {}

fn short_name(l : &Entry) -> String {
    l.path.file_name().unwrap().to_str().unwrap().to_string()
}

impl EntryPrinter for ShortFormat {
    fn format(&self, config : &Config, ep_config : &EntryPrinterConfig, entry : &Entry) -> String {
        let name = short_name(entry);
        let width = ep_config.width - 2;
        format!("{icon}{color}{name:<width$}{reset}",
                name = name,
                icon = entry.attr.icon,
                color = color::Fg(color_for(config, &entry.attr.color)),
                reset = color::Fg(color::Reset),
                width = width,
        )
    }

    fn predict(&self, entry : &Entry) -> usize {
        strlen(&short_name(entry)) + 3
    }
}

// NOTE: Colors DO count to length. Sadly.
fn strlen(s : &String) -> usize {
    s.graphemes(true).count() as usize
}

#[cfg(test)]
mod strlen_tests {
    use super::*;
    #[test]
    fn for_normal_string() {
        assert_eq!(6, strlen(&".local".to_string()))
    }

    #[test]
    fn for_string_with_icons() {
        assert_eq!(7, strlen(&".local".to_string()))
    }

    #[test]
    fn for_string_with_weird_stuff() {
        assert_eq!(7, strlen(&"a̐.local".to_string()))
    }

    #[test]
    fn for_string_with_icons_via_code() {
        assert_eq!(7, strlen(&format!("{}.local", "\u{f115}")))
    }

    #[test]
    fn for_string_with_color() {
        assert_eq!(20, strlen(&format!("{color}.local{reset}", color = color::Fg(color::Red), reset = color::Fg(color::Reset))))
    }
}

type Output = Vec<Vec<String>>;

pub trait Formatter: fmt::Debug {
    fn format(&self, &Config, Vec<Entry>) -> Output;
}

fn as_rows<T : Clone>(names : &Vec<T>, row_cap : usize) -> Vec<Vec<T>> {
    let mut rows = Vec::with_capacity(names.len() / row_cap + 1);
    let mut row = Vec::with_capacity(row_cap);
    for (i, out) in names.iter().enumerate() {
        row.push(out.clone());
        if i % row_cap == row_cap - 1 {
            rows.push(row);
            row = Vec::new();
        }
    }
    rows
}

// NOTE: Assumes out has same-sized rows
fn is_valid(out : Vec<Vec<usize>>, max_width : usize) -> bool {
    let mut col_widths = vec![0; out[0].len()];
    for r in out {
        for (i, s) in r.iter().enumerate() {
            col_widths[i] = max(col_widths[i], *s);
        }
    }
    let mut width = 0;
    for c in col_widths { width += c }
    return width < max_width
}

#[cfg(test)]
mod is_valid_tests {
    use super::*;
    #[test]
    fn for_simple_case() {
        assert_eq!(false, is_valid(vec![vec![1,2], vec![2,1]], 2))
    }

    #[test]
    fn when_total_col_width_exceeds_max() {
        assert_eq!(false, is_valid(vec![vec![1,3], vec![3,1]], 3))
    }

    #[test]
    fn when_fits() {
        assert_eq!(true, is_valid(vec![vec![1,2], vec![1,1]], 4))
    }
}

fn is_valid_as_rows(config: &Config, names : &Vec<Entry>, row_cap : usize) -> bool {
    is_valid(as_rows(&names.iter().map(|e| config.printer.predict(e)).collect(), row_cap), config.max_width)
}

// NOTE: Assumes names has same-sized rows
fn format_as_rows(config : &Config, names : &Vec<Entry>, row_cap : usize) -> Output {
    let rows = as_rows(names, row_cap);
    let mut col_widths = vec![0; rows[0].len()];
    for r in &rows {
        for (i, s) in r.iter().enumerate() {
            let predicted = config.printer.predict(s);
            col_widths[i] = max(col_widths[i],predicted)
        }
    }
    let ep_configs : Vec<EntryPrinterConfig> = col_widths.iter().map(|width| EntryPrinterConfig{width: *width}).collect();
    let mut out = Vec::with_capacity(names.len());
    for r in rows {
        for (i, s) in r.iter().enumerate() {
            out.push(config.printer.format(config, &ep_configs[i], s));
        }
    }
    as_rows(&out, row_cap)
}

fn max_width(config : &Config, names : &Vec<Entry>) -> usize {
    let mut width = 0;
    for l in names {
        let cwidth = config.printer.predict(l);
        if cwidth > width {
            width = cwidth;
        }
    }
    width
}

const MIN_FORMAT_ENTRY_LENGTH : usize = 5;

#[derive(Debug)]
pub struct PlanningFormatter {}
impl Formatter for PlanningFormatter {
    fn format(&self, config : &Config, names : Vec<Entry>) -> Output {
        let width = max_width(config, &names);
        let min_rows = (config.max_width / (width + 1)) as i64;
        let max_rows = (config.max_width / MIN_FORMAT_ENTRY_LENGTH) as i64;
        for row_cap in range_step(max_rows, min_rows, -1) {
            if is_valid_as_rows(config, &names, row_cap as usize) {
                return format_as_rows(config, &names, row_cap as usize)
            }
        }
        format_as_rows(config, &names, min_rows as usize)
    }
}

#[derive(Debug)]
pub struct NaiveFormatter {}
impl Formatter for NaiveFormatter {
    fn format(&self, config : &Config, names : Vec<Entry>) -> Output {
        let width = max_width(config, &names) + 2;
        let rows = config.max_width / width;
        format_as_rows(config, &names, rows)
    }
}

pub fn run(action : Action) {
    if action.verbosity != Verbosity::Quiet {
        println!("Looking at {}", action.directory.display());

    }
    let dirs = fs::read_dir(action.directory).unwrap();
    let config = action.config;
    let mut ls : Vec<Entry> = dirs.map(|dir| {
        let path = dir.unwrap().path();
        Entry { path: path.clone(), attr: get_attr(&config, &path) }
    }).collect();
    ls.sort_unstable();
    let rows = action.formatter.format(&config, ls);
    for items in rows {
        for item in items {
            print!("{}", item);
        }
        println!("");
    }
}
