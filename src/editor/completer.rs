use crossterm::event::KeyEvent;
use lsp_types::{CompletionItem, CompletionItemKind};

use crate::{
    color, draw::screen::Canvas, editor::gadget::Gadget, grapheme::GraphemeExt, style::Style,
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
    label: String,
    kind: CompletionKind,
    text: String,
}

impl CompletionOption {
    fn from_lsp(item: CompletionItem) -> Self {
        let CompletionItem {
            label,
            kind,
            insert_text,
            ..
        }: CompletionItem = item;

        Self {
            text: insert_text.unwrap_or_else(|| label.clone()),
            label,
            kind: kind.map(|k| k.into()).unwrap_or(CompletionKind::Other),
        }
    }
}

pub struct Completer {
    items: Vec<CompletionOption>,
}

impl Completer {
    pub fn new(items: impl IntoIterator<Item = CompletionItem>) -> Self {
        Completer {
            items: items.into_iter().map(CompletionOption::from_lsp).collect(),
        }
    }
}

impl Gadget for Completer {
    fn on_key(&mut self, _: KeyEvent) -> Option<Box<dyn FnOnce(&mut super::Editor)>> {
        None
    }

    fn draw(&self, mut canvas: Canvas<'_>) {
        let style = (Style::fg(color::FG) + Style::bg(color::BG)).into();

        for (i, item) in (0..canvas.height()).zip(&self.items) {
            for (j, g) in (0..canvas.width()).zip(item.label.graphemes()) {
                let cell = &mut canvas[(i, j)];
                cell.grapheme = g;
                cell.style = style;
            }
        }
    }
}
