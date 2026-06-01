#![expect(unused)]

use crate::lsp::channel::GotoKind;

mod select;

pub trait Action<O> {
    fn act(self, object: O);
}

#[derive(Copy, Clone)]
enum ScrollAction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Copy, Clone)]
enum EditorAction {
    OpenFile,
    PreviousFile,
    NextFile,
    Undo,
    Redo,
    Save,
    
    Inspect,
    ViewLog,
}

#[derive(Copy, Clone)]
enum LspAction {
    Hover,
    CodeActions,
    Rename,
    Goto(GotoKind),
    Refresh,
}

#[derive(Copy, Clone)]
enum DocumentAction {
    Scroll(ScrollAction),
    Undo,

    CycleCursorsBack,
    CycleCursorsForward,

    ScrollToMainCursor,
    DropNonMainCursors,

    Jump,
    Find,
}

#[derive(Copy, Clone)]
enum InsertAction {
    Select,
    Backspace,
    Return,
    TabInOrComplete,
    TabOut,
    Paste,
}

#[derive(Copy, Clone)]
enum AnySelectAction {
    TabIn,
    TabOut,

    SyntaxExtend,

    SplitCursorsByLines,
    CollapseToStart,
    CollapseToEnd,

    Delete,
    Cut,
    Copy,
    Paste,
}

#[derive(Copy, Clone)]
enum SelectAction {
    Any(AnySelectAction),
    InsertBefore,
    InsertAfter,
    InsertBeforeLine,
    InsertAfterLine,
    MirrorInsertIn,
    MirrorInsertOut,

    WordExtend,

    LineSelect,
    TextualSelect,
    BlockSelect,

    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,

    ExtendUp,
    ExtendDown,
    ExtendLeft,
    ExtendRight,

    RetractUp,
    RetractDown,
    RetractLeft,
    RetractRight,
}

#[derive(Copy, Clone)]
enum LineSelectAction {
    Any(AnySelectAction),
    InsertBefore,
    InsertAfter,
    InsertBeforeLine,
    InsertAfterLine,
    MirrorInsertIn,
    MirrorInsertOut,

    ParagraphExtend,

    Select,
    TextualSelect,
    BlockSelect,

    MoveUp,
    MoveDown,

    ExtendUp,
    ExtendDown,

    RetractUp,
    RetractDown,
}