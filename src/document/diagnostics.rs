use crossterm::style::Color;
use culit::culit;
use lsp_types::DiagnosticSeverity;

use crate::{lang::Language, style::Style};

#[derive(Copy, Clone, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub enum Severity {
    Context,
    Hint,
    Info,
    #[default]
    Warn,
    Err,
}

impl Severity {
    pub fn from_lsp(lang: Option<Language>, severity: Option<DiagnosticSeverity>) -> Self {
        try { (lang?.lsp_info()?.severity_map)(severity?) }.unwrap_or_default()
    }

    #[culit]
    pub fn fg(self) -> Color {
        match self {
            Severity::Err => 0xff007frgb,
            Severity::Warn => 0xbfff01rgb,
            Severity::Info => 0x00ff7frgb,
            Severity::Hint => 0x00b5ffrgb,
            Severity::Context => 0x906060rgb,
        }
    }

    #[culit]
    pub fn bg(self) -> Color {
        match self {
            Severity::Err => 0x300015rgb,
            Severity::Warn => 0x203000rgb,
            Severity::Info => 0x005042rgb,
            Severity::Hint => 0x003a52rgb,
            Severity::Context => 0x302020rgb,
        }
    }

    pub fn style(self) -> Style {
        Style::fg(self.fg()) + Style::bg(self.bg())
    }

    pub fn is_bad(self) -> bool {
        use Severity::*;
        matches!(self, Err | Warn)
    }
}

#[derive(Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
}
