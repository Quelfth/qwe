use crate::{action::{Action, DocumentAction, ScrollAction}, app::AppSignal, document::Document, editor::{Editor, line_jumper::LineJumper}};

impl Action<&mut Editor> for ScrollAction {
    fn act(self, ed: &mut Editor) -> Option<AppSignal> {
        match self {
            ScrollAction::Up(amount) => ed.scroll_up(amount),
            ScrollAction::Down(amount) => ed.scroll_down(amount),
            ScrollAction::Left(amount) => ed.scroll_left(amount),
            ScrollAction::Right(amount) => ed.scroll_right(amount),
        }

        None
    }
}

impl Action<&mut Editor> for DocumentAction {
    fn act(self, ed: &mut Editor) -> Option<AppSignal> {
        match self {
            DocumentAction::Scroll(action) => return action.act(ed),
            DocumentAction::Editor(action) => return action.act(ed),
            DocumentAction::CycleCursorsBack => ed.cycle_cursors_backward(),
            DocumentAction::CycleCursorsForward => ed.cycle_cursors_forward(),
            DocumentAction::ScrollToMainCursor => ed.scroll_main_cursor_on_screen(),
            DocumentAction::DropNonMainCursors => ed.drop_other_selections(),
            DocumentAction::Jump => ed.jump(),
            DocumentAction::Find => ed.find_in(),
            DocumentAction::FindAll => ed.find_all(),
            DocumentAction::LineJump => ed.open_gadget(LineJumper::new()),
            DocumentAction::GotoDiagnostic => ed.doc_mut().goto_diagnostic(),
        }

        None
    }
}