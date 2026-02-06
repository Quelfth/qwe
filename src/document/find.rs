use std::ops::Range;

use crate::{
    document::Document,
    editor::{cursors::CursorState, finder::Haystack},
    ix::{Column, Ix, Line},
    util::MapBounds,
};

impl Document {
    fn full_haystack(&self) -> Haystack {
        Haystack {
            text: self.text().to_string(),
            offset: 0,
        }
    }

    pub fn range_haystack(&self, line: Ix<Line>, columns: Range<Ix<Column>>) -> Option<Haystack> {
        let line_start = self.text().byte_of_line(line)?;
        let line = self.text().line(line)?;
        let range = line
            .column_range_to_byte_range(columns)
            .map_bounds(|r| r + line_start);
        if range.is_empty() {
            return None;
        }

        Some(Haystack {
            text: self.text.byte_slice(range.clone())?.to_string(),
            offset: range.start.inner(),
        })
    }

    pub fn line_haystack(&self, line: Ix<Line>) -> Option<Haystack> {
        Some(Haystack {
            text: self.text.line(line)?.to_string(),
            offset: self.text.byte_of_line(line)?.inner(),
        })
    }

    pub fn find_haystacks(&self) -> Vec<Haystack> {
        let Some(cursors) = &self.cursors else {
            return vec![self.full_haystack()];
        };
        let haystacks = match cursors {
            CursorState::MirrorInsert(_) | CursorState::Insert(_) => vec![self.full_haystack()],
            CursorState::Select(c) => c
                .iter()
                .flat_map(|c| {
                    c.lines()
                        .flat_map(|l| self.range_haystack(c.line, l.start..l.end))
                })
                .collect(),
            CursorState::LineSelect(c) => c
                .iter()
                .flat_map(|c| {
                    (c.line..c.line + c.height - Ix::new(1)).flat_map(|l| self.line_haystack(l))
                })
                .collect(),
        };
        if haystacks.is_empty() {
            vec![self.full_haystack()]
        } else {
            haystacks
        }
    }
}
