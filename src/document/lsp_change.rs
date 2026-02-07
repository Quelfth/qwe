use lsp_types::{Position, Range, TextDocumentContentChangeEvent};

use crate::pos::Utf16Pos;

pub struct LspChange {
    pub start: Utf16Pos,
    pub end: Utf16Pos,
    pub text: String,
}

impl From<LspChange> for TextDocumentContentChangeEvent {
    fn from(value: LspChange) -> Self {
        let LspChange { start, end, text } = value;
        TextDocumentContentChangeEvent {
            range: Some(Range {
                start: Position {
                    line: start.line.inner() as u32,
                    character: start.column.inner() as u32,
                },
                end: Position {
                    line: end.line.inner() as u32,
                    character: end.column.inner() as u32,
                },
            }),
            range_length: None,
            text,
        }
    }
}
