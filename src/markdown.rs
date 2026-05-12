use std::{collections::HashMap, iter, sync::{Arc}};

use crate::{color, document::Document, draw::screen::{CanvasCursor, EndOfCanvas}, grapheme::Grapheme, lang::Language, style::Style};

use markdown::mdast::*;
use mutx::Mutex;

pub type MdCxCache = Arc<Mutex<HashMap<(Option<String>, String), Document>>>;

#[derive(Clone)]
pub struct MdContext {
    italic: bool,
    bold: bool,
    strikethrough: bool,
    cache: MdCxCache,
}

impl MdContext {
    pub fn new(cache: MdCxCache) -> Self {
        Self {
            italic: false,
            bold: false,
            strikethrough: false,
            cache,
        }
    }
}

impl MdContext {
    fn style(self) -> Style {
        let Self { italic, bold, strikethrough, .. } = self;
        let mut style = Style::default();
        if italic { style = style + Style::italic() }
        if bold { style = style + Style::bold() }
        style
    }
}

pub trait MdDraw {
    fn draw(&self, cx: MdContext, cursor: &mut CanvasCursor<'_, '_>) -> Result<(), EndOfCanvas>;
}

impl MdDraw for Node {
    fn draw(&self, cx: MdContext, cursor: &mut CanvasCursor<'_, '_>) -> Result<(), EndOfCanvas> {
        match self {
            Node::Root(root) => root.draw(cx, cursor)?,
            Node::Paragraph(paragraph) => paragraph.draw(cx, cursor)?,
            Node::Text(text) => text.draw(cx, cursor)?,
            Node::Emphasis(emphasis) => emphasis.draw(cx, cursor)?,
            Node::Strong(strong) => strong.draw(cx, cursor)?,
            Node::Delete(delete) => delete.draw(cx, cursor)?,
            Node::InlineCode(inline_code) => inline_code.draw(cx, cursor)?,
            Node::Code(code) => code.draw(cx, cursor)?,
            Node::Heading(heading) => heading.draw(cx, cursor)?,
            Node::ThematicBreak(thematic_break) => thematic_break.draw(cx, cursor)?,
            Node::Blockquote(blockquote) => blockquote.draw(cx, cursor)?,
            Node::List(list) => list.draw(cx, cursor)?,
            Node::ListItem(list_item) => list_item.draw(cx, cursor)?,
            Node::Break(r#break) => (),
            Node::Link(link) => link.draw(cx, cursor)?,
            Node::LinkReference(link_reference) => link_reference.draw(cx, cursor)?,
            Node::Definition(definition) => (),
            Node::FootnoteDefinition(footnote_definition) => (),
            Node::FootnoteReference(footnote_reference) => (),
            Node::Table(table) => (),
            Node::TableRow(table_row) => (),
            Node::TableCell(table_cell) => (),
            _ => (),
        }
        Ok(())
    }
}

impl MdDraw for Root {
    fn draw(&self, cx: MdContext, cursor: &mut CanvasCursor<'_, '_>) -> Result<(), EndOfCanvas> {
        for child in &self.children {
            child.draw(cx.clone(), cursor)?
        }
        Ok(())
    }
}

impl MdDraw for Paragraph {
    fn draw(&self, cx: MdContext, cursor: &mut CanvasCursor<'_, '_>) -> Result<(), EndOfCanvas> {
        for child in &self.children {
            child.draw(cx.clone(), cursor)?
        }
        cursor.break_line()?;
        Ok(())
    }
}

impl MdDraw for Text {
    fn draw(&self, cx: MdContext, cursor: &mut CanvasCursor<'_, '_>) -> Result<(), EndOfCanvas> {
        cursor.write_wrapping(&self.value, Style::fg(color::MD_FG) + Style::bg(color::MD_BG) + cx.style())?;
        Ok(())
    }
}

impl MdDraw for Emphasis {
    fn draw(&self, cx: MdContext, cursor: &mut CanvasCursor<'_, '_>) -> Result<(), EndOfCanvas> {
        for child in &self.children {
            child.draw(MdContext { italic: true, ..cx.clone() }, cursor)?;
        }
        Ok(())
    }
}

impl MdDraw for Strong {
    fn draw(&self, cx: MdContext, cursor: &mut CanvasCursor<'_, '_>) -> Result<(), EndOfCanvas> {
        for child in &self.children {
            child.draw(MdContext { bold: true, ..cx.clone() }, cursor)?;
        }
        Ok(())
    }
}

impl MdDraw for Delete {
    fn draw(&self, cx: MdContext, cursor: &mut CanvasCursor<'_, '_>) -> Result<(), EndOfCanvas> {
        for child in &self.children {
            child.draw(MdContext { strikethrough: true, ..cx.clone() }, cursor)?;
        }
        Ok(())
    }
}

impl MdDraw for InlineCode {
    fn draw(&self, _: MdContext, cursor: &mut CanvasCursor<'_, '_>) -> Result<(), EndOfCanvas> {
        cursor.write_box_wrapping(&self.value, Style::fg(color::FG) + Style::bg(color::BG), (Grapheme::LEFT_SEMICIRCLE, Grapheme::RIGHT_SEMICIRCLE))?;
        Ok(())
    }
}

impl MdDraw for Code {
    fn draw(&self, cx: MdContext, cursor: &mut CanvasCursor<'_, '_>) -> Result<(), EndOfCanvas> {
        let cache = &mut *cx.cache.lock();
        let doc = cache.entry((self.lang.clone(), self.value.clone())).or_insert_with(|| Document::new(self.lang.as_deref().and_then(Language::from_file_ext), &self.value, None));
        cursor.draw_document(doc)?;
        Ok(())
    }
}

impl MdDraw for Heading {
    fn draw(&self, cx: MdContext, cursor: &mut CanvasCursor<'_, '_>) -> Result<(), EndOfCanvas> {
        for child in &self.children {
            child.draw(MdContext { bold: true, ..cx.clone() }, cursor)?;
        }
        cursor.break_line()?;
        Ok(())
    }
}

impl MdDraw for ThematicBreak {
    fn draw(&self, _: MdContext, cursor: &mut CanvasCursor<'_, '_>) -> Result<(), EndOfCanvas> {
       cursor.break_line()?;
       _=cursor.write(iter::repeat_n("-", cursor.canvas_width() as _).collect::<String>(), Style::fg(color::MD_FG) + Style::bg(color::MD_BG) + Style::bold());
       cursor.break_line()?;
       Ok(())
    }
}

impl MdDraw for Blockquote {
    fn draw(&self, cx: MdContext, cursor: &mut CanvasCursor<'_, '_>) -> Result<(), EndOfCanvas> {
        let style = Style::fg(color::MD_FG) + Style::bg(color::MD_BG);
        cursor.break_line()?;
        _= cursor.write("\"", style + Style::bold());
        cursor.next_line()?;
        _= cursor.write("    ", style);
        for child in &self.children {
            child.draw(cx.clone(), cursor)?;
        }
        Ok(())
    }
}

impl MdDraw for List {
    fn draw(&self, cx: MdContext, cursor: &mut CanvasCursor<'_, '_>) -> Result<(), EndOfCanvas> {
        //cursor.break_line()?;
        for child in &self.children {
            child.draw(cx.clone(), cursor)?;
        }
        Ok(())
    }
}

impl MdDraw for ListItem {
    fn draw(&self, cx: MdContext, cursor: &mut CanvasCursor<'_, '_>) -> Result<(), EndOfCanvas> {
        cursor.break_line()?;
        _= cursor.write("  \u{2022} ", Style::fg(color::MD_FG) + Style::bg(color::MD_BG));
        for child in &self.children {
            child.draw(cx.clone(), cursor)?;
        }
        Ok(())
    }
}

impl MdDraw for Link {
    fn draw(&self, cx: MdContext, cursor: &mut CanvasCursor<'_, '_>) -> Result<(), EndOfCanvas> {
        for child in &self.children {
            child.draw(cx.clone(), cursor)?;
        }
        Ok(())
    }
}

impl MdDraw for LinkReference {
    fn draw(&self, cx: MdContext, cursor: &mut CanvasCursor<'_, '_>) -> Result<(), EndOfCanvas> {
        for child in &self.children {
            child.draw(cx.clone(), cursor)?;
        }
        Ok(())
    }
}
