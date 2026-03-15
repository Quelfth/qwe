use std::sync::LazyLock;

use crossterm::event::KeyCode::*;

use crate::{
    editor::{
        Editor,
        keymap::{Key, Keymap, Keymaps, Mapping},
    },
    lsp::channel::GotoKind::*,
};

static UNIVERSAL: LazyLock<&'static [(Key, Mapping)]> = LazyLock::new(|| {
    Box::leak(Box::new({
        [
            (Key::ctrl('d'), Mapping::rep(|e| e.scroll_down(4))),
            (Key::ctrl('u'), Mapping::rep(|e| e.scroll_up(4))),
            (Key::ctrl('y'), Mapping::once(Editor::debug_undo)),
        ]
    }))
});

fn universal() -> impl Iterator<Item = (Key, Mapping)> {
    UNIVERSAL.iter().copied()
}

impl Default for Keymaps {
    fn default() -> Self {
        Self {
            insert: Keymap::from_iter(universal().chain([
                (Key::code(Esc), Mapping::once(Editor::select)),
                (Key::code(Backspace), Mapping::rep(Editor::backspace)),
                (Key::code(Enter), Mapping::rep(Editor::insert_return)),
                (
                    Key::code(Tab),
                    Mapping::rep(Editor::insert_tab_else_complete),
                ),
                (Key::code(BackTab), Mapping::rep(Editor::tab_out)),
                (Key::ctrl('z'), Mapping::once(Editor::undo)),
                (Key::ctrl('v'), Mapping::once(Editor::paste)),
            ])),
            select: Keymap::from_iter(universal().chain([
                (Key::char('i'), Mapping::once(Editor::insert_before)),
                (Key::char('a'), Mapping::once(Editor::insert_after)),
                (Key::char('I'), Mapping::once(Editor::insert_before_line)),
                (Key::char('A'), Mapping::once(Editor::insert_after_line)),
                (Key::char('['), Mapping::once(Editor::mirror_insert_in)),
                (Key::char(']'), Mapping::once(Editor::mirror_insert_out)),
                //
                (Key::code(Tab), Mapping::rep(Editor::tab_lines_in)),
                (Key::code(BackTab), Mapping::rep(Editor::tab_lines_out)),
                //
                (Key::char('o'), Mapping::once(Editor::incremental_select)),
                (Key::char(';'), Mapping::once(Editor::line_select)),
                (Key::char(':'), Mapping::once(Editor::cursor_line_split)),
                (Key::code(Esc), Mapping::once(Editor::drop_other_selections)),
                (Key::char('9'), Mapping::rep(Editor::cycle_cursors_backward)),
                (Key::char('0'), Mapping::rep(Editor::cycle_cursors_forward)),
                (Key::char('8'), Mapping::once(Editor::scroll_to_main_cursor)),
                //
                (Key::char('h'), Mapping::rep(|e| e.move_x(-1))),
                (Key::char('j'), Mapping::rep(|e| e.move_y(1))),
                (Key::char('k'), Mapping::rep(|e| e.move_y(-1))),
                (Key::char('l'), Mapping::rep(|e| e.move_x(1))),
                (Key::char('H'), Mapping::rep(|e| e.retract_left(1))),
                (Key::char('J'), Mapping::rep(|e| e.text_extend_down(1))),
                (Key::char('K'), Mapping::rep(|e| e.retract_up(1))),
                (Key::char('L'), Mapping::rep(|e| e.extend_right(1))),
                (Key::alt('h'), Mapping::rep(|e| e.extend_left(1))),
                (Key::alt('j'), Mapping::rep(|e| e.retract_down(1))),
                (Key::alt('k'), Mapping::rep(|e| e.text_extend_up(1))),
                (Key::alt('l'), Mapping::rep(|e| e.retract_right(1))),
                //
                (Key::char(' '), Mapping::once(Editor::jump)),
                (Key::char('f'), Mapping::once(Editor::find)),
                (Key::char('F'), Mapping::once(Editor::pick_file)),
                (Key::char('('), Mapping::once(Editor::previous_file)),
                (Key::char(')'), Mapping::once(Editor::next_file)),
                //
                (Key::char('z'), Mapping::once(Editor::undo)),
                (Key::char('Z'), Mapping::once(Editor::redo)),
                (Key::char('X'), Mapping::once(Editor::delete)),
                (Key::char('x'), Mapping::once(Editor::cut)),
                (Key::char('c'), Mapping::once(Editor::copy)),
                (Key::char('v'), Mapping::once(Editor::paste)),
                (Key::ctrl('s'), Mapping::once(Editor::save_file)),
                //
                (Key::char('\''), Mapping::once(Editor::hover)),
                (Key::char('2'), Mapping::once(Editor::code_actions)),
                (Key::char('*'), Mapping::once(|e| e.goto(Definition))),
                (Key::alt('8'), Mapping::once(|e| e.goto(Declaration))),
                (Key::alt('*'), Mapping::once(|e| e.goto(Implementation))),
                (Key::char('&'), Mapping::once(|e| e.goto(References))),
                (Key::char('Y'), Mapping::once(|e| e.goto(TypeDefinition))),
                //
                (Key::code(F(6)), Mapping::once(Editor::inspect)),
                (
                    Key::code(F(5)),
                    Mapping::once(Editor::refresh_semantic_tokens),
                ),
            ])),
            line_select: Keymap::from_iter(universal().chain([
                (
                    Key::char('i'),
                    Mapping::once(Editor::insert_on_newline_before),
                ),
                (
                    Key::char('a'),
                    Mapping::once(Editor::insert_on_newline_after),
                ),
                (Key::char('I'), Mapping::once(Editor::insert_before)),
                (Key::char('A'), Mapping::once(Editor::insert_after)),
                //
                (Key::code(Tab), Mapping::rep(Editor::tab_lines_in)),
                (Key::code(BackTab), Mapping::rep(Editor::tab_lines_out)),
                //
                (Key::char(';'), Mapping::once(Editor::select)),
                (Key::char(':'), Mapping::once(Editor::cursor_line_split)),
                (Key::code(Esc), Mapping::once(Editor::drop_other_selections)),
                (Key::char('9'), Mapping::rep(Editor::cycle_cursors_backward)),
                (Key::char('0'), Mapping::rep(Editor::cycle_cursors_forward)),
                //
                (Key::char('j'), Mapping::rep(|e| e.move_y(1))),
                (Key::char('k'), Mapping::rep(|e| e.move_y(-1))),
                (Key::char('J'), Mapping::rep(|e| e.text_extend_down(1))),
                (Key::char('K'), Mapping::rep(|e| e.retract_up(1))),
                (Key::alt('j'), Mapping::rep(|e| e.retract_down(1))),
                (Key::alt('k'), Mapping::rep(|e| e.text_extend_up(1))),
                //
                (Key::char('f'), Mapping::once(Editor::find)),
                //
                (Key::char('z'), Mapping::once(Editor::undo)),
                (Key::char('Z'), Mapping::once(Editor::redo)),
                (Key::char('X'), Mapping::once(Editor::delete)),
                (Key::char('x'), Mapping::once(Editor::cut)),
                (Key::char('c'), Mapping::once(Editor::copy)),
                (Key::char('v'), Mapping::once(Editor::paste)),
                (Key::ctrl('s'), Mapping::once(Editor::save_file)),
                //
                (Key::code(F(6)), Mapping::once(Editor::inspect)),
            ])),
        }
    }
}
