use std::{iter, range::Range};

use crate::{
    constants::TAB_WIDTH, document::{Document, force_cursors}, editor::cursors::{CursorIndex, CursorState, Cursors, mirror_insert::{MirrorInsertCursor, MirrorInsertCursors}}, ix::{Byte, Column, Ix, Line, ix}, pos::Pos, util::{Case, MapBounds as _, RangeLen as _, indent_string, is_right_delimiter}
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
                LineSelect(_) => self.insert_around_lines(false),
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
                LineSelect(_) => self.insert_around_lines(true),
            }
        }
    }

    fn insert_around_lines(&mut self, out: bool) {
        self.timeline.history.checkpoint();
        let mut new_cursors = Vec::new();
        for i in self.cursor_indices() {
            let Some(range) = self.cursor_line_range(i) else {continue};
            let mut indent = None;
            for line in range {
                let line_indent = self.text.indent_on_line(line);
                if indent.is_none_or(|indent| line_indent > indent) {
                    indent = Some(line_indent);
                }
                self.tab_line_in(line);
            }
            let indent = indent.unwrap_or(ix(0));
            let indent_nl = format!("{}\n", indent_string(indent));

            self.direct_insert(Pos { line: range.end, column: ix(0) }, &indent_nl);
            self.direct_insert(Pos { line: range.start, column: ix(0) }, &indent_nl);
            let forward = Pos { line: range.start, column: indent };
            let reverse = Pos { line: range.end + ix(1), column: indent };

            new_cursors.push(MirrorInsertCursor { forward, reverse }.flip_if(out));
        }
        self.cursors = MirrorInsertCursors::from_iter(new_cursors).map(Into::into);
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
                    let pos_range = self.text.pos_of_byte_pos(start)?..self.text.pos_of_byte_pos(end)?;
                    if !self.cursor_is_tangent(index, pos_range) {continue}
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
                    let pos_range = self.text.pos_of_byte_pos(start)?..self.text.pos_of_byte_pos(end)?;
                    close_range = self.cursor_is_tangent(index, pos_range).then_some((start..end, newlines));
                };
                if let Some((range, newlines)) = close_range {
                    let newlines = iter::repeat_n("\n", newlines.max(1)).collect::<String>();
                    self.direct_replace_byte(range, &format!("{newlines}{}", indent_string(indent_amount)))
                }

                let indent = indent_string(indent_amount + ix(TAB_WIDTH));
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

    pub fn apply_case(&mut self, case: Case) {
        #[derive(Copy, Clone)]
        enum WordPosition {
            Partial,
            Initial,
            Subsequent,
        }

        struct Word {
            position: WordPosition,
            range: Range<Ix<Byte>>,
        }

        let mut words = Vec::<Word>::new();

        fn is_word_char(char: char) -> bool {
            char.is_alphanumeric() || matches!(char, '-' | '_')
        }

        fn word_boundary(before: Option<char>, current: char, next: Option<char>) -> Option<WordPosition> {
            use WordPosition::*;
            if !is_word_char(current) { return None }

            let Some(before) = before else {return Some(Initial)};
            if !is_word_char(before) { return Some(Initial) }

            if matches!(current, '-' | '_') {
                return Some(Subsequent)
            }

            if current.is_uppercase()
                && (!before.is_uppercase() || !next.is_some_and(char::is_uppercase)) {
                    return Some(Subsequent)
                }

            Some(Partial)
        }

        fn is_word_boundary(before: Option<char>, current: char, next: Option<char>) -> bool {
            !matches!(word_boundary(before, current, next), Some(WordPosition::Partial))
        }

        for index in self.cursor_indices() {
            for range in self.cursor_selection_ranges(index) {
                let s = range.start;
                let mut char_before = try {self.text.byte_slice(..s)?.chars().next_back()?};
                let mut i: Ix<Byte> = ix(0);
                let mut j: Ix<Byte> = ix(0);
                let Some(text) = self.text.byte_slice(range) else {continue};

                let Some(char) = text.chars().next() else {continue};
                j += ix(char.len_utf8());

                let mut next_char = text.byte_slice(j..).and_then(|s| s.chars().next());
                if let Some(position) = word_boundary(char_before, char, next_char) {
                    char_before = Some(char);
                    while let Some(char) = next_char {
                        j += ix(char.len_utf8());
                        next_char = text.byte_slice(j..).and_then(|s| s.chars().next());
                        if is_word_boundary(char_before.replace(char), char, next_char) {
                            break;
                        }
                    }
                    words.push(Word { position, range: s + i..s + j });
                    i = j;
                }

                let mut position = None::<WordPosition>;

                while let Some(char) = next_char {
                    j += ix(char.len_utf8());
                    next_char = text.byte_slice(j..).and_then(|s| s.chars().next());
                    if let Some(new_position) = word_boundary(char_before.replace(char), char, next_char) {
                        match new_position {
                            WordPosition::Partial => continue,
                            new_position => {
                                if let Some(position) = position.replace(new_position) {
                                    words.push(Word { position, range: s + i..s + j })
                                }
                            }
                        }
                    } else {
                        position = None;
                        i = j;
                        continue;
                    }
                }
            }
        }

        words.sort_by_key(|w| w.range.start);

        for Word { position, range } in words.into_iter().rev() {
            let Some(word) = self.text.byte_slice(range) else {continue};
            let word = word.to_string();
            use Case::*;
            let word = match position {
                WordPosition::Partial => {
                    match case {
                        Snake | Kebab | Camel | Pascal => word.to_lowercase(),
                        Ada | Train => if matches!(self.text.byte_slice(..range.start).and_then(|slice| slice.chars().next_back()), Some('-' | '_')) {
                            let i = word.ceil_char_boundary(1);
                            format!("{}{}", word[..i].to_uppercase(), word[i..].to_lowercase())
                        } else {
                            word.to_lowercase()
                        },
                        ScreamingSnake | Cobol => word.to_uppercase(),
                    }
                },
                WordPosition::Initial => {
                    match case {
                        Camel | Snake | Kebab => word.to_lowercase(),
                        Pascal | Ada | Train => {
                            let i = word.ceil_char_boundary(1);
                            format!("{}{}", word[..i].to_uppercase(), word[i..].to_lowercase())
                        },
                        ScreamingSnake | Cobol => word.to_uppercase(),
                    }
                },
                WordPosition::Subsequent => {
                    let word = word.strip_prefix('-').or_else(|| word.strip_prefix('_')).unwrap_or(&word);

                    let word = match case {
                        Snake | Kebab => word.to_lowercase(),
                        Camel | Pascal | Ada | Train => {
                            let i = word.ceil_char_boundary(1);
                            format!("{}{}", word[..i].to_uppercase(), word[i..].to_lowercase())
                        },
                        ScreamingSnake | Cobol => word.to_uppercase(),
                    };

                    match case {
                        Camel | Pascal => word,
                        Snake | Ada | ScreamingSnake => format!("_{word}"),
                        Kebab | Train | Cobol => format!("-{word}"),
                    }
                },
            };

            self.direct_replace_byte(range, &word);
        }
    }
}
