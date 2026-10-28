//! Syntax colours for fenced code in an agent's answer.
//!
//! syntect parses each block and reports the scopes covering it; this keeps the
//! scope stack and reduces it to a handful of kinds, so the renderer only has to
//! know how code reads, not what every grammar calls it.

use std::ops::Range;
use std::sync::OnceLock;

use syntect::parsing::{ParseState, ScopeStack, SyntaxSet};

/// What a run of code reads as.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Comment,
    Function,
    Keyword,
    Number,
    Plain,
    Str,
    Type,
}

/// A styled run inside one line: byte range and what it reads as.
pub type Run = (Range<usize>, Kind);

/// The runs of each line of a block, in order. An unknown language is one plain
/// run per line, which is how code rendered before there was colour.
pub fn block(code: &str, language: Option<&str>) -> Vec<Vec<Run>> {
    let lines = code.split('\n').collect::<Vec<_>>();
    let syntax = language.and_then(|language| syntaxes().find_syntax_by_token(language));

    let Some(syntax) = syntax else {
        return plain(lines);
    };

    let mut state = ParseState::new(syntax);
    let mut stack = ScopeStack::new();
    let mut highlighted = Vec::with_capacity(lines.len());

    for line in &lines {
        let mut runs = Vec::new();
        let ops = match state.parse_line(line, syntaxes()) {
            Ok(ops) => ops,
            Err(_) => return plain(lines),
        };

        // The scope carried in from the previous lines sets the opening kind.
        let mut start = 0;
        let mut kind = kind_of(&stack);

        for (offset, op) in ops {
            if stack.apply(&op).is_err() {
                return plain(lines);
            }

            let offset = offset.min(line.len());
            let next = kind_of(&stack);

            if next != kind {
                if offset > start {
                    runs.push((start..offset, kind));
                }

                start = offset;
                kind = next;
            }
        }

        if line.len() > start {
            runs.push((start..line.len(), kind));
        }

        highlighted.push(runs);
    }

    highlighted
}

fn kind_of(stack: &ScopeStack) -> Kind {
    let Some(scope) = stack.as_slice().last() else {
        return Kind::Plain;
    };
    let scope = scope.build_string();

    if scope.starts_with("comment") {
        Kind::Comment
    } else if scope.starts_with("string") {
        Kind::Str
    } else if scope.starts_with("constant.numeric") {
        Kind::Number
    } else if scope.starts_with("entity.name.function") || scope.starts_with("support.function") {
        Kind::Function
    } else if scope.starts_with("keyword") || scope.starts_with("storage") {
        Kind::Keyword
    } else if scope.starts_with("entity.name.type")
        || scope.starts_with("entity.name.class")
        || scope.starts_with("support.type")
        || scope.starts_with("support.class")
    {
        Kind::Type
    } else {
        Kind::Plain
    }
}

fn plain(lines: Vec<&str>) -> Vec<Vec<Run>> {
    lines
        .into_iter()
        .map(|line| {
            if line.is_empty() {
                Vec::new()
            } else {
                vec![(0..line.len(), Kind::Plain)]
            }
        })
        .collect()
}

fn syntaxes() -> &'static SyntaxSet {
    static SYNTAXES: OnceLock<SyntaxSet> = OnceLock::new();

    SYNTAXES.get_or_init(SyntaxSet::load_defaults_newlines)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(runs: &[Run]) -> Vec<Kind> {
        runs.iter().map(|(_, kind)| *kind).collect()
    }

    #[test]
    fn colours_a_rust_block() {
        let runs = block(
            "// adds\nfn main() {\n    let total = 41 + 1;\n}",
            Some("rust"),
        );

        assert_eq!(runs.len(), 4);
        assert!(kinds(&runs[0]).contains(&Kind::Comment));
        assert!(kinds(&runs[1]).contains(&Kind::Keyword));
        assert!(kinds(&runs[2]).contains(&Kind::Keyword));
        assert!(kinds(&runs[2]).contains(&Kind::Number));
    }

    #[test]
    fn colours_a_python_block() {
        let runs = block("def greet():\n    return 'hi'", Some("python"));

        assert!(kinds(&runs[0]).contains(&Kind::Keyword));
        assert!(kinds(&runs[1]).contains(&Kind::Str));
    }

    #[test]
    fn an_unknown_language_is_plain() {
        let runs = block("hola\nmundo", Some("not-a-language"));

        assert_eq!(
            runs,
            vec![vec![(0..4, Kind::Plain)], vec![(0..5, Kind::Plain)]]
        );
        assert_eq!(block("hola", None)[0], vec![(0..4, Kind::Plain)]);
    }

    #[test]
    fn runs_cover_each_line_once() {
        let runs = block("let x = 1;", Some("rust"));
        let mut end = 0;

        for (range, _) in &runs[0] {
            assert_eq!(range.start, end);
            end = range.end;
        }

        assert_eq!(end, "let x = 1;".len());
    }
}
