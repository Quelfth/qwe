use crate::{incremental_select::increment_range, rope::Rope};

use super::SelectCursor;

impl SelectCursor {
    pub fn incremental_select(&mut self, text: &Rope) {
        let Ok(start) = text.byte_pos_of_pos(self.start_pos()) else {
            return;
        };
        let Ok(end) = text.byte_pos_of_pos(self.end_pos()) else {
            return;
        };

        let range = increment_range(text, start..end);

        *self = Self::byte_range(range, text);
    }
}
