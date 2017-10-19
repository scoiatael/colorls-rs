use std::cmp::max;
use num_iter::range_step;

use std::fmt;

use self::super::formatter::{Formatter,Entry,EntryConfig};

#[derive(Debug)]
pub struct Config {
    pub entry: EntryConfig,
    pub max_width: usize,
    pub formatter: Box<Formatter>,
}

type Output = Vec<Vec<String>>;

pub trait Tabulator: fmt::Debug {
    fn tabulate(&self, &Config, Vec<Entry>) -> Output;
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
        assert_eq!(false, is_valid(vec![vec![1,3], vec![3,1]], 5))
    }

    #[test]
    fn when_fits() {
        assert_eq!(true, is_valid(vec![vec![1,2], vec![1,1]], 4))
    }
}

fn is_valid_as_rows(config: &Config, names : &Vec<Entry>, row_cap : usize) -> bool {
    is_valid(as_rows(&names.iter().map(|e| config.formatter.predict(e)).collect(), row_cap), config.max_width)
}

// NOTE: Assumes names has same-sized rows
fn format_as_rows(config : &Config, names : &Vec<Entry>, row_cap : usize) -> Output {
    let rows = as_rows(names, row_cap);
    let mut col_widths = vec![0; rows[0].len()];
    for r in &rows {
        for (i, s) in r.iter().enumerate() {
            let predicted = config.formatter.predict(s);
            col_widths[i] = max(col_widths[i],predicted)
        }
    }
    let entry_configs : Vec<EntryConfig> = col_widths.iter().map(|width| EntryConfig{width: *width, ..config.entry.clone()}).collect();
    let mut out = Vec::with_capacity(names.len());
    for r in rows {
        for (i, s) in r.iter().enumerate() {
            out.push(config.formatter.format(&entry_configs[i], s));
        }
    }
    as_rows(&out, row_cap)
}

fn max_width(config : &Config, names : &Vec<Entry>) -> usize {
    let mut width = 0;
    for l in names {
        let cwidth = config.formatter.predict(l);
        if cwidth > width {
            width = cwidth;
        }
    }
    width
}

const MIN_FORMAT_ENTRY_LENGTH : usize = 5;

#[derive(Debug)]
pub struct PlanningTabulator;
impl Tabulator for PlanningTabulator {
    fn tabulate(&self, config : &Config, names : Vec<Entry>) -> Output {
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

fn binsearch<T, F>(range : &Vec<T>, pred : F) -> Option<usize>
    where F: Fn(&T) -> bool {
    let mut slice = &range[..];
    let mut index = 0;
    let mut counter = range.len();
    loop {
        if slice.is_empty() {
            return None
        }
        counter -= 1;
        assert!(counter > 0);

        let mid = slice.len() / 2;
        let mid_value = &slice[mid];
        if pred(mid_value) {
            // Go left
            slice = &slice[0..mid+1]
        } else {
            // Go right
            index += mid;
            slice = &slice[mid..]
        }
        if slice.len() < 3 {
            if pred(&slice[0]) { return Some(index) }
            if slice.len() == 2 && pred(&slice[1]) { return Some(index+1) }
            return None
        }
    }
}

#[cfg(test)]
mod binsearch_tests {
    use super::*;
    #[test]
    fn for_simple_case() {
        assert_eq!(Some(1), binsearch(&vec![0,1,2], |i| *i >= 1))
    }

    #[test]
    fn when_nothing_matches() {
        assert_eq!(None, binsearch(&vec![0,1,2], |i| *i >= 3))
    }

    #[test]
    fn when_array_is_small() {
        assert_eq!(Some(1), binsearch(&vec![0,1], |i| *i >= 1));
        assert_eq!(Some(0), binsearch(&vec![0,1], |i| *i >= 0))
    }

    #[test]
    fn when_array_is_big() {
        assert_eq!(Some(11), binsearch(&vec![0,1,2,3,4,5,6,7,8,9,10,11,12,13,14], |i| *i >= 11));
    }

    #[test]
    fn when_array_is_empty() {
        assert_eq!(None, binsearch(&(vec![] as Vec<i64>), |i| *i >= 11));
    }
}


#[derive(Debug)]
pub struct BinsearchTabulator;
impl Tabulator for BinsearchTabulator {
    fn tabulate(&self, config : &Config, names : Vec<Entry>) -> Output {
        let width = max_width(config, &names);
        let min_rows = (config.max_width / (width + 1)) as i64;
        let max_rows = (config.max_width / MIN_FORMAT_ENTRY_LENGTH) as i64;
        let range : Vec<i64> = range_step(max_rows, min_rows, -1).collect();
        let row_cap = if let Some(idx) = binsearch(&range, |row_cap| is_valid_as_rows(config, &names, *row_cap as usize)) {
            range[idx]
        } else {
            1
        };
        format_as_rows(config, &names, row_cap as usize)
    }
}

#[derive(Debug)]
pub struct NaiveTabulator;
impl Tabulator for NaiveTabulator {
    fn tabulate(&self, config : &Config, names : Vec<Entry>) -> Output {
        let width = max_width(config, &names) + 2;
        let rows = config.max_width / width;
        format_as_rows(config, &names, rows)
    }
}
