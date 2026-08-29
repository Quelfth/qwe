use std::range::Range;

use crate::{
    document::{Document, diagnostics::Severity},
    draw::screen::Canvas,
    grapheme::Grapheme,
    ix::{Column, Ix, Line, ix},
    style::Style,
    util::RangeLen as _,
};

struct Annotation {
    lines: Range<Ix<Line>>,
    column: Ix<Column>,
    severity: Severity,
}

fn annotations(doc: &Document) -> impl Iterator<Item = Annotation> {
    doc.diagnostics.ranges().filter_map(|(r, d)| {
        let range = doc.text().pos_of_byte_pos(r.start)?..doc.text().pos_of_byte_pos(r.end)?;
        let lines = range.start.line..range.end.line + if range.end.column != ix(0) { ix(1) } else { ix(0) };
        if lines.len() <= ix(1) { return None }
        let column = lines.into_iter().map(|l| doc.text().context_indent_inc(l)).min().unwrap_or_default();

        Some(Annotation {
            lines,
            column,
            severity: d.severity,
        })
    })
}

impl Document {
    pub fn draw_annotations(&self, mut canvas: Canvas<'_>) {
        let g = self.gutter_width();
        'outer:
        for Annotation { lines, column, severity } in annotations(self) {
            let Some(c) = (column.inner() as u16 + g).checked_sub(1) else {continue};
            for (i, line) in lines.into_iter().enumerate() {
                let Some(r) = line.checked_sub(self.scroll) else {continue};
                let r = r.inner() as u16;
                if r >= canvas.height() { continue 'outer }
                let cell = &mut canvas[(r, c)];
                let n = lines.len().inner();
                cell.grapheme = if severity.is_bad() {
                    Grapheme::VERTICAL_SQUIGGLE
                } else {
                    brace_part(i, n)
                };
                cell.style = cell.style + Style::fg(severity.fg());
            }
        }
    }
}

fn brace_part(i: usize, n: usize) -> Grapheme {
    if n == 2 {
        match i {
            0 => Grapheme::BRACE_2_TOP,
            1 => Grapheme::BRACE_2_BOTTOM,
            _ => unreachable!(),
        }
    } else {
        if i == 0 {
            Grapheme::BRACE_TOP
        } else if i == n - 1 {
            Grapheme::BRACE_BOTTOM
        } else if i == n / 2 {
            if n.is_multiple_of(2) {
                Grapheme::BRACE_CUSP_BOTTOM
            } else {
                Grapheme::BRACE_CUSP
            }
        } else if i == n / 2 - 1 && n.is_multiple_of(2) {
            Grapheme::BRACE_CUSP_TOP
        } else {
            Grapheme::BRACE_BAR
        }
    }
}
