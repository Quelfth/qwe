use std::{collections::{HashMap, HashSet}, sync::LazyLock};

use arc_swap::ArcSwap;
use mutx::Mutex;

use crate::{action::*, keymap::{KeyMap, keymap}, lang::Language, lsp::channel::GotoKind};

pub static GLOBAL_CONFIG: LazyLock<GlobalConfig> = LazyLock::new(Default::default);

pub struct GlobalConfig {
    pub autosave_langs: Mutex<HashSet<Language>>,
    pub keymaps: Keymaps,
    pub special_chars: Mutex<HashMap<char, CharSpecial>>,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            autosave_langs: Mutex::new(Language::ALL.into_iter().filter(|l| l.autosave()).collect()),
            keymaps: Default::default(),
            special_chars: {
                use CharSpecial::*;
                Mutex::new(HashMap::from_iter([
                    ('(', StrongLeft(')')),
                    ('[', StrongLeft(']')),
                    ('{', StrongLeft('}')),

                    (')', Right),
                    (']', Right),
                    ('}', Right),

                    ('<', WeakLeft('>')),
                    ('>', Right),

                    ('"', WeakPair),
                    ('\'', WeakPair),
                    ('`', WeakPair),
                    ('|', WeakPair),
                    ('/', WeakPair),
                    (' ', WeakPair),

                    ('*', AltInsert),
                    ('+', AltInsert),
                    ('?', AltInsert),
                    (',', AltInsert),
                    (';', AltInsert),
                ]))
            },
        }
    }
}

pub enum CharSpecial {
    StrongLeft(char),
    Right,
    WeakLeft(char),
    WeakPair,
    AltInsert,
}

type Keymap<A> = ArcSwap<KeyMap<A>>;

pub struct Keymaps {
    pub app: Keymap<AppAction>,
    pub mirror_insert: Keymap<InsertAction>,
    pub insert: Keymap<InsertAction>,
    pub select: Keymap<SelectAction>,
    pub line_select: Keymap<LineSelectAction>,
    pub navigator: Keymap<NavigatorAction>,
}

