//! A small Markdown model for agent answers.
//!
//! It keeps only what a reply needs to read well: paragraphs with inline
//! emphasis, headings, lists, quotes, rules and fenced code. Anything the model
//! writes that this does not model (tables, raw HTML) still reaches the reader,
//! as its own text.

use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};

/// A run of text with the styling Markdown gave it.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Span {
    pub bold: bool,
    pub code: bool,
    pub italic: bool,
    pub link: bool,
    pub strike: bool,
    pub text: String,
}

impl Span {
    fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            ..Self::default()
        }
    }
}

/// What a block is, beyond the text it carries.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BlockKind {
    Code { language: Option<String> },
    Heading { level: u8 },
    ListItem { depth: usize, marker: String },
    Paragraph,
    Quote,
    Rule,
}

/// One top-level block of a document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    pub kind: BlockKind,
    pub spans: Vec<Span>,
}

impl Block {
    /// The block's text, flattened: what a code block or a heading holds.
    pub fn text(&self) -> String {
        self.spans
            .iter()
            .map(|span| span.text.as_str())
            .collect::<String>()
    }
}

/// Parse Markdown into the blocks the chat renders.
pub fn parse(source: &str) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut spans: Vec<Span> = Vec::new();
    let mut kind: Option<BlockKind> = None;
    let mut inline = Inline::default();
    let mut lists: Vec<ListState> = Vec::new();
    let mut code: Option<(Option<String>, String)> = None;
    let mut quote_depth = 0usize;

    let options = Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS;

    for event in Parser::new_ext(source, options) {
        match event {
            Event::Start(tag) => match tag {
                Tag::Paragraph => {
                    kind = Some(BlockKind::Paragraph);
                }
                Tag::Heading { level, .. } => {
                    kind = Some(BlockKind::Heading {
                        level: heading_level(level),
                    });
                }
                Tag::BlockQuote(_) => {
                    quote_depth += 1;
                    kind = Some(BlockKind::Quote);
                }
                Tag::CodeBlock(kind) => {
                    let language = match kind {
                        CodeBlockKind::Fenced(info) => info
                            .split_whitespace()
                            .next()
                            .filter(|language| !language.is_empty())
                            .map(str::to_string),
                        CodeBlockKind::Indented => None,
                    };

                    code = Some((language, String::new()));
                }
                Tag::List(start) => {
                    lists.push(ListState {
                        next: start,
                        seen: 0,
                    });
                }
                Tag::Item => {
                    let depth = lists.len().saturating_sub(1);
                    let marker = match lists.last_mut() {
                        Some(list) => match list.next.as_mut() {
                            Some(next) => {
                                let marker = format!("{next}.");
                                *next += 1;
                                marker
                            }
                            None => "•".to_string(),
                        },
                        None => "•".to_string(),
                    };

                    if let Some(list) = lists.last_mut() {
                        list.seen += 1;
                    }

                    kind = Some(BlockKind::ListItem { depth, marker });
                }
                Tag::Emphasis => inline.italic = true,
                Tag::Strong => inline.bold = true,
                Tag::Strikethrough => inline.strike = true,
                Tag::Link { .. } => inline.link = true,
                Tag::Image { .. } => inline.link = true,
                _ => {}
            },
            Event::End(tag) => match tag {
                TagEnd::Paragraph | TagEnd::Heading(_) | TagEnd::Item => {
                    push_block(&mut blocks, &mut kind, &mut spans, quote_depth);
                }
                TagEnd::BlockQuote(_) => {
                    quote_depth = quote_depth.saturating_sub(1);
                }
                TagEnd::CodeBlock => {
                    if let Some((language, text)) = code.take() {
                        blocks.push(Block {
                            kind: BlockKind::Code { language },
                            spans: vec![Span::new(text.trim_end())],
                        });
                    }
                }
                TagEnd::List(_) => {
                    lists.pop();
                }
                TagEnd::Emphasis => inline.italic = false,
                TagEnd::Strong => inline.bold = false,
                TagEnd::Strikethrough => inline.strike = false,
                TagEnd::Link | TagEnd::Image => inline.link = false,
                _ => {}
            },
            Event::Text(text) => {
                if let Some((_, buffer)) = code.as_mut() {
                    buffer.push_str(&text);
                } else {
                    push_span(&mut spans, &inline, &text);
                }
            }
            Event::Code(text) => {
                let mut code_inline = inline;
                code_inline.code = true;
                push_span(&mut spans, &code_inline, &text);
            }
            Event::SoftBreak | Event::HardBreak => {
                if let Some((_, buffer)) = code.as_mut() {
                    buffer.push('\n');
                } else {
                    push_span(&mut spans, &inline, " ");
                }
            }
            Event::Rule => blocks.push(Block {
                kind: BlockKind::Rule,
                spans: Vec::new(),
            }),
            Event::TaskListMarker(checked) => {
                push_span(&mut spans, &inline, if checked { "☑ " } else { "☐ " });
            }
            _ => {}
        }
    }

    push_block(&mut blocks, &mut kind, &mut spans, quote_depth);

    blocks
}

