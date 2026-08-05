use std::{iter, range::Range};

use crate::{
    constants::TAB_WIDTH,
    document::{Document, force_cursors},
    editor::cursors::{CursorIndex, CursorState, Cursors},
    ix::{Byte, Column, Ix, Line, ix},
    util::{MapBounds as _, RangeLen as _, indent_string, is_right_delimiter},
};

impl Document {
    pub fn insert_before(&mut self) {
        self.timeline.history.checkpoint();
        if let Some(c) = &self.cursors {
            use CursorState::*;
            match c {
                MirrorInsert(_) => todo!(),
                Insert(_) => (),
                Select(cursors) => self.cursors = Some(cursors.to_insert_before().into()),
                LineSelect(cursors) => {
                    self.cursors = Some(cursors.to_insert_before(&self.text).into())
                }
            }
        }
    }
    pub fn insert_after(&mut self) {
        self.timeline.history.checkpoint();
        if let Some(c) = &self.cursors {
            use CursorState::*;
            match c {
                MirrorInsert(_) => todo!(),
                Insert(_) => (),
                Select(cursors) => self.cursors = Some(cursors.to_insert_after().into()),
                LineSelect(cursors) => {
                    self.cursors = Some(cursors.to_insert_after(&self.text).into())
                }
            }
        }
    }
    pub fn insert_before_line(&mut self) {
        self.timeline.history.checkpoint();
        if let Some(c) = &self.cursors {
            use CursorState::*;
            match c {
                MirrorInsert(_) => (),
                Insert(_) => (),
                Select(cursors) => {
                    self.cursors = Some(cursors.to_insert_before_line(&self.text).into())
                }
                LineSelect(cursors) => {
                    self.cursors = Some(cursors.to_insert_before(&self.text).into())
                }
            }
        }
    }
    pub fn insert_after_line(&mut self) {
        self.timeline.history.checkpoint();
        if let Some(c) = &self.cursors {
            use CursorState::*;
            match c {
                MirrorInsert(_) => todo!(),
                Insert(_) => (),
                Select(cursors) => {
                    self.cursors = Some(cursors.to_insert_after_line(&self.text).into())
                }
                LineSelect(cursors) => {
                    self.cursors = Some(cursors.to_insert_after(&self.text).into())
                }
            }
        }
    }
    pub fn line_select(&mut self) {
        if let Some(c) = &self.cursors {
            use CursorState::*;
            match c {
                MirrorInsert(_) => todo!(),
                Insert(c) => self.cursors = Some(c.to_line_select().into()),
                Select(c) => self.cursors = Some(c.to_line_select().into()),
                LineSelect(_) => (),
            }
        }
    }

    pub fn insert_around_in(&mut self) {
        if let Some(c) = &self.cursors {
            use CursorState::*;
            match c {
                MirrorInsert(_) => (),
                Insert(_) => (),
                Select(c) => self.cursors = Some(c.to_mirror_insert_in().into()),
                LineSelect(c) => self.cursors = Some(c.to_insert_around_in(&self.text).into()),
            }
        }
    }
    pub fn insert_around_out(&mut self) {
        if let Some(c) = &self.cursors {
            use CursorState::*;
            match c {
                MirrorInsert(_) => (),
                Insert(_) => (),
                Select(c) => self.cursors = Some(c.to_mirror_insert_out().into()),
                LineSelect(c) => self.cursors = Some(c.to_insert_around_out(&self.text).into()),
            }
        }
    }

    pub fn block_select(&mut self) {
        if let Some(c) = &mut self.cursors {
            use CursorState::*;
            match c {
                MirrorInsert(_) => (),
                Insert(_) => (),
                Select(c) => c.block_select(),
                LineSelect(c) => self.cursors = Some(c.to_block_select(&self.text).into()),
            }
        }
    }

    pub fn text_select(&mut self) {
        if let Some(c) = &mut self.cursors {
            use CursorState::*;
            match c {
                MirrorInsert(_) => (),
                Insert(_) => (),
                Select(c) => c.text_select(&self.text),
                LineSelect(c) => self.cursors = Some(c.to_select(&self.text).into()),
            }
        }
    }

    pub fn move_x(&mut self, columns: Ix<Column, isize>) {
        if let Some(c) = &mut self.cursors {
            c.move_x(columns)
        }
    }

    pub fn move_y(&mut self, rows: Ix<Line, isize>) {
        force_cursors!(self).move_y(rows);
    }

    pub fn extend_up(&mut self, rows: Ix<Line>) {
        force_cursors!(self).extend_up(rows, &self.text)
    }
    pub fn extend_down(&mut self, rows: Ix<Line>) {
        force_cursors!(self).extend_down(rows, &self.text)
    }

    pub fn extend_left(&mut self, columns: Ix<Column>) {
        force_cursors!(self).extend_left(columns);
    }
    pub fn extend_right(&mut self, columns: Ix<Column>) {
        force_cursors!(self).extend_right(columns);
    }
    pub fn retract_up(&mut self, rows: Ix<Line>) {
        force_cursors!(self).retract_up(rows);
    }
    pub fn retract_down(&mut self, rows: Ix<Line>) {
        force_cursors!(self).retract_down(rows);
    }
    pub fn retract_left(&mut self, rows: Ix<Column>) {
        force_cursors!(self).retract_left(rows);
    }
    pub fn retract_right(&mut self, rows: Ix<Column>) {
        force_cursors!(self).retract_right(rows);
    }

    pub fn drop_other_selections(&mut self) {
        force_cursors!(self).drop_others();
    }

    pub fn syntax_extend(&mut self) {
        if let Some(tree) = &self.tree
            && let Some(c) = &mut self.cursors {
                c.syntax_extend(&self.text, &tree.tree)
            }
    }

    pub fn open_lines(&mut self) {
        self.timeline.history.checkpoint();
        for index in self.cursor_indices() {
            try {
                let range = self.text.region_to_byte_range(self.cursor_convex_range(index)?);
                let tree = &self.tree.as_ref()?.tree;
                let node = tree.root_node().named_descendant_for_byte_range(range.start.0, range.end.0)?;
                let mut ws_ranges = Vec::new();
                for child in node.named_children(&mut node.walk()) {
                    let end: Ix<Byte> = ix(child.byte_range().start);
                    let mut start = end;
                    let mut newlines: usize = 0;
                    for char in self.text.byte_slice(..end)?.chars().rev() {
                        if !char.is_whitespace() {break}
                        if char == '\n' {
                            newlines += 1;
                        }
                        start -= ix(char.len_utf8());
                    }
                    ws_ranges.push((start..end, newlines));
                }
                let start_line = self.text.line_of_byte(ix(node.byte_range().start))?;
                let indent_amount = self.text.indent_on_line(start_line);
                let mut close_range = None;
                if let Some(closer) = node.children(&mut node.walk()).last()
                    && !closer.is_named()
                    && let Some(slice) = self.text.byte_slice(Range::from(closer.byte_range()).map_bounds(ix))
                    && is_right_delimiter(&slice.to_string())
                {
                    let end: Ix<Byte> = ix(closer.byte_range().start);
                    let mut start = end;
                    let mut newlines: usize = 0;
                    for char in self.text.byte_slice(..end)?.chars().rev() {
                        if !char.is_whitespace() {break}
                        if char == '\n' {
                            newlines += 1;
                        }
                        start -= ix(char.len_utf8());
                    }
                    close_range = Some((start..end, newlines));
                };
                if let Some((range, newlines)) = close_range {
                    let newlines = iter::repeat_n("\n", newlines.max(1)).collect::<String>();
                    self.direct_replace_byte(range, &format!("{newlines}{}", indent_string(indent_amount * TAB_WIDTH)))
                }

                let indent = indent_string((indent_amount + ix(1)) * TAB_WIDTH);
                for (range, newlines) in ws_ranges.into_iter().rev() {
                    let newlines = iter::repeat_n("\n", newlines.max(1)).collect::<String>();
                    self.direct_replace_byte(range, &format!("{newlines}{indent}"))
                }
            };
        }
    }

    pub fn close_lines(&mut self) {
        self.timeline.history.checkpoint();
        for index in self.cursor_indices() {
            let Some(range) = self.cursor_line_range(index) else {continue};
            if range.len() < ix(2) {continue}
            for line in (range.start + ix(1)..range.end).into_iter().rev() {
                try {
                    let byte = self.text.byte_of_line(line)?;
                    let mut range = byte - ix(1)..byte;
                    for char in self.text.byte_slice(..range.start)?.chars().rev() {
                        if char.is_whitespace() {
                            range.start -= ix(char.len_utf8());
                        } else {
                            break
                        }
                    }
                    for char in self.text.byte_slice(range.end..)?.chars() {
                        if char.is_whitespace() {
                            range.end += ix(char.len_utf8());
                        } else {
                            break
                        }
                    }
                    self.delete(range);
                    self.direct_insert(self.text.pos_of_byte_pos(range.start)?, " ");
                };
            }
        }
    }

    pub fn flit_forward(&mut self) {
        let Some(cursors) = &self.cursors else {return};
        if cursors.len() < 2 {return};
        let last_index = cursors.last_index();

        self.timeline.history.checkpoint();
        let main_text = self.cut_from_cursor(CursorIndex::Main).unwrap();
        let other_text = self.cut_from_cursor(CursorIndex::Other(0)).unwrap();

        self.cursors.as_mut().unwrap().cycle_forward();

        self.paste_at_cursor(main_text, CursorIndex::Main);
        self.paste_at_cursor(other_text, last_index);
    }

    pub fn flit_backward(&mut self) {
        let Some(cursors) = &self.cursors else {return};
        if cursors.len() < 2 {return};
        let last_index = cursors.last_index();

        self.timeline.history.checkpoint();
        let main_text = self.cut_from_cursor(CursorIndex::Main).unwrap();
        let other_text = self.cut_from_cursor(last_index).unwrap();

        self.cursors.as_mut().unwrap().cycle_backward();

        self.paste_at_cursor(main_text, CursorIndex::Main);
        self.paste_at_cursor(other_text, CursorIndex::Other(0));
    }
}
