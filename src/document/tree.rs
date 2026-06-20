use std::{cmp, mem, range::Range};

use thiserror::Error;
use tree_sitter::{InputEdit, LanguageError, Parser, QueryCapture, QueryCursor, Tree};

use crate::{ix::{Byte, Ix}, lang::{Injections, Language}, rope::Rope, ts::{self, QueryCx}};


pub struct MetaTree {
    pub tree: Tree,
    pub injections: Vec<Injection>,
}

impl MetaTree {
    pub fn simple(tree: Tree) -> Self {
        Self {
            tree,
            injections: Vec::new(),
        }
    }

    pub fn parse(cx: Option<&QueryCx<'_>>, lang: Language, text: &Rope, range: Option<Range<Ix<Byte>>>) -> Result<Self, ParseTreeError> {
        parse_meta_tree(cx, lang, text, range, None)
    }

    pub fn reparse(&self, cx: Option<&QueryCx<'_>>, lang: Language, text: &Rope, range: Option<Range<Ix<Byte>>>) -> Result<Self, ParseTreeError> {
        parse_meta_tree(cx, lang, text, range, Some(self))
    }

    pub fn edit(&mut self, edit: &InputEdit) {
        self.tree.edit(edit);
        for inj in &mut self.injections {
            inj.tree.edit(edit);
        }
    }
}

#[derive(Debug, Error)]
pub enum ParseTreeError {
    #[error("{0}")]
    Language(#[from] LanguageError),
    #[error("no tree was parsed")]
    NoTree,
}

fn parse_tree(lang: Language, text: &Rope, range: Option<Range<Ix<Byte>>>, previous: Option<&Tree>) -> Result<Tree, ParseTreeError> {
    let mut parser = Parser::new();
    _=parser.set_language(&lang.ts_lang());

    if let Some(range) = range && let Some(range) = text.ts_range_of_byte_range(range) {
        _=parser.set_included_ranges(&[range]);
    }
    
    let tree = parser
        .parse_with_options(&mut text.ts_callback(), previous, None)
        .ok_or(ParseTreeError::NoTree)?;

    Ok(tree)
}

fn query_injections(cx: &QueryCx<'_>, lang: Language, tree: &Tree, text: &Rope, previous: Option<&MetaTree>) -> Result<Vec<Injection>, ParseTreeError> {
    let inj_query = lang.query::<Injections>();

    let mut all_injections = ts::query_captures(tree, text, &mut QueryCursor::new(), cx, inj_query, false)
        .filter_map(|QueryCapture { node, index }| {
            let name = inj_query.capture_names()[*index as usize];
            let lang = Language::from_injection_name(name)?;
            let std::ops::Range { start, end } = node.byte_range();

            Some((Ix::<Byte>::new(start)..Ix::new(end), lang))
        }).collect::<Vec<_>>();

    all_injections.sort_unstable_by_key(|i| cmp::Reverse(i.0.start));

    let mut injections = Vec::new();
    if let Some(mut inj) = all_injections.pop() {
        while let Some(next) = all_injections.pop() {
            if next.0.start >= inj.0.end {
                injections.push(mem::replace(&mut inj, next));
            }
        }
        injections.push(inj);
    }

    let mut previous_injections = previous.iter().flat_map(|p| &p.injections).peekable();

    let injections = injections.into_iter().map(|inj @ (range, lang)| {
        let old_tree = loop {
            let Some(&prev) = previous_injections.peek() else { break None };
            if prev.range.start > range.start { break None }
            previous_injections.next();
            if prev.identity() == inj {
                break Some(&prev.tree);
            }
        };

        let tree = parse_meta_tree(Some(cx), lang, text, Some(range), old_tree)?;

        Ok(Injection {
            range,
            lang,
            tree,
        })
    }).collect::<Result<Vec<_>, ParseTreeError>>()?;

    Ok(injections)
}

fn queryless_injections(text: &Rope, previous: Option<&MetaTree>) -> Result<Vec<Injection>, ParseTreeError> {
    previous
        .iter()
        .flat_map(|p| &p.injections)
        .map(|&Injection{ range, lang, ref tree }| {
            Ok(Injection {
                range,
                lang,
                tree: parse_meta_tree(None, lang, text, Some(range), Some(tree))?,
            })
        })
        .collect()
}

fn parse_meta_tree(cx: Option<&QueryCx<'_>>, lang: Language, text: &Rope, range: Option<Range<Ix<Byte>>>, previous: Option<&MetaTree>) -> Result<MetaTree, ParseTreeError> {
    let tree = parse_tree(lang, text, range, previous.as_ref().map(|t| &t.tree))?;

    let injections = if let Some(cx) = cx {
        query_injections(cx, lang, &tree, text, previous)?
    } else {
        queryless_injections(text, previous)?
    };
    
    Ok(MetaTree {
        tree,
        injections,
    })
}

pub struct Injection {
    pub range: Range<Ix<Byte>>,
    pub lang: Language,
    pub tree: MetaTree,
}

impl Injection {
    pub fn identity(&self) -> (Range<Ix<Byte>>, Language) {
        (self.range, self.lang)
    }
}