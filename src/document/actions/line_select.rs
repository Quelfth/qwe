use crate::{document::Document, editor::cursors::Cursors, ix::Ix, pos::Pos};


impl Document {
    pub fn insert_newline_above(&mut self) {
        if let Some(cursors) = &self.cursors {
            for i in cursors.indices() {
                let range = self.cursors.as_ref().unwrap().line_range_at(i);
                self.direct_insert(Pos { line: range.start, column: Ix::new(0) }, "\n");
            }
        }
    }

    pub fn insert_newline_below(&mut self) {
        if let Some(cursors) = &self.cursors {
            for i in cursors.indices() {
                let range = self.cursors.as_ref().unwrap().line_range_at(i);
                self.direct_insert(Pos { line: range.end, column: Ix::new(0) }, "\n");
            }
        }
    }
}