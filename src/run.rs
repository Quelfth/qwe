use std::{io, time::Duration};

use crossterm::{
    event::{self, Event, poll},
    style::Color,
    terminal,
};
use dispa::dispatch;
use tokio::sync::mpsc;

use crate::{
    app::{
        AppSignal,
        AppState,
    },
    action::Action as _,
    draw::screen::Canvas,
    editor::Editor,
    global_config::GLOBAL_CONFIG,
    init::InitState,
    ix::Ix,
    key::{Key, KeyOrChar},
    lsp::{self, channel::EditorToLspMessage, run_lsp_thread},
    navigator::Navigator,
    presenter::{
        Present,
        Presenter,
    },
    terminal_size::set_terminal_size,
};

pub fn run(InitState{ doc, pos, autosave }: InitState) -> io::Result<()> {
    let (width, height) = terminal::size()?;
    set_terminal_size(width, height);

    let mut editor = Editor::new();
    let (send_lsp_to_editor, recv_lsp_to_editor) = std::sync::mpsc::channel();
    let (send_editor_to_lsp, recv_editor_to_lsp) = mpsc::unbounded_channel();
    editor.set_lsp_channels(send_editor_to_lsp, recv_lsp_to_editor);
    if let Some(file) = doc {
        _= editor.open_file_doc(file.path);
    } else {
        editor.open_scratch_doc();
    }

    let _lsp_thread_handle = run_lsp_thread(lsp::channel::LspChannels {
        outgoing: send_lsp_to_editor,
        incoming: recv_editor_to_lsp,
    })?;

    if autosave && let Some(lang) = editor.doc().language() {
        GLOBAL_CONFIG.autosave_langs.lock().insert(lang);
    }

    if let Some(pos) = pos {
        editor.jump_to(pos);
        *editor.doc().view_height.lock() = Ix::new(height as _);
        editor.scroll_main_cursor_on_screen();
    }
    editor.draw()?;

    #[dispatch(AppState)]
    #[dispatch(Present)]
    enum State {
        Editor(Editor),
        Navigator(Navigator),
    }

    let mut state = State::Editor(editor);

    macro handle_signal($signal: expr) {
        use AppSignal::*;
        if let Some(signal) = $signal {
            match signal {
                Quit => break,
                Editor => state = state_into_editor(state)?,
                Navigator => state = state_into_navigator(state)?,
            }
        }
    }

    fn state_into_editor(mut state: State) -> io::Result<State> {
        if let State::Navigator(navg) = state {
            state = State::Editor(navg.into_editor());
            state.draw()?;
        }
        Ok(state)
    }
    fn state_into_navigator(mut state: State) -> io::Result<State> {
        if let State::Editor(editor) = state {
            state = State::Navigator(editor.into_navigator());
            state.draw()?;
        }
        Ok(state)
    }

    loop {
        if poll(Duration::from_millis(2))? {
            match event::read()? {
                Event::FocusGained => (),
                Event::FocusLost => (),
                Event::Key(event) =>
                    if let Some(key) = KeyOrChar::from_key_event(event) {
                        if let KeyOrChar::Key(key) = key
                            && let Some(action) = GLOBAL_CONFIG.keymaps.app.load()[key] {
                                handle_signal!(action.act(()));
                            }
                        handle_signal!(state.on_key_or_char(key)?);
                    },
                Event::Mouse(event) => {
                    if let Some(key) = Key::from_mouse_event(event) {
                        handle_signal!(state.on_key_or_char(KeyOrChar::Key(key))?);
                    }
                },
                Event::Paste(string) => state.on_paste(string)?,
                Event::Resize(width, height) => {
                    if set_terminal_size(width, height) {
                        state.draw()?
                    }
                }
            }
        }
        state.poll()?;
    }
    if let State::Editor(editor) = state
        && let Some(cx) = editor.lsp {
            cx.tx.send(EditorToLspMessage::Exit).unwrap();
        }

    Ok(())
}
