use std::ops::Range;

use regex::Regex;

use crate::pos::Pos;

pub struct Finder {
    haystack: String,
    offset: usize,
    regex: String,
}

impl Finder {
    pub fn new(haystack: String, offset: usize) -> Self {
        Self {
            haystack,
            offset,
            regex: String::new(),
        }
    }

    pub fn r#type(&mut self, char: char) {
        self.regex.push(char);
    }

    pub fn backspace(&mut self) {
        self.regex.pop();
    }

    pub fn find(&self) -> Option<Vec<Range<usize>>> {
        let re = Regex::new(&self.regex).ok()?;

        Some(
            re.find_iter(&self.haystack)
                .map(|m| {
                    let Range { start, end } = m.range();
                    start + self.offset..end + self.offset
                })
                .collect(),
        )
    }
}