impl Default for Keymaps {
    fn default() -> Self {
        use {
            ScrollAction as Scroll,
            InsertAction as Insert,
            AnySelectAction as Select,
        };
        let scroll = keymap!{
            [ctrl d] => Scroll::Down(4),
            [scroll down] => Scroll::Down(4),
            [ctrl alt d] => Scroll::Down(10),
            [ctrl u] => Scroll::Up(4),
            [scroll up] => Scroll::Up(4),
            [ctrl alt u] => Scroll::Up(10),
            [ctrl r] => Scroll::Right(4),
            [scroll right] => Scroll::Right(4),
            [ctrl alt r] => Scroll::Right(10),
            [ctrl a] => Scroll::Left(4),
            [scroll left] => Scroll::Left(4),
            [ctrl alt a] => Scroll::Left(10),
        };
        let mouse = keymap! {
            [left click] => EditorAction::MouseSelectNew,
            [left drag] => EditorAction::MouseSelectContinue,
            [alt left click] => EditorAction::MouseLineSelectNew,
            [alt left drag] => EditorAction::MouseLineSelectContinue,
        };
        let common_insert = keymap!{
            ..mouse,
            [esc] => Insert::Select,
            [backspace] => Insert::Backspace,
            [alt backspace] => Insert::ReverseBackspace,
            [return] => Insert::Return,
            [tab] => Insert::TabInOrComplete,
            [back tab] => Insert::TabOut,
            [ ] => Insert::Space,
            [ctrl z] => EditorAction::Undo.into(),
            [ctrl v] => Insert::Paste,
        };
        let lsp_select = {
            use LspAction::*;
            keymap! {
                ["'"] => Hover,
                [2] => CodeActions,
                [@] => Rename,
                [*] => Goto(GotoKind::Definition),
                [alt 8] => Goto(GotoKind::Declaration),
                [alt *] => Goto(GotoKind::Implementation),
                [&] => Goto(GotoKind::References),
                [Y] => Goto(GotoKind::TypeDefinition),
                
                [f5] => Refresh,
            }
        };
        let editor_select = {
            use EditorAction::*;
            keymap! {
                ..lsp_select,
                ..mouse,
                [ctrl o] => OpenFile,
                ['('] => PreviousFile,
                [')'] => NextFile,
                [z] => Undo,
                [Z] => Redo,
                [ctrl s] => Save,

                [f3] => ViewLog,
                [f6] => Inspect,

                [n] => Navigator,

                [ctrl c] => SystemCopy,
                ['"'] => ViewDiagnostics,
            }
        };
        let document_select = {
            use DocumentAction::*;
            keymap! {
                ..editor_select,
                [esc] => DropNonMainCursors,
                [9] => CycleCursorsBack,
                [0] => CycleCursorsForward,
                [8] => ScrollToMainCursor,
                [ ] => Jump,
                [f] => Find,
                [F] => FindAll,
                [ctrl l] => LineJump,
                [1] => GotoDiagnostic,
            }
        };
        let select = {use Select::*; keymap!{
            ..document_select,
            [tab] => TabIn,
            [back tab] => TabOut,
            [o] => SyntaxExtend,
            [:] => SplitCursorsByLines,
            [u] => CollapseToStart,
            [q] => CollapseToEnd,
            [alt 9] => FlitBackward,
            [alt 0] => FlitForward,
            [return] => Open,
            [backspace] => Close,
            [X] => Delete,
            [x] => Cut,
            [c] => Copy,
            [v] => Paste,
            [6] => CamelCase,
            [^] => PascalCase,
            [_] => SnakeCase,
            [alt ^] => AdaCase,
            [alt _] => ScreamingSnakeCase,
            [-] => KebabCase,
            [alt 6] => TrainCase,
            [alt -] => CobolCase,
        }};
        Self {
            app: ArcSwap::from_pointee(keymap!{
                [ctrl q] => AppAction::Quit,
            }),
            mirror_insert: ArcSwap::from_pointee(keymap!{
                ..scroll,
                ..common_insert,
            }),
            insert: ArcSwap::from_pointee(keymap!{
                ..scroll,
                ..common_insert,
            }),
            select: {
                use SelectAction::*;
                ArcSwap::from_pointee(keymap!{
                    ..scroll,
                    ..select,
                    [i] => InsertBefore,
                    [a] => InsertAfter,
                    [I] => InsertBeforeLine,
                    [A] => InsertAfterLine,
                    ['['] => MirrorInsertIn,
                    [']'] => MirrorInsertOut,
                    [w] => WordExtend,
                    [;] => LineSelect,
                    [backslash] => TextualSelect,
                    [ | ] => BlockSelect,

                    [h] => MoveLeft,
                    [j] => MoveDown,
                    [k] => MoveUp,
                    [l] => MoveRight,

                    [H] => RetractLeft,
                    [J] => ExtendDown,
                    [K] => RetractUp,
                    [L] => ExtendRight,

                    [alt h] => ExtendLeft,
                    [alt j] => RetractDown,
                    [alt k] => ExtendUp,
                    [alt l] => RetractRight,
                })
            }, 
            line_select: {
                use LineSelectAction::*;
                ArcSwap::from_pointee(keymap!{
                    ..scroll,
                    ..select,

                    [i] => InsertBefore,
                    [a] => InsertAfter,
                    [I] => InsertBeforeLine,
                    [A] => InsertAfterLine,
                    ['['] => MirrorInsertIn,
                    [']'] => MirrorInsertOut,

                    [w] => ParagraphExtend,

                    [;] => Select,
                    [backslash] => TextualSelect,
                    [ | ] => BlockSelect,

                    [j] => MoveDown,
                    [k] => MoveUp,

                    [J] => ExtendDown,
                    [K] => RetractUp,

                    [alt j] => RetractDown,
                    [alt k] => ExtendUp,
                })
            },
            navigator: {
                use NavigatorAction::*;
                ArcSwap::from_pointee(keymap!{
                    [j] => Down,
                    [k] => Up,
                    [h] => Out,
                    [l] => In,
                    [i] => NewChild,
                    [a] => NewSibling,
                    [@] => Rename,
                    [X] => DeleteEmpty,
                    [n] => Editor,
                    [return] => Editor,
                })
            },
        }
    }
}
