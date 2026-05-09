#![expect(unused)]

use crate::lsp::channel::GotoKind;

enum ScrollAction {
    Up,
    Down,
    Left,
    Right,
}

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

enum LspAction {
    Hover,
    CodeActions,
    Rename,
    Goto(GotoKind),
    Refresh,
}

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

enum InsertAction {
    Select,
    Backspace,
    Return,
    TabInOrComplete,
    TabOut,
    Paste,
}

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