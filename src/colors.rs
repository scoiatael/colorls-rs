use serde::de::{self, Visitor, Deserialize, Deserializer};
use std::fmt;
use termion::color;

#[derive(Hash, Debug, PartialEq, Eq, Clone, Copy)]
pub enum ColorType {
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
pub enum RealColor {
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

pub struct ColorWrapper(pub Box<color::Color>);

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
