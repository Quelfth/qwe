use crate::{
    app::AppSignal,
    action::{Action, InsertAction},
    document::Document, editor::Editor,
};


impl Action<&mut Editor> for InsertAction {
    fn act(self, ed: &mut Editor) -> Option<AppSignal> {
        match self {
            InsertAction::Document(action) => return action.act(ed),
            InsertAction::Select => ed.select(),
            InsertAction::Backspace => ed.backspace(),
            InsertAction::ReverseBackspace => ed.doc_mut().reverse_backspace(),
            InsertAction::Return => ed.insert_return(),
            InsertAction::TabIn => ed.tab_lines_in(),
            InsertAction::TabInOrComplete => ed.insert_tab_else_complete(),
            InsertAction::TabOut => ed.tab_lines_out(),
            InsertAction::Space => ed.insert_space(),
            InsertAction::Paste => ed.paste(),
        }

        None
    }
}