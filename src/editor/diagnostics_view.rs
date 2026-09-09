use crate::{document::diagnostics::Diagnostic, draw::screen::Canvas, editor::gadget::Gadget, grapheme::Grapheme, key::{KeyOrChar, key}, lang::{DiagnosticDisplayStyle, Language}, log::{DisplayLog, log}, style::Style};


pub struct DiagnosticsView {
    lang: Option<Language>,
    diagnostics: Vec<Diagnostic>,
}

impl DiagnosticsView {
    pub fn new(lang: Option<Language>, diagnostics: Vec<Diagnostic>) -> Self {
        Self {
            lang,
            diagnostics,
        }
    }
}

impl Gadget for DiagnosticsView {
    fn on_key(&mut self, event: KeyOrChar) -> Option<Box<dyn FnOnce(&mut super::Editor)>> {
        if event == KeyOrChar::Key(key![f6]) {
            log!(DisplayLog {
                category: crate::log::LogCategory::Debug,
                message: "ra diagnostic data",
                details: format!("{:#?}", self.diagnostics.iter().filter_map(|d| d.data.as_ref()).collect::<Vec<_>>()),
            })
        }

        None
    }

    fn draw(&self, mut canvas: Canvas<'_>) {
        let mut cursor = canvas.at((0, 0));
        try {
            match try { self.lang?.lsp_info()?.diagnostic_display_style } {
                Some(DiagnosticDisplayStyle::RustRendered) => for diagnostic in &self.diagnostics {
                    let Some(msg) = (try { diagnostic.data.as_ref()?.as_object()?.get("rendered")?.as_str()? }) else {continue};
                    cursor.write_lines(msg, diagnostic.severity.style()).ok()?;
                    cursor.next_line().ok()?;
                    cursor.next_line().ok()?;
                },
                _ => for diagnostic in &self.diagnostics {
                    cursor.write_boxed_lines_wrapping(&diagnostic.message, diagnostic.severity.style() + Style::italic(), (Grapheme::LEFT_TRIANGLE, Grapheme::RIGHT_TRIANGLE)).ok()?;
                    cursor.next_line().ok()?;
                    cursor.next_line().ok()?;
                }
            }
        };
    }
}
