use crate::{
    constants::TAB_WIDTH,
    document::{Change, CursorChange, Document},
    editor::cursors::{CursorState, mirror_insert::InsertDirection},
    ix::Ix,
    pos::Pos,
    util::{flip_delimiter, indent_string, is_right_delimiter, mirror_string},
};

impl Document {
    pub fn select(&mut self) {
        let Some(cursors) = &self.cursors else {
            return;
        };
        use CursorState::*;
        match cursors {
            MirrorInsert(c) => self.cursors = Some(c.to_select(&self.text).into()),
            Insert(c) => self.cursors = Some(c.to_select().into()),
            Select(_) => (),
            LineSelect(c) => self.cursors = Some(c.to_select(&self.text).into()),
        }
    }

    pub fn backspace(&mut self) {
        self.do_insert(|doc, pos, dir| match dir {
            InsertDirection::Forward => doc.backspace_change(pos),
            InsertDirection::Reverse => doc.reverse_backspace_change(pos),
        })
    }

    pub fn insert(&mut self, str: &str) {
        self.do_insert(insert_effect(str))
    }

    pub fn insert_pair(&mut self, left: &str, right: &str) {
        self.do_insert(|doc, pos, _| doc.insert_pair_change(pos, left.to_owned(), right.to_owned()))
    }

    pub fn insert_reluctant(&mut self, str: &str) {
        self.do_insert(|doc, pos, _| doc.insert_reluctant_change(pos, str.to_owned()))
    }

    pub fn insert_space(&mut self) {
        self.do_insert(|doc, pos, _| {
            if let Ok(byte_pos) = doc.text.byte_pos_of_pos(pos)
                && let Some(g) = doc.text.byte_slice(byte_pos..).unwrap().graphemes().next()
                && is_right_delimiter(g.as_str())
                && let Some(l) = doc.text.byte_slice(..byte_pos).unwrap().graphemes().next_back()
                && let Some(d) = flip_delimiter(g.as_str())
                && d == l.as_str()
            {
                return doc.insert_pair_change(pos, " ".to_owned(), " ".to_owned());
            }
            doc.insert_change(pos, " ".to_owned())
        })
    }

    pub fn insert_return(&mut self) {
        self.do_insert(|doc, pos, _| doc.return_change(pos))
    }

    pub fn insert_tab(&mut self) -> Result<(), ()> {
        let mut success = false;
        if let Some(c) = &self.cursors {
            for i in c.indices() {
                match self.cursors.as_ref().unwrap() {
                    CursorState::Insert(c) => {
                        let cursor = c[i];
                        let line = self.text.line(cursor.pos.line);
                        if line.is_none_or(|l| l.chars().all(char::is_whitespace)) {
                            self.cursors.as_mut().unwrap().assume_insert_mut()[i].tab();
                            success = true;
                        } else if {
                            let line = line.unwrap();

                            line.byte_slice(..line.columns_to_bytes(cursor.pos.column))
                                .is_none_or(|x| x.chars().all(char::is_whitespace))
                        } {
                            let indent = self.text.indent_on_line(cursor.pos.line);
                            let rem = indent % TAB_WIDTH;
                            let ind = Ix::new(TAB_WIDTH) - rem;

                            self.do_insert_at_index(i, insert_effect(&indent_string(ind)));
                            success = true;
                        }
                    }
                    _ => todo!(),
                }
            }
        }

        if success { Ok(()) } else { Err(()) }
    }

    pub fn tab_out(&mut self) {
        self.do_insert(|doc, pos, dir| match dir {
            InsertDirection::Forward => doc.tab_out_change(pos),
            InsertDirection::Reverse => (None, None),
        })
    }

    pub fn direct_insert(&mut self, pos: Pos, text: &str) {
        self.do_change(self.insert_change(pos, text.to_owned()))
    }

    pub fn insert_completion(&mut self, pos: Pos, text: &str) {
        let mut to_insert = text;
        for i in 0..text.len() {
            let Ok(byte_pos) = self.text.byte_pos_of_pos(pos) else {
                continue;
            };
            let Some(prefix) = text.get(0..text.len() - i) else {
                continue;
            };
            if i > byte_pos.inner() {
                continue;
            }
            if self
                .text
                .byte_slice(byte_pos - Ix::new(text.len() - i)..byte_pos)
                .is_some_and(|slice| slice.to_string() == prefix)
            {
                to_insert = &text[text.len() - i..];
                break;
            }
        }
        self.direct_insert(pos, to_insert);
    }
}

fn insert_effect(
    str: &str,
) -> impl Fn(&Document, Pos, InsertDirection) -> (Option<Change>, Option<CursorChange>) {
    |doc, pos, dir| {
        let text = match dir {
            InsertDirection::Forward => str.to_owned(),
            InsertDirection::Reverse => mirror_string(str),
        };
        doc.insert_change(pos, text)
    }
}

