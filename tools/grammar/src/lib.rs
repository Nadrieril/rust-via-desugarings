//! Support for loading the grammar.
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::sync::LazyLock;

mod parser;
pub mod render;
pub mod rustylr;

#[derive(Debug, Default)]
pub struct Grammar {
    pub productions: HashMap<String, Production>,
    /// The order that the production names were discovered.
    pub name_order: Vec<String>,
}

#[derive(Debug)]
pub struct Production {
    pub name: String,
    /// Comments and breaks that precede the production name.
    pub comments: Vec<Expression>,
    /// Category is from the markdown lang string, and defines how it is
    /// grouped and organized on the summary page.
    pub category: String,
    pub expression: Expression,
    pub alternatives: Vec<Alternative>,
    /// The path to the chapter where this is defined, relative to the book's
    /// `src` directory.
    pub path: PathBuf,
    pub is_root: bool,
}

#[derive(Debug)]
pub struct Alternative {
    pub expression: Expression,
    pub action: String,
    pub action_layout: ActionLayout,
}

#[derive(Debug)]
pub struct ActionLayout {
    pub separator_on_new_line: bool,
    pub separator_indent: String,
}

#[derive(Clone, Debug)]
pub struct Expression {
    pub kind: ExpressionKind,
    /// `foo=` in front of an expression.
    pub binding: Option<String>,
    /// Suffix is the `_foo_` part that is shown as a subscript.
    pub suffix: Option<String>,
    /// A footnote is a markdown footnote link.
    pub footnote: Option<String>,
}

#[derive(Clone, Debug)]
pub enum ExpressionKind {
    /// `( A B C )`
    Grouped(Box<Expression>),
    /// `A | B | C`
    Alt(Vec<Expression>),
    /// `A B C`
    Sequence(Vec<Expression>),
    /// `A?`
    Optional(Box<Expression>),
    /// `!A`
    NegativeLookahead(Box<Expression>),
    /// `A*`
    Repeat(Box<Expression>),
    /// `A+`
    RepeatPlus(Box<Expression>),
    /// `A{2..4}` or `A{2..=4}` or `A{name:2..=4}`
    RepeatRange {
        expr: Box<Expression>,
        name: Option<String>,
        min: Option<u32>,
        max: Option<u32>,
        limit: RangeLimit,
    },
    /// `A{name}`
    RepeatRangeNamed(Box<Expression>, String),
    /// `NonTerminal`
    Nt(String),
    /// `` `string` ``
    Terminal(String),
    /// `<english description>`
    Prose(String),
    /// An LF followed by the given number of spaces.
    ///
    /// Used by the renderer to help format and structure the grammar.
    Break(usize),
    /// `// Single line comment.`
    Comment(String),
    /// ``[`A`-`Z` `_` LF]``
    Charset(Vec<Characters>),
    /// ``~[` ` LF]``
    NegExpression(Box<Expression>),
    /// `^ A B C`
    Cut(Box<Expression>),
    /// `U+0060`
    Unicode((char, String)),
}

#[derive(Copy, Clone, Debug)]
pub enum RangeLimit {
    /// `..`
    HalfOpen,
    /// `..=`
    Closed,
}

impl Display for RangeLimit {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            RangeLimit::HalfOpen => "..",
            RangeLimit::Closed => "..=",
        }
        .fmt(f)
    }
}

#[derive(Clone, Debug)]
pub enum Characters {
    /// `LF`
    Named(String),
    /// `` `_` ``
    Terminal(String),
    /// `` `A`-`Z` ``
    Range(Character, Character),
}

#[derive(Clone, Debug)]
pub enum Character {
    Char(char),
    /// `U+0060`
    ///
    /// The `String` is the hex digits after `U+`.
    Unicode((char, String)),
}

impl Character {
    pub fn get_ch(&self) -> char {
        match self {
            Character::Char(ch) => *ch,
            Character::Unicode((ch, _)) => *ch,
        }
    }
}

impl Display for Character {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Character::Char(ch) => write!(f, "`{ch}`"),
            Character::Unicode((_, s)) => write!(f, "U+{s}"),
        }
    }
}

impl Grammar {
    fn visit_nt(&self, callback: &mut dyn FnMut(&str)) {
        for p in self.productions.values() {
            p.expression.visit_nt(callback);
        }
    }
}

impl Expression {
    pub fn new_kind(kind: ExpressionKind) -> Self {
        Self {
            kind,
            binding: None,
            suffix: None,
            footnote: None,
        }
    }

