use std::iter;

use crate::{
    document::{Change, CursorChange, Document, PosError},
    ix::ix,
    pos::Pos,
    util::LinesColumnsExt as _,
};

impl Document {
    fn insert_change_inner(&self, pos: Pos, text: String) -> Change {
        match self.text.byte_pos_of_pos(pos) {
            Ok(byte_pos) => Change {
                byte_pos,
                delete: ix(0),
                insert: text,
            },
            Err(e) => match e {
                PosError::BadLine { len } => Change {
                    byte_pos: self.text.byte_len(),
                    delete: ix(0),
                    insert: iter::repeat_n("\n", (pos.line - len).inner())
                        .chain(iter::repeat_n(" ", (pos.column).inner()))
                        .chain(iter::once(&*text))
                        .collect(),
                },
                PosError::BadColumn {
                    byte_of_line,
                    bytes_in_line: len,
                    columns_in_line,
                } => Change {
                    byte_pos: byte_of_line + len,
                    delete: ix(0),
                    insert: iter::repeat_n(" ", (pos.column - columns_in_line).inner())
                        .chain(iter::once(&*text))
                        .collect(),
                },
            },
        }
    }

    pub fn insert_change(&self, pos: Pos, text: String) -> (Option<Change>, Option<CursorChange>) {
        let cursor_change = CursorChange::insert_end(pos, &text);
        (
            Some(self.insert_change_inner(pos, text)),
            cursor_change,
        )
    }

    pub fn insert_pair_change(&self, pos: Pos, left: String, right: String) -> (Option<Change>, Option<CursorChange>) {
        let (lines, columns) = left.lines_columns();
        let total = left + &right;
        let cursor_change = CursorChange::insert(pos, &total, pos.offset(lines, columns));
        (
            Some(self.insert_change_inner(pos, total)),
            cursor_change,
        )
    }

    pub fn insert_reluctant_change(&self, pos: Pos, text: String, escape: Option<&str>) -> (Option<Change>, Option<CursorChange>) {
        try {
            let byte_pos = self.text.byte_pos_of_pos(pos).ok()?;
            let slice = self.text.byte_slice(byte_pos..)?;
            let mut ws = 0;

            if let Some(c) = slice.graphemes()
                .inspect(|g| if g.as_str() == " " { ws += 1 })
                .find(|g| g.as_str() != " ")
                && c.as_str() == text
                && escape.is_none_or(|escape| {
                    try {
                        self.text
                            .byte_slice(..byte_pos)?
                            .graphemes()
                            .rev()
                            .take_while(|g| g.as_str() == escape)
                            .count()
                            .is_multiple_of(2)
                    }.unwrap_or_default()
                })
            {
                return (
                    None,
                    CursorChange::insert_end(
                        pos,
                        &iter::repeat_n(" ", ws)
                            .chain(iter::once(&*text))
                            .collect::<String>()
                    )
                )
            }
        };
        let cursor_change = CursorChange::insert_end(pos, &text);
        (
            Some(self.insert_change_inner(pos, text)),
            cursor_change,
        )

        //(None, CursorChange::insert_end(pos, &text))
    }
}
