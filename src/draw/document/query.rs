use crate::{
    document::Document, range_tree::RangeTree, ts::QueryCx
};


impl Document {
    pub fn query_capture_context(&self) -> QueryCx<'_> {
        QueryCx {
            semtoks: self.semtoks.ranges().collect::<RangeTree<_, _>>(),
            screen_lines: self.screen_line_range(),
        }
    }
}
