use crate::{AppSignal, action::{Action, LineSelectAction}, editor::Editor};


impl Action<&mut Editor> for LineSelectAction {
    fn act(self, ed: &mut Editor) -> Option<AppSignal> {
        match self {
            LineSelectAction::Any(action) => return action.act(ed),
            LineSelectAction::InsertBefore => ed.insert_before(),
            LineSelectAction::InsertAfter => ed.insert_after(),
            LineSelectAction::InsertBeforeLine => ed.insert_before_line(),
            LineSelectAction::InsertAfterLine => ed.insert_after_line(),
            LineSelectAction::MirrorInsertIn => ed.mirror_insert_in(),
            LineSelectAction::MirrorInsertOut => ed.mirror_insert_out(),
            LineSelectAction::ParagraphExtend => ed.incremental_select(),
            LineSelectAction::Select => ed.select(),
            LineSelectAction::TextualSelect => ed.text_select(),
            LineSelectAction::BlockSelect => ed.block_select(),
            LineSelectAction::MoveUp => ed.move_y(-1),
            LineSelectAction::MoveDown => ed.move_y(1),
            LineSelectAction::ExtendUp => ed.extend_up(1),
            LineSelectAction::ExtendDown => ed.extend_down(1),
            LineSelectAction::RetractUp => ed.retract_up(1),
            LineSelectAction::RetractDown => ed.retract_down(1),
        }

        None
    }
}