use crate::{action::{Action, SelectAction}, app::AppSignal, editor::Editor, util::Case};

use super::AnySelectAction;

impl Action<&mut Editor> for AnySelectAction {
    fn act(self, ed: &mut Editor) -> Option<AppSignal> {
        use AnySelectAction::*;
        match self {
            Document(action) => return action.act(ed),
            TabIn => ed.tab_lines_in(),
            TabOut => ed.tab_lines_out(),
            SyntaxExtend => ed.syntax_extend(),
            SplitCursorsByLines => ed.cursor_line_split(),
            CollapseToStart => ed.collapse_cursors_to_start(),
            CollapseToEnd => ed.collapse_cursors_to_end(),
            FlitForward => ed.doc_mut().flit_forward(),
            FlitBackward => ed.doc_mut().flit_backward(),
            Open => ed.doc_mut().open_lines(),
            Close => ed.doc_mut().close_lines(),
            Delete => ed.delete(),
            Cut => ed.cut(),
            Copy => ed.copy(),
            Paste => ed.paste(),
            CamelCase => ed.doc_mut().apply_case(Case::Camel),
            PascalCase => ed.doc_mut().apply_case(Case::Pascal),
            SnakeCase => ed.doc_mut().apply_case(Case::Snake),
            AdaCase => ed.doc_mut().apply_case(Case::Ada),
            ScreamingSnakeCase => ed.doc_mut().apply_case(Case::ScreamingSnake),
            KebabCase => ed.doc_mut().apply_case(Case::Kebab),
            TrainCase => ed.doc_mut().apply_case(Case::Train),
            CobolCase => ed.doc_mut().apply_case(Case::Cobol),
        }

        None
    }
}


impl Action<&mut Editor> for SelectAction {
    fn act(self, ed: &mut Editor) -> Option<AppSignal> {
        use SelectAction::*;
        match self {
            Any(action) => return action.act(ed),
            InsertBefore => ed.insert_before(),
            InsertAfter => ed.insert_after(),
            InsertBeforeLine => ed.insert_before_line(),
            InsertAfterLine => ed.insert_after_line(),
            MirrorInsertIn => ed.mirror_insert_in(),
            MirrorInsertOut => ed.mirror_insert_out(),
            WordExtend => ed.incremental_select(),
            LineSelect => ed.line_select(),
            TextualSelect => ed.text_select(),
            BlockSelect => ed.block_select(),
            MoveUp => ed.move_y(-1),
            MoveDown => ed.move_y(1),
            MoveLeft => ed.move_x(-1),
            MoveRight => ed.move_x(1),
            ExtendUp => ed.extend_up(1),
            ExtendDown => ed.extend_down(1),
            ExtendLeft => ed.extend_left(1),
            ExtendRight => ed.extend_right(1),
            RetractUp => ed.retract_up(1),
            RetractDown => ed.retract_down(1),
            RetractLeft => ed.retract_left(1),
            RetractRight => ed.retract_right(1),
        }

        None
    }
}