extern crate clap;
use clap::{Arg, App};

extern crate termion;
use termion::color;

use std::path;
use std::env;
use std::fs;

#[derive(Debug, PartialEq, Clone, Copy)]
enum Verbosity {
    Quiet,
    Warn,
    Debug,
}

#[derive(Debug)]
struct Action {
    verbosity: Verbosity,
    directory: path::PathBuf,
}

struct Attr {
    icon: String,
    color: color::AnsiValue,
}

// ai:       "\ue7b4"
// android:  "\ue70e"
// apple:    "\uf179"
// audio:    "\uf001"
// avro:     "\ue60b"
// c:        "\ue61e"
// clj:      "\ue768"
// coffee:   "\uf0f4"
// conf:     "\ue615"
// cpp:      "\ue61d"
// css:      "\ue749"
// d:        "\ue7af"
// dart:     "\ue798"
// db:       "\uf1c0"
// diff:     "\uf440"
// doc:      "\uf1c2"
// ebook:    "\ue28b"
// epub:     "\ue28a"
// erl:      "\ue7b1"
// file:     "\uf15b"
// font:     "\uf031"
// gform:    "\uf298"
// git:      "\uf1d3"
// go:       "\ue626"
// hs:       "\ue777"
// html:     "\uf13b"
// image:    "\uf1c5"
// iml:      "\ue7b5"
// java:     "\ue204"
// js:       "\ue74e"
// json:     "\ue60b"
// less:     "\ue758"
// lua:      "\ue620"
// md:       "\uf48a"
// mustache: "\ue60f"
// npmignore: "\ue71e"
// pdf:      "\uf1c1"
// php:      "\ue73d"
// pl:       "\ue769"
// ppt:      "\uf1c4"
// psd:      "\ue7b8"
// py:       "\ue606"
// r:        "\uf25d"
// rb:       "\ue21e"
// rdb:      "\ue76d"
// rss:      "\uf09e"
// scala:    "\ue737"
// shell:    "\uf489"
// sqlite3:  "\ue7c4"
// styl:     "\ue600"
// tex:      "\ue600"
// ts:       "\ue628"
// twig:     "\ue61c"
// txt:      "\uf15c"
// video:    "\uf03d"
// vim:      "\ue62b"
// windows:  "\uf17a"
// xls:      "\uf1c3"
// xml:      "\ue619"
// yml:      "\uf481"
// zip:      "\uf410"

fn get_attr(_ : &path::Path) -> Attr {
    return Attr { icon: String::from("\u{f17a}"), color: color::AnsiValue::rgb(2,2,5) }
}

fn run(action : Action) {
    if action.verbosity != Verbosity::Quiet {
        println!("Looking at {}", action.directory.display());

    }
    let dirs = fs::read_dir(action.directory).unwrap();

    for dir in dirs {
        let path = dir.unwrap().path();
        let attr = get_attr(&path);
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

    let cdir = env::current_dir().unwrap();
    let action = Action {
        verbosity: verbosity,
        directory: cdir,
    };

    if verbosity == Verbosity::Debug {
        println!("{:?}", action);

    }
    run(action);
}
