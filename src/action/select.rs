use crate::{AppSignal, action::{Action, SelectAction}, editor::Editor};

use super::AnySelectAction;

impl Action<&mut Editor> for AnySelectAction {
    fn act(self, ed: &mut Editor) -> Option<AppSignal> {
        match self {
            AnySelectAction::Document(action) => return action.act(ed),
            AnySelectAction::TabIn => _= ed.tab_lines_in(),
            AnySelectAction::TabOut => ed.tab_lines_out(),
            AnySelectAction::SyntaxExtend => ed.syntax_extend(),
            AnySelectAction::SplitCursorsByLines => ed.cursor_line_split(),
            AnySelectAction::CollapseToStart => ed.collapse_cursors_to_start(),
            AnySelectAction::CollapseToEnd => ed.collapse_cursors_to_end(),
            AnySelectAction::Delete => ed.delete(),
            AnySelectAction::Cut => ed.cut(),
            AnySelectAction::Copy => ed.copy(),
            AnySelectAction::Paste => ed.paste(),
        }

        None
    }
}


impl Action<&mut Editor> for SelectAction {
    fn act(self, ed: &mut Editor) -> Option<AppSignal> {
        match self {
            SelectAction::Any(action) => return action.act(ed),
            SelectAction::InsertBefore => ed.insert_before(),
            SelectAction::InsertAfter => ed.insert_after(),
            SelectAction::InsertBeforeLine => ed.insert_before_line(),
            SelectAction::InsertAfterLine => ed.insert_after_line(),
            SelectAction::MirrorInsertIn => ed.mirror_insert_in(),
            SelectAction::MirrorInsertOut => ed.mirror_insert_out(),
            SelectAction::WordExtend => ed.incremental_select(),
            SelectAction::LineSelect => ed.line_select(),
            SelectAction::TextualSelect => ed.text_select(),
            SelectAction::BlockSelect => ed.block_select(),
            SelectAction::MoveUp => ed.move_y(-1),
            SelectAction::MoveDown => ed.move_y(1),
            SelectAction::MoveLeft => ed.move_x(-1),
            SelectAction::MoveRight => ed.move_x(1),
            SelectAction::ExtendUp => ed.extend_up(1),
            SelectAction::ExtendDown => ed.extend_down(1),
            SelectAction::ExtendLeft => ed.extend_left(1),
            SelectAction::ExtendRight => ed.extend_right(1),
            SelectAction::RetractUp => ed.retract_up(1),
            SelectAction::RetractDown => ed.retract_down(1),
            SelectAction::RetractLeft => ed.retract_left(1),
            SelectAction::RetractRight => ed.retract_right(1),
        }

        None
    }
}