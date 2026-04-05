use std::{convert::identity, sync::Arc};

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use lsp_types::{CompletionItem, CompletionItemKind};

use crate::{
    color,
    draw::screen::Canvas,
    editor::{Editor, gadget::Gadget},
    grapheme::GraphemeExt,
    style::Style,
};

pub enum CompletionKind {
    Other,
    Method,
    Function,
    Field,
    Variable,
    Class,
    Interface,
    Module,
    Enum,
    Keyword,
    Snippet,
    File,
    Folder,
    Variant,
    Constant,
    Struct,
    Operator,
    TypeParameter,
}

impl From<CompletionItemKind> for CompletionKind {
    fn from(value: CompletionItemKind) -> Self {
        match value {
            CompletionItemKind::METHOD => Self::Method,
            CompletionItemKind::FUNCTION => Self::Function,
            CompletionItemKind::CONSTRUCTOR => Self::Function,
            CompletionItemKind::FIELD => Self::Field,
            CompletionItemKind::VARIABLE => Self::Variable,
            CompletionItemKind::CLASS => Self::Class,
            CompletionItemKind::INTERFACE => Self::Interface,
            CompletionItemKind::MODULE => Self::Module,
            CompletionItemKind::PROPERTY => Self::Field,
            CompletionItemKind::ENUM => Self::Enum,
            CompletionItemKind::KEYWORD => Self::Keyword,
            CompletionItemKind::SNIPPET => Self::Snippet,
            CompletionItemKind::FILE => Self::File,
            CompletionItemKind::FOLDER => Self::Folder,
            CompletionItemKind::ENUM_MEMBER => Self::Variant,
            CompletionItemKind::CONSTANT => Self::Constant,
            CompletionItemKind::STRUCT => Self::Struct,
            CompletionItemKind::OPERATOR => Self::Operator,
            CompletionItemKind::TYPE_PARAMETER => Self::TypeParameter,
            _ => Self::Other,
        }
    }
}

pub struct CompletionOption {
    label: Arc<str>,
    #[allow(unused)]
    kind: CompletionKind,
    text: Arc<str>,
}

impl CompletionOption {
    fn from_lsp(item: CompletionItem) -> Self {
        let CompletionItem {
            label,
            kind,
            insert_text,
            ..
        } = item;

        let label: Arc<str> = label.into();

        let or_label = |input: Option<String>| -> Arc<str> {
            input.map(Into::into).unwrap_or_else(|| label.clone())
        };

        Self {
            text: or_label(insert_text),
            label,
            kind: kind.map(|k| k.into()).unwrap_or(CompletionKind::Other),
        }
    }
}

pub struct Completer {
    items: Vec<CompletionOption>,
    selected: usize,
}

impl Completer {
    pub fn new(mut items: Vec<CompletionItem>) -> Self {
        items.sort_unstable_by(|a, b| a.sort_text.cmp(&b.sort_text));
        let selected = items
            .iter()
            .enumerate()
            .find(|(_, i)| i.preselect.is_some_and(identity))
            .map(|(i, _)| i)
            .unwrap_or(0);
        Completer {
            selected,
            items: items.into_iter().map(CompletionOption::from_lsp).collect(),
        }
    }
}

impl Gadget for Completer {
    fn on_key(&mut self, event: KeyEvent) -> Option<Box<dyn FnOnce(&mut super::Editor)>> {
        macro_rules! xx {
            ($($tokens: tt)*) => {
                Some(Box::new($($tokens)*))
            };
        }
        match event {
            KeyEvent {
                code: KeyCode::Char(_),
                modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                kind: KeyEventKind::Press | KeyEventKind::Repeat,
                ..
            } => None,

            KeyEvent {
                code: KeyCode::Backspace,
                kind: KeyEventKind::Press | KeyEventKind::Repeat,
                ..
            } => None,

            KeyEvent {
                code: KeyCode::Tab,
                modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                kind: KeyEventKind::Press | KeyEventKind::Repeat,
                ..
            } => {
                if self.items.is_empty() {
                    return None;
                }
                self.selected = (self.selected + 1) % self.items.len();
                xx!(Editor::noop)
            }
            KeyEvent {
                code: KeyCode::BackTab,
                modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                kind: KeyEventKind::Press | KeyEventKind::Repeat,
                ..
            } => {
                if self.items.is_empty() {
                    return None;
                }
                self.selected = self.selected.wrapping_sub(1) % self.items.len();
                xx!(Editor::noop)
            }

            KeyEvent {
                code: KeyCode::Enter,
                kind: KeyEventKind::Press,
                ..
            } => {
                let text = self.items[self.selected].text.clone();
                xx! {
                    move |e| {
                        if let Some(pos) = e.doc.main_cursor_pos() {
                            e.doc.insert_completion(pos, &text);
                        }
                        e.close_gadget();
                    }
                }
            }

            _ => None,
        }
    }

    fn draw(&self, mut canvas: Canvas<'_>) {
        let style = (Style::fg(color::FG) + Style::bg(color::BG)).into();

        for (i, item) in (0..canvas.height()).zip(&self.items) {
            let style = if i == self.selected as u16 {
                (Style::fg(color::FG) + Style::bg(color::LIT_BG)).into()
            } else {
                style
            };
            for (j, g) in (0..canvas.width()).zip(item.label.graphemes()) {
                let cell = &mut canvas[(i, j)];
                cell.grapheme = g;
                cell.style = style;
            }
        }
    }
}