#[derive(Clone, Copy, Default)]
struct Inline {
    bold: bool,
    code: bool,
    italic: bool,
    link: bool,
    strike: bool,
}

struct ListState {
    next: Option<u64>,
    seen: usize,
}

fn heading_level(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

fn push_span(spans: &mut Vec<Span>, inline: &Inline, text: &str) {
    if text.is_empty() {
        return;
    }

    spans.push(Span {
        bold: inline.bold,
        code: inline.code,
        italic: inline.italic,
        link: inline.link,
        strike: inline.strike,
        text: text.to_string(),
    });
}

fn push_block(
    blocks: &mut Vec<Block>,
    kind: &mut Option<BlockKind>,
    spans: &mut Vec<Span>,
    quote_depth: usize,
) {
    let Some(kind) = kind.take() else {
        return;
    };

    let spans = std::mem::take(spans);

    if spans.is_empty() {
        return;
    }

    let kind = match kind {
        BlockKind::Paragraph if quote_depth > 0 => BlockKind::Quote,
        other => other,
    };

    blocks.push(Block { kind, spans });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(blocks: &[Block]) -> Vec<&BlockKind> {
        blocks.iter().map(|block| &block.kind).collect()
    }

    #[test]
    fn reads_a_plain_answer() {
        let blocks = parse("¡Hola! ¿En qué te ayudo?");

        assert_eq!(
            blocks,
            vec![Block {
                kind: BlockKind::Paragraph,
                spans: vec![Span::new("¡Hola! ¿En qué te ayudo?")],
            }]
        );
    }

    #[test]
    fn keeps_inline_emphasis_and_code() {
        let blocks = parse("Usá `cargo test` y **revisá** el *diff*");
        let spans = &blocks[0].spans;

        assert_eq!(spans[0].text, "Usá ");
        assert!(spans[1].code);
        assert_eq!(spans[1].text, "cargo test");
        assert!(spans[3].bold);
        assert_eq!(spans[3].text, "revisá");
        assert!(spans[5].italic);
    }

    #[test]
    fn reads_a_fenced_code_block_with_its_language() {
        let blocks = parse("Antes\n\n```rust\nfn main() {}\n```\n");

        assert_eq!(
            kinds(&blocks),
            vec![
                &BlockKind::Paragraph,
                &BlockKind::Code {
                    language: Some("rust".to_string())
                }
            ]
        );
        assert_eq!(blocks[1].text(), "fn main() {}");
    }

    #[test]
    fn reads_lists_with_markers() {
        let blocks = parse("- uno\n- dos\n\n1. primero\n2. segundo\n");

        assert_eq!(
            blocks
                .iter()
                .map(|block| match &block.kind {
                    BlockKind::ListItem { marker, .. } => marker.as_str(),
                    _ => "?",
                })
                .collect::<Vec<_>>(),
            vec!["•", "•", "1.", "2."]
        );
    }

    #[test]
    fn reads_headings_quotes_and_rules() {
        let blocks = parse("# Título\n\n> cita\n\n---\n");

        assert_eq!(
            kinds(&blocks),
            vec![
                &BlockKind::Heading { level: 1 },
                &BlockKind::Quote,
                &BlockKind::Rule
            ]
        );
        assert_eq!(blocks[0].text(), "Título");
    }

    #[test]
    fn strings_a_wrapped_paragraph_back_together() {
        let blocks = parse("una línea\nsigue acá");

        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].text(), "una línea sigue acá");
    }
}