    fn visit_nt(&self, callback: &mut dyn FnMut(&str)) {
        match &self.kind {
            ExpressionKind::Grouped(e)
            | ExpressionKind::Optional(e)
            | ExpressionKind::NegativeLookahead(e)
            | ExpressionKind::Repeat(e)
            | ExpressionKind::RepeatPlus(e)
            | ExpressionKind::RepeatRange { expr: e, .. }
            | ExpressionKind::RepeatRangeNamed(e, _)
            | ExpressionKind::NegExpression(e)
            | ExpressionKind::Cut(e) => {
                e.visit_nt(callback);
            }
            ExpressionKind::Alt(es) | ExpressionKind::Sequence(es) => {
                for e in es {
                    e.visit_nt(callback);
                }
            }
            ExpressionKind::Nt(nt) => {
                callback(&nt);
            }
            ExpressionKind::Terminal(_)
            | ExpressionKind::Prose(_)
            | ExpressionKind::Break(_)
            | ExpressionKind::Comment(_)
            | ExpressionKind::Unicode(_) => {}
            ExpressionKind::Charset(set) => {
                for ch in set {
                    match ch {
                        Characters::Named(s) => callback(s),
                        Characters::Terminal(_) | Characters::Range(_, _) => {}
                    }
                }
            }
        }
    }

    pub fn is_break(&self) -> bool {
        matches!(self.kind, ExpressionKind::Break(_))
    }
}

pub static GRAMMAR_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?ms)^```grammar(?:,([^\n]+))?\n(.*?)^```").unwrap());

pub fn parse_grammar(
    input: &str,
    grammar: &mut Grammar,
    category: &str,
    path: impl Into<PathBuf>,
) -> Result<(), parser::Error> {
    parser::parse_grammar(input, grammar, category, &path.into())
}

pub fn parse_grammar_blocks(
    markdown: &str,
    grammar: &mut Grammar,
    default_category: &str,
    path: impl Into<PathBuf>,
) -> Result<(), parser::Error> {
    let path = path.into();
    for cap in GRAMMAR_RE.captures_iter(markdown) {
        let category = cap.get(1).map_or(default_category, |m| m.as_str());
        let input = cap.get(2).unwrap().as_str();
        parser::parse_grammar(input, grammar, category, &path)?;
    }
    Ok(())
}

pub fn check_grammar(grammar: &Grammar) {
    check_undefined_nt(grammar);
    check_unexpected_roots(grammar);
}

pub fn load_grammar_from_chapters<'a>(
    chapters: impl IntoIterator<Item = (&'a str, PathBuf)>,
) -> Grammar {
    let mut grammar = Grammar::default();
    for (content, path) in chapters {
        if let Err(e) = parse_grammar_blocks(content, &mut grammar, "syntax", path.clone()) {
            panic!("failed to parse grammar in {path:?}: {e}");
        }
    }

    check_grammar(&grammar);
    grammar
}

/// Checks for nonterminals that are used but not defined.
fn check_undefined_nt(grammar: &Grammar) {
    grammar.visit_nt(&mut |nt| {
        if !grammar.productions.contains_key(nt) {
            panic!("non-terminal `{nt}` is used but not defined");
        }
    });
}

/// This checks that all the grammar roots are what we expect.
///
/// This is intended to help catch any unexpected misspellings, orphaned
/// productions, or general mistakes.
fn check_unexpected_roots(grammar: &Grammar) {
    // `set` starts with every production name.
    let mut set: HashSet<_> = grammar.name_order.iter().map(|s| s.as_str()).collect();
    fn remove(set: &mut HashSet<&str>, grammar: &Grammar, prod: &Production, root_name: &str) {
        prod.expression.visit_nt(&mut |nt| {
            // Leave the root name in the set if we find it recursively.
            if nt == root_name {
                return;
            }
            if !set.remove(nt) {
                return;
            }
            if let Some(nt_prod) = grammar.productions.get(nt) {
                remove(set, grammar, nt_prod, root_name);
            }
        });
    }
    // Walk the productions starting from the root nodes, and remove every
    // non-terminal from `set`. What's left must be the set of roots.
    grammar
        .productions
        .values()
        .filter(|prod| prod.is_root)
        .for_each(|root| {
            remove(&mut set, grammar, root, &root.name);
        });
    let expected: HashSet<_> = grammar
        .productions
        .values()
        .filter_map(|p| p.is_root.then(|| p.name.as_str()))
        .collect();
    if set != expected {
        let new: Vec<_> = set.difference(&expected).collect();
        let removed: Vec<_> = expected.difference(&set).collect();
        if !new.is_empty() {
            panic!(
                "New grammar production detected that is not used in any root-accessible\n\
                 production. If this is expected, mark the production with\n\
                 `@root`. If not, make sure it is spelled correctly and used in\n\
                 another root-accessible production.\n\
                 \n\
                 The new names are: {new:?}\n"
            );
        } else if !removed.is_empty() {
            panic!(
                "Old grammar production root seems to have been removed\n\
                 (it is used in some other production that is root-accessible).\n\
                 If this is expected, remove `@root` from the production.\n\
                 \n\
                 The removed names are: {removed:?}\n"
            );
        } else {
            unreachable!("unexpected");
        }
    }
}
