use crate::{constants::TAB_WIDTH, document::{Change, CursorChange, Document, PosError}, ix::ix, pos::Pos, util::{flip_delimiter, indent_string, is_right_delimiter}};


impl Document {
    pub fn return_change(&self, pos: Pos) -> (Option<Change>, Option<CursorChange>) {
        let indent = self.text.context_indent_inc(pos.line);
        let lf_indent = format!("\n{}", indent_string(indent));
        let byte_pos = match self.text.byte_pos_of_pos(pos) {
            Ok(pos) => pos,
            Err(e) => match e {
                PosError::BadLine { .. } => self.text.byte_len(),
                PosError::BadColumn {
                    byte_of_line,
                    bytes_in_line: len,
                    ..
                } => byte_of_line + len,
            },
        };
        if let Some(g) = self.text.byte_slice(byte_pos..).unwrap().graphemes().next() && !g.is_newline() {
            let g = g.as_str();
            if is_right_delimiter(g) {
                let indent = if let Some(l) = flip_delimiter(g)
                    && let Some(d) = self.text.byte_slice(..byte_pos).unwrap().graphemes().next_back()
                    && d.as_str() == l
                {
                    indent
                } else {
                    indent.saturating_sub(ix(TAB_WIDTH))
                };
                let indent1 = indent + ix(TAB_WIDTH);
                let insert = format!("\n{}\n{}", indent_string(indent1), indent_string(indent));
                return (
                    Some(Change {
                        byte_pos,
                        delete: ix(0),
                        insert,
                    }),
                    CursorChange::insert(pos, &lf_indent, pos.offset(ix(1), indent1)),
                );
            }
        }
        
        (
            Some(Change {
                byte_pos,
                delete: ix(0),
                insert: if self.text.line(pos.line).is_none_or(|l| {
                    l.column_slice(pos.column..)
                        .chars()
                        .all(char::is_whitespace)
                }) {
                    "\n".to_owned()
                } else {
                    lf_indent.clone()
                },
            }),
            CursorChange::insert_end(pos, &lf_indent),
        )
    }
}
