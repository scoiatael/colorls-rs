use std::cmp::max;
use num_iter::range_step;

use std::fmt;

use self::super::formatter::{Formatter,Entry,EntryConfig};

#[derive(Debug)]
pub struct Config {
    pub entry: EntryConfig,
    pub max_width: usize,
    pub formatter: Box<Formatter>,
    pub entry_order : EntryOrder,
}

type Rows<T> = Vec<Vec<T>>;

#[derive(Hash, Debug, PartialEq, Eq, Clone, Copy)]
pub enum EntryOrder {
    Horizontal,
    Vertical
}

fn break_lines_with<T : Clone>(order : EntryOrder, names : &Vec<T>, n_cols : usize) -> Rows<T> {
    let n_rows = (names.len() + n_cols - 1) / n_cols;
    let mut rows = vec![Vec::with_capacity(n_cols); n_rows];
    for (i, out) in names.iter().enumerate() {
        let pos = match order {
            EntryOrder::Horizontal => i / n_cols,
            EntryOrder::Vertical => i % n_rows,
        };
        rows[pos].push(out.clone());
    }
    rows
}

#[cfg(test)]
mod horizontal_line_breaker_tests {
    use super::*;
    #[test]
    fn for_simple_case() {
        assert_eq!(vec![vec![1,2], vec![3,4]], break_lines_with(EntryOrder::Horizontal, &vec![1,2,3,4], 2))
    }

    #[test]
    fn when_only_one_row() {
        assert_eq!(vec![vec![1,2,3]], break_lines_with(EntryOrder::Horizontal, &vec![1,2,3], 10))
    }
}

#[cfg(test)]
mod vertical_line_breaker_tests {
    use super::*;
    #[test]
    fn for_simple_case() {
        assert_eq!(vec![vec![1,3], vec![2,4]], break_lines_with(EntryOrder::Vertical, &vec![1,2,3,4], 2))
    }

    #[test]
    fn when_only_one_row() {
        assert_eq!(vec![vec![1,2,3]], break_lines_with(EntryOrder::Vertical, &vec![1,2,3], 10))
    }
}

type Output = Rows<String>;
pub trait Tabulator: fmt::Debug {
    fn tabulate(&self, &Config, Vec<Entry>) -> Output;
}

type ColumnSetup = Vec<usize>;

fn is_valid(out : &ColumnSetup, max_width : usize) -> bool {
    let mut width = 0;
    for c in out { width += *c }
    return width < max_width
}

// NOTE: Assumes out has same-sized rows
fn column_setup(out : Vec<Vec<usize>>) -> ColumnSetup {
    if out.is_empty() {
        return vec![]
    }
    let mut col_widths = vec![0; out[0].len()];
    for r in out {
        for (i, s) in r.iter().enumerate() {
            col_widths[i] = max(col_widths[i], *s);
        }
    }
    col_widths
}

#[cfg(test)]
mod is_valid_tests {
    use super::*;
    #[test]
    fn for_simple_case() {
        assert_eq!(false, is_valid(&column_setup(vec![vec![1,2], vec![2,1]]), 2))
    }

    #[test]
    fn when_total_col_width_exceeds_max() {
        assert_eq!(false, is_valid(&column_setup(vec![vec![1,3], vec![3,1]]), 5))
    }

    #[test]
    fn when_fits() {
        assert_eq!(true, is_valid(&column_setup(vec![vec![1,2], vec![1,1]]), 4))
    }
}

fn predict_column_setup(config: &Config, names : &Vec<Entry>, n_cols : usize) -> ColumnSetup {
    column_setup(break_lines_with(config.entry_order, &names.iter().map(|e| config.formatter.predict(e)).collect(), n_cols))
}

fn is_valid_as_rows(config: &Config, names : &Vec<Entry>, n_cols : usize) -> Option<ColumnSetup> {
    let setup = predict_column_setup(config, names, n_cols);
    if is_valid(&setup, config.max_width) {
        Some(setup)
    } else { None }
}

fn format_as_rows(config : &Config, names : &Vec<Entry>, col_widths : ColumnSetup) -> Output {
    let n_cols = col_widths.len();
    let rows = break_lines_with(config.entry_order, names, n_cols);
    let entry_configs : Vec<EntryConfig> = col_widths.iter().map(|width| EntryConfig{width: *width, ..config.entry.clone()}).collect();
    let mut out : Output = rows.iter().map(|r| Vec::with_capacity(r.len())).collect();
    for (y, row) in rows.iter().enumerate() {
        for (x, item) in row.iter().enumerate() {
            out[y].push(config.formatter.format(&entry_configs[x], item));
        }
    }
    out
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
        let min_cols = max((config.max_width / (width + 1)) as i64, 1);
        let max_cols = (config.max_width / MIN_FORMAT_ENTRY_LENGTH) as i64;
        for n_cols in range_step(max_cols, min_cols, -1) {
            if let Some(column_setup) = is_valid_as_rows(config, &names, n_cols as usize) {
                return format_as_rows(config, &names, column_setup)
            }
        }
        format_as_rows(config, &names, predict_column_setup(config, &names, 1))
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
        // NOTE: Cannot use usize, due to negative range.
        let min_cols = max((config.max_width / (width + 1)) as i64, 1);
        let max_cols = (config.max_width / MIN_FORMAT_ENTRY_LENGTH) as i64;
        let range : Vec<i64> = range_step(max_cols, min_cols, -1).collect();
        let n_cols = if let Some(idx) = binsearch(&range, |n_cols| is_valid_as_rows(config, &names, *n_cols as usize).is_some()) {
            range[idx]
        } else {
            1
        };
        format_as_rows(config, &names, predict_column_setup(config, &names, n_cols as usize))
    }
}

#[derive(Debug)]
pub struct NaiveTabulator;
impl Tabulator for NaiveTabulator {
    fn tabulate(&self, config : &Config, names : Vec<Entry>) -> Output {
        let width = max_width(config, &names) + 2;
        let rows = max(config.max_width / width, 1);
        format_as_rows(config, &names, predict_column_setup(config, &names, rows as usize))
    }
}
