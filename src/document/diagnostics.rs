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
}

pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
}
