use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashMap;

mod default;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Key {
    code: KeyCode,
    ctrl: bool,
    alt: bool,
}

trait ToKey {
    fn to_key_code(self) -> KeyCode;
}

impl ToKey for KeyCode {
    fn to_key_code(self) -> KeyCode {
        self
    }
}

impl ToKey for char {
    fn to_key_code(self) -> KeyCode {
        KeyCode::Char(self)
    }
}

impl Key {
    pub fn code(code: KeyCode) -> Self {
        Self {
            code,
            ctrl: false,
            alt: false,
        }
    }

    pub fn char(char: char) -> Self {
        Self::code(KeyCode::Char(char))
    }

    pub fn alt(key: impl ToKey) -> Self {
        Self {
            code: key.to_key_code(),
            ctrl: false,
            alt: true,
        }
    }

    pub fn ctrl(key: impl ToKey) -> Self {
        Self {
            code: key.to_key_code(),
            ctrl: true,
            alt: false,
        }
    }

    pub fn from_event(
        KeyEvent {
            code, modifiers, ..
        }: KeyEvent,
    ) -> Self {
        let ctrl = !modifiers.intersection(KeyModifiers::CONTROL).is_empty();
        let alt = !modifiers.intersection(KeyModifiers::ALT).is_empty();
        Self { code, ctrl, alt }
    }
}

use crate::editor::Editor;

pub struct Keymaps {
    pub insert: Keymap,
    pub select: Keymap,
    pub line_select: Keymap,
}

pub struct Keymap(HashMap<Key, Mapping>);

impl FromIterator<(Key, Mapping)> for Keymap {
    fn from_iter<T: IntoIterator<Item = (Key, Mapping)>>(iter: T) -> Self {
        Self(HashMap::from_iter(iter))
    }
}

#[derive(Copy, Clone)]
struct Mapping {
    repeatable: bool,
    effect: fn(&mut Editor),
}

impl Mapping {
    fn rep(effect: fn(&mut Editor)) -> Self {
        Self {
            repeatable: true,
            effect,
        }
    }
    fn once(effect: fn(&mut Editor)) -> Self {
        Self {
            repeatable: false,
            effect,
        }
    }
}

impl Keymap {
    pub fn map_event(&self, event: KeyEvent) -> Option<impl Fn(&mut Editor) + use<>> {
        let key = Key::from_event(event);
        let mapping = self.0.get(&key)?;
        (event.is_press() || event.is_repeat() && mapping.repeatable).then_some(mapping.effect)
    }
}
