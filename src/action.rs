#![expect(unused)]

use crate::{app::AppSignal, lsp::channel::GotoKind};

mod app;
mod editor;
mod document;
mod insert;
mod select;
mod line_select;
mod navigator;

pub trait Action<O> {
    fn act(self, object: O) -> Option<AppSignal>;
}

#[derive(Copy, Clone)]
pub enum AppAction {
    Quit,
}

#[derive(Copy, Clone)]
pub enum ScrollAction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Copy, Clone)]
pub enum EditorAction {
    OpenFile,
    PreviousFile,
    NextFile,
    Undo,
    Redo,
    Save,
    
    Inspect,
    ViewLog,

    Navigator,

    Lsp(LspAction),
}

impl From<LspAction> for EditorAction {
    fn from(value: LspAction) -> Self {
        Self::Lsp(value)
    }
}

#[derive(Copy, Clone)]
pub enum LspAction {
    Hover,
    CodeActions,
    Rename,
    Goto(GotoKind),
    Refresh,
}

#[derive(Copy, Clone)]
pub enum DocumentAction {
    Scroll(ScrollAction),
    Editor(EditorAction),

    CycleCursorsBack,
    CycleCursorsForward,

    ScrollToMainCursor,
    DropNonMainCursors,

    Jump,
    Find,
}

impl From<ScrollAction> for DocumentAction {
    fn from(value: ScrollAction) -> Self {
        Self::Scroll(value)
    }
}

impl From<EditorAction> for DocumentAction {
    fn from(value: EditorAction) -> Self {
        Self::Editor(value)
    }
}

#[derive(Copy, Clone)]
pub enum InsertAction {
    Document(DocumentAction),
    Select,
    Backspace,
    Return,
    TabIn,
    TabInOrComplete,
    TabOut,

    Paste,
}

impl From<DocumentAction> for InsertAction {
    fn from(value: DocumentAction) -> Self {
        Self::Document(value)
    }
}

impl From<ScrollAction> for InsertAction {
    fn from(value: ScrollAction) -> Self {
        Self::Document(value.into())
    }
}

impl From<EditorAction> for InsertAction {
    fn from(value: EditorAction) -> Self {
        Self::Document(value.into())
    }
}

#[derive(Copy, Clone)]
pub enum AnySelectAction {
    Document(DocumentAction),
    TabIn,
    TabOut,

    SyntaxExtend,

    SplitCursorsByLines,
    CollapseToStart,
    CollapseToEnd,

    FlitForward,
    FlitBackward,

    Delete,
    Cut,
    Copy,
    Paste,
}

impl From<DocumentAction> for AnySelectAction {
    fn from(value: DocumentAction) -> Self {
        Self::Document(value)
    }
}

impl From<ScrollAction> for AnySelectAction {
    fn from(value: ScrollAction) -> Self {
        Self::Document(value.into())
    }
}

#[derive(Copy, Clone)]
pub enum SelectAction {
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

impl From<AnySelectAction> for SelectAction {
    fn from(value: AnySelectAction) -> Self {
        Self::Any(value)
    }
}

impl From<DocumentAction> for SelectAction {
    fn from(value: DocumentAction) -> Self {
        Self::Any(value.into())
    }
}

impl From<ScrollAction> for SelectAction {
    fn from(value: ScrollAction) -> Self {
        Self::Any(value.into())
    }
}

#[derive(Copy, Clone)]
pub enum LineSelectAction {
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

impl From<AnySelectAction> for LineSelectAction {
    fn from(value: AnySelectAction) -> Self {
        Self::Any(value)
    }
}

impl From<DocumentAction> for LineSelectAction {
    fn from(value: DocumentAction) -> Self {
        Self::Any(value.into())
    }
}

impl From<ScrollAction> for LineSelectAction {
    fn from(value: ScrollAction) -> Self {
        Self::Any(value.into())
    }
}

#[derive(Copy, Clone)]
pub enum NavigatorAction {
    Down,
    Up,
    Out,
    In,
    NewChild,
    NewSibling,
    Rename,
    DeleteEmpty,
    Editor,
}
