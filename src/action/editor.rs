use crate::{app::AppSignal, action::{Action, EditorAction}, editor::Editor};

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
            EditorAction::SystemCopy => ed.system_copy(),
            EditorAction::ViewDiagnostics => ed.view_diagnostics(),
            EditorAction::Lsp(action) => match action {
                LspAction::Hover => ed.hover(),
                LspAction::CodeActions => ed.code_actions(),
                LspAction::Rename => ed.rename(),
                LspAction::Goto(kind) => ed.goto(kind),
                LspAction::Refresh => ed.refresh_lsp(),
            }
            EditorAction::MouseSelectNew => ed.mouse_select_new(),
            EditorAction::MouseSelectContinue => ed.mouse_select_continue(),
            EditorAction::MouseLineSelectNew => ed.mouse_line_select_new(),
            EditorAction::MouseLineSelectContinue => ed.mouse_line_select_continue(),
        }

        None
    }
}
