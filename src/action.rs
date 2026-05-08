
enum ScrollAction {
    Up,
    Down,
    Left,
    Right,
}

enum DocumentAction {
    Undo,

    CycleCursorsBack,
    CycleCursorsForward,

    ScrollToMainCursor,
    DropNonMainCursors,
}

enum InsertAction {
    Select,
    Backspace,
    Return,
    TabInOrComplete,
    TabOut,
    Paste,
}

enum SelectAction {
    InsertBefore,
    InsertAfter,
    InsertBeforeLine,
    InsertAfterLine,
    MirrorInsertIn,
    MirrorInsertOut,
    TabIn,
    TabOut,

    SyntaxExtend,
    WordExtend,

    LineSelect,
    SplitCursorsAcrossLines,
    TextualSelect,
    BlockSelect,
    CollapseToStart,
    CollapseToEnd,

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