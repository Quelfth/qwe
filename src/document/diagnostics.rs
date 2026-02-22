use crossterm::style::Color;
use culit::culit;
use lsp_types::DiagnosticSeverity;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Default)]
pub enum Severity {
    Err,
    #[default]
    Warn,
    Info,
    Hint,
}

impl Severity {
    pub fn from_lsp(severity: Option<DiagnosticSeverity>) -> Self {
        let Some(severity) = severity else {
            return Self::default();
        };
        match severity {
            DiagnosticSeverity::ERROR => Self::Err,
            DiagnosticSeverity::WARNING => Self::Warn,
            DiagnosticSeverity::INFORMATION => Self::Info,
            DiagnosticSeverity::HINT => Self::Hint,
            _ => Self::default(),
        }
    }

    #[culit]
    pub fn fg(self) -> Color {
        match self {
            Severity::Err => 0xff007frgb,
            Severity::Warn => 0xbfff01rgb,
            Severity::Info => 0x00ff7frgb,
            Severity::Hint => 0x906060rgb,
        }
    }

    #[culit]
    pub fn bg(self) -> Color {
        match self {
            Severity::Err => 0x300015rgb,
            Severity::Warn => 0x203000rgb,
            Severity::Info => 0x302020rgb,
            Severity::Hint => 0x302020rgb,
        }
    }
}

pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
}
