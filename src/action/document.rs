use crate::{AppSignal, action::{Action, DocumentAction, ScrollAction}, document::Document, editor::Editor};

impl Action<&mut Editor> for ScrollAction {
    fn act(self, ed: &mut Editor) -> Option<AppSignal> {
        match self {
            ScrollAction::Up => ed.scroll_up(4),
            ScrollAction::Down => ed.scroll_down(4),
            ScrollAction::Left => ed.scroll_left(4),
            ScrollAction::Right => ed.scroll_right(4),
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
            DocumentAction::Find => ed.find(),
        }

        None
    }
}