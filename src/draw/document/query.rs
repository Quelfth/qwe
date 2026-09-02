use crate::{document::Document, ts::QueryCx};

pub macro query_cx($self: ident) {
    crate::ts::QueryCx {
        locals: $self.resolve_locals(),
        semtoks: $self.semtoks.ranges().collect::<crate::range_tree::RangeTree<_, _>>(),
        relevant_lines: $self.screen_line_range(),
    }
}

impl Document {
    pub fn query_cx(&self) -> QueryCx<'_> {
        query_cx!(self)
    }
}