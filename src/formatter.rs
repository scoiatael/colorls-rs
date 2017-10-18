use std::collections::HashMap;
use std::path;
use std::cmp::Ordering;
use std::ffi;
use std::fmt;

use unicode_segmentation::UnicodeSegmentation;

use termion::color;

use self::super::colors::{ColorType, RealColor, ColorWrapper};

pub type Options = HashMap<String, String>;

#[derive(Debug, Clone)]
pub struct EntryConfig {
    pub files: Options,
    pub file_aliases: Options,
    pub folders: Options,
    pub folder_aliases: Options,
    pub colors: HashMap<ColorType, RealColor>,
    pub width: usize,
}

#[derive(PartialEq, Eq, Clone)]
pub struct Attr {
    icon: String,
    color: ColorType,
}

fn get_file_attr(conf : &EntryConfig, suffix : &str) -> Attr {
    match conf.files.get(suffix) {
        Some(icon) => Attr { icon: icon.clone(), color: ColorType::RecognizedFile },
        None => Attr { icon: conf.files.get("file").unwrap().clone(), color: ColorType::UnrecognizedFile }
    }
}

fn get_file_attr_alias(conf : &EntryConfig, suffix : &str) -> Attr {
    match conf.file_aliases.get(suffix) {
        Some(alias) => get_file_attr(conf, alias),
        None => get_file_attr(conf, suffix)
    }
}

fn get_folder_attr(conf : &EntryConfig, name : &str) -> Attr {
    match conf.folders.get(name) {
        Some(icon) => Attr { icon: icon.clone(), color: ColorType::Dir },
        None => Attr { icon: conf.folders.get("folder").unwrap().clone(), color: ColorType::Dir }
    }
}

fn get_folder_attr_alias(conf : &EntryConfig, name : &str) -> Attr {
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

pub fn get_attr(config : &EntryConfig, path : &path::Path) -> Attr {
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

fn color_for(config : &EntryConfig, color : &ColorType) -> ColorWrapper {
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
    pub path: path::PathBuf,
    pub attr: Attr,
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

pub trait Formatter: fmt::Debug {
    fn format(&self, &EntryConfig, &Entry) -> String;
    fn predict(&self, &Entry) -> usize;
}

#[derive(Debug)]
pub struct LongFormat;
impl Formatter for LongFormat {
    fn format(&self, config :  &EntryConfig, entry : &Entry) -> String {
        let name = entry.path.display();
        let width = config.width - 2;
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
pub struct ShortFormat;

fn short_name(l : &Entry) -> String {
    l.path.file_name().unwrap().to_str().unwrap().to_string()
}

impl Formatter for ShortFormat {
    fn format(&self, config : &EntryConfig, entry : &Entry) -> String {
        let name = short_name(entry);
        let width = config.width - 2;
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
