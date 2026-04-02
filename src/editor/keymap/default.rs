use std::sync::LazyLock;

use crossterm::event::KeyCode::*;

use crate::{
    editor::{
        Editor,
        keymap::{Key, Keymap, Keymaps, Mapping},
    },
    lsp::channel::GotoKind::*,
};

use super::ScrollDir;

static UNIVERSAL: LazyLock<&'static [(Key, Mapping)]> = LazyLock::new(|| {
    Box::leak(Box::new({
        [
            (Key::ctrl('d'), Mapping::rep(|e| e.scroll_down(4))),
            (Key::ctrl('u'), Mapping::rep(|e| e.scroll_up(4))),
            (Key::ctrl('r'), Mapping::rep(|e| e.scroll_right(4))),
            (Key::ctrl('y'), Mapping::rep(|e| e.scroll_left(4))),
            (Key::base(ScrollDir::Down), Mapping::rep(|e| e.scroll_down(4))),
            (Key::base(ScrollDir::Up), Mapping::rep(|e| e.scroll_up(4))),
            (Key::base(ScrollDir::Left), Mapping::rep(|e| e.scroll_left(4))),
            (Key::base(ScrollDir::Right), Mapping::rep(|e| e.scroll_right(4))),
            //(Key::ctrl('y'), Mapping::once(Editor::debug_undo)),
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
                (Key::base(Esc), Mapping::once(Editor::select)),
                (Key::base(Backspace), Mapping::rep(Editor::backspace)),
                (Key::base(Enter), Mapping::rep(Editor::insert_return)),
                (
                    Key::base(Tab),
                    Mapping::rep(Editor::insert_tab_else_complete),
                ),
                (Key::base(BackTab), Mapping::rep(Editor::tab_out)),
                (Key::ctrl('z'), Mapping::once(Editor::undo)),
                (Key::ctrl('v'), Mapping::once(Editor::paste)),
            ])),
            select: Keymap::from_iter(universal().chain([
                (Key::base('i'), Mapping::once(Editor::insert_before)),
                (Key::base('a'), Mapping::once(Editor::insert_after)),
                (Key::base('I'), Mapping::once(Editor::insert_before_line)),
                (Key::base('A'), Mapping::once(Editor::insert_after_line)),
                (Key::base('['), Mapping::once(Editor::mirror_insert_in)),
                (Key::base(']'), Mapping::once(Editor::mirror_insert_out)),
                //
                (Key::base(Tab), Mapping::rep(Editor::tab_lines_in)),
                (Key::base(BackTab), Mapping::rep(Editor::tab_lines_out)),
                //
                (Key::base('o'), Mapping::once(Editor::syntax_extend)),
                (Key::base('w'), Mapping::once(Editor::incremental_select)),
                (Key::base(';'), Mapping::once(Editor::line_select)),
                (Key::base(':'), Mapping::once(Editor::cursor_line_split)),
                (Key::base(Esc), Mapping::once(Editor::drop_other_selections)),
                (Key::base('u'), Mapping::once(Editor::collapse_cursors_to_start)),
                (Key::base('q'), Mapping::once(Editor::collapse_cursors_to_end)),
                (Key::base('9'), Mapping::rep(Editor::cycle_cursors_backward)),
                (Key::base('0'), Mapping::rep(Editor::cycle_cursors_forward)),
                (Key::base('8'), Mapping::once(Editor::scroll_to_main_cursor)),
                //
                (Key::base('h'), Mapping::rep(|e| e.move_x(-1))),
                (Key::base('j'), Mapping::rep(|e| e.move_y(1))),
                (Key::base('k'), Mapping::rep(|e| e.move_y(-1))),
                (Key::base('l'), Mapping::rep(|e| e.move_x(1))),
                (Key::base('H'), Mapping::rep(|e| e.retract_left(1))),
                (Key::base('J'), Mapping::rep(|e| e.text_extend_down(1))),
                (Key::base('K'), Mapping::rep(|e| e.retract_up(1))),
                (Key::base('L'), Mapping::rep(|e| e.extend_right(1))),
                (Key::alt('h'), Mapping::rep(|e| e.extend_left(1))),
                (Key::alt('j'), Mapping::rep(|e| e.retract_down(1))),
                (Key::alt('k'), Mapping::rep(|e| e.text_extend_up(1))),
                (Key::alt('l'), Mapping::rep(|e| e.retract_right(1))),
                //
                (Key::base(' '), Mapping::once(Editor::jump)),
                (Key::base('f'), Mapping::once(Editor::find)),
                (Key::base('F'), Mapping::once(Editor::pick_file)),
                (Key::base('('), Mapping::once(Editor::previous_file)),
                (Key::base(')'), Mapping::once(Editor::next_file)),
                //
                (Key::base('z'), Mapping::once(Editor::undo)),
                (Key::base('Z'), Mapping::once(Editor::redo)),
                (Key::base('X'), Mapping::once(Editor::delete)),
                (Key::base('x'), Mapping::once(Editor::cut)),
                (Key::base('c'), Mapping::once(Editor::copy)),
                (Key::base('v'), Mapping::once(Editor::paste)),
                (Key::ctrl('s'), Mapping::once(Editor::save_file)),
                //
                (Key::base('\''), Mapping::once(Editor::hover)),
                (Key::base('2'), Mapping::once(Editor::code_actions)),
                (Key::base('*'), Mapping::once(|e| e.goto(Definition))),
                (Key::alt('8'), Mapping::once(|e| e.goto(Declaration))),
                (Key::alt('*'), Mapping::once(|e| e.goto(Implementation))),
                (Key::base('&'), Mapping::once(|e| e.goto(References))),
                (Key::base('Y'), Mapping::once(|e| e.goto(TypeDefinition))),
                //
                (Key::base(F(6)), Mapping::once(Editor::inspect)),
                (
                    Key::base(F(5)),
                    Mapping::once(Editor::refresh_semantic_tokens),
                ),
            ])),
            line_select: Keymap::from_iter(universal().chain([
                (
                    Key::base('i'),
                    Mapping::once(Editor::insert_on_newline_before),
                ),
                (
                    Key::base('a'),
                    Mapping::once(Editor::insert_on_newline_after),
                ),
                (Key::base('I'), Mapping::once(Editor::insert_before)),
                (Key::base('A'), Mapping::once(Editor::insert_after)),
                //
                (Key::base(Tab), Mapping::rep(Editor::tab_lines_in)),
                (Key::base(BackTab), Mapping::rep(Editor::tab_lines_out)),
                //
                (Key::base(';'), Mapping::once(Editor::select)),
                (Key::base(':'), Mapping::once(Editor::cursor_line_split)),
                (Key::base(Esc), Mapping::once(Editor::drop_other_selections)),
                (Key::base('u'), Mapping::once(Editor::collapse_cursors_to_start)),
                (Key::base('q'), Mapping::once(Editor::collapse_cursors_to_end)),
                (Key::base('9'), Mapping::rep(Editor::cycle_cursors_backward)),
                (Key::base('0'), Mapping::rep(Editor::cycle_cursors_forward)),
                //
                (Key::base('j'), Mapping::rep(|e| e.move_y(1))),
                (Key::base('k'), Mapping::rep(|e| e.move_y(-1))),
                (Key::base('J'), Mapping::rep(|e| e.text_extend_down(1))),
                (Key::base('K'), Mapping::rep(|e| e.retract_up(1))),
                (Key::alt('j'), Mapping::rep(|e| e.retract_down(1))),
                (Key::alt('k'), Mapping::rep(|e| e.text_extend_up(1))),
                //
                (Key::base('f'), Mapping::once(Editor::find)),
                //
                (Key::base('z'), Mapping::once(Editor::undo)),
                (Key::base('Z'), Mapping::once(Editor::redo)),
                (Key::base('X'), Mapping::once(Editor::delete)),
                (Key::base('x'), Mapping::once(Editor::cut)),
                (Key::base('c'), Mapping::once(Editor::copy)),
                (Key::base('v'), Mapping::once(Editor::paste)),
                (Key::ctrl('s'), Mapping::once(Editor::save_file)),
                //
                (Key::base(F(6)), Mapping::once(Editor::inspect)),
            ])),
        }
    }
}
