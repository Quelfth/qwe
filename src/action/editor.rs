use crate::{AppSignal, action::{Action, EditorAction}, editor::Editor};

use super::LspAction;


impl Action<&mut Editor> for EditorAction {
    fn act(self, ed: &mut Editor) -> Option<AppSignal> {
        match self {
            EditorAction::OpenFile => ed.pick_file(),
            EditorAction::PreviousFile => ed.previous_file(),
            EditorAction::NextFile => ed.next_file(),
            EditorAction::Undo => ed.undo(),
            EditorAction::Redo => ed.redo(),
            EditorAction::Save => ed.save_file(),
            EditorAction::Inspect => ed.inspect(),
            EditorAction::ViewLog => ed.view_log(),
            EditorAction::Navigator => return Some(AppSignal::Navigator),
            EditorAction::Lsp(action) => match action {
                LspAction::Hover => ed.hover(),
                LspAction::CodeActions => ed.code_actions(),
                LspAction::Rename => ed.rename(),
                LspAction::Goto(kind) => ed.goto(kind),
                LspAction::Refresh => ed.refresh_lsp(),
            }
        }

        None
    }
}