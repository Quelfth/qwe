use std::sync::LazyLock;

use crossterm::event::KeyCode::*;

use crate::editor::{
    Editor,
    keymap::{Key, Keymap, Keymaps, Mapping},
};

static UNIVERSAL: LazyLock<&'static [(Key, Mapping)]> = LazyLock::new(|| {
    Box::leak(Box::new({
        [
            (Key::ctrl('d'), Mapping::rep(|e| e.scroll_down(4))),
            (Key::ctrl('u'), Mapping::rep(|e| e.scroll_up(4))),
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
                (Key::code(Enter), Mapping::rep(Editor::r#return)),
                (Key::code(Tab), Mapping::rep(Editor::insert_tab)),
            ])),
            select: Keymap::from_iter(universal().chain([
                (Key::char('i'), Mapping::once(Editor::insert_before)),
                (Key::char('a'), Mapping::once(Editor::insert_after)),
                (Key::char('I'), Mapping::once(Editor::insert_before_line)),
                (Key::char('A'), Mapping::once(Editor::insert_after_line)),
                //
                (Key::char(';'), Mapping::once(Editor::line_select)),
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
                (Key::ctrl('s'), Mapping::once(Editor::save_file)),
                //
                (Key::alt('^'), Mapping::once(Editor::inspect)),
            ])),
            line_select: Keymap::from_iter(universal().chain([
                (Key::char('i'), Mapping::once(Editor::insert_before)),
                (Key::char('a'), Mapping::once(Editor::insert_after)),
                (Key::char('I'), Mapping::once(Editor::insert_before_line)),
                (Key::char('A'), Mapping::once(Editor::insert_after_line)),
                //
                (Key::char(','), Mapping::once(Editor::select)),
                //
                (Key::char('j'), Mapping::rep(|e| e.move_y(1))),
                (Key::char('k'), Mapping::rep(|e| e.move_y(-1))),
                (Key::char('J'), Mapping::rep(|e| e.text_extend_down(1))),
                (Key::char('K'), Mapping::rep(|e| e.retract_up(1))),
                (Key::alt('j'), Mapping::rep(|e| e.retract_down(1))),
                (Key::alt('k'), Mapping::rep(|e| e.text_extend_up(1))),
                //
                (Key::ctrl('s'), Mapping::once(Editor::save_file)),
                //
                (Key::alt('^'), Mapping::once(Editor::inspect)),
            ])),
        }
    }
}
