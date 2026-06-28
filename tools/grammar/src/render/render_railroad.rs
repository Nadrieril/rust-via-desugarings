//! Converts a [`Grammar`] to SVG railroad diagrams.

use super::{RenderCtx, require_production};
use crate::{Character, Characters, Expression, ExpressionKind, Grammar, Production, RangeLimit};
use anyhow::Result;
use railroad::*;
use regex::Regex;
use std::fmt::Write;
use std::sync::LazyLock;

pub fn render_railroad(
    grammar: &Grammar,
    cx: &RenderCtx,
    names: &[&str],
    output: &mut String,
) -> Result<()> {
    for name in names {
        let prod = require_production(grammar, name)?;
        render_production(prod, cx, output);
    }
    Ok(())
}

pub fn railroad_id(name: &str, for_summary: bool) -> String {
    if for_summary {
        format!("railroad-summary-{name}")
    } else {
        format!("railroad-{name}")
    }
}

fn render_production(prod: &Production, cx: &RenderCtx, output: &mut String) {
    let mut dia = make_diagram(prod, cx, false);
    if dia.width() > 900 {
        dia = make_diagram(prod, cx, true);
    }
    writeln!(
        output,
        "<div style=\"width: {width}px; height: auto; max-width: 100%; max-height: 100%\" \
                class=\"railroad-production\" \
                id=\"{id}\">{dia}</div>",
        width = dia.width(),
        id = railroad_id(&prod.name, cx.for_summary),
    )
    .unwrap();
}

fn make_diagram(prod: &Production, cx: &RenderCtx, stack: bool) -> Diagram<Box<dyn Node>> {
    let n = render_expression(&prod.expression, cx, stack);
    let dest = cx
        .md_link_map
        .get(&prod.name)
        .map(|path| path.to_string())
        .unwrap_or_else(|| "missing".to_string());
    let seq: Sequence<Box<dyn Node>> = Sequence::new(vec![
        Box::new(SimpleStart),
        n.unwrap_or_else(|| Box::new(railroad::Empty)),
        Box::new(SimpleEnd),
    ]);
    let vert = VerticalGrid::<Box<dyn Node>>::new(vec![
        Box::new(Link::new(Comment::new(prod.name.clone()), dest)),
        Box::new(seq),
    ]);

    Diagram::new(Box::new(vert))
}

fn render_expression(expr: &Expression, cx: &RenderCtx, stack: bool) -> Option<Box<dyn Node>> {
    let mut state;
    let mut state_ref = &expr.kind;
    let n: Box<dyn Node> = 'l: loop {
        state_ref = 'cont: {
            break 'l match state_ref {
                ExpressionKind::Grouped(e)
                | ExpressionKind::RepeatRange {
                    expr: e,
                    name: _,
                    min: Some(1),
                    max: Some(1),
                    limit: RangeLimit::Closed,
                } => render_expression(e, cx, stack)?,
                ExpressionKind::Alt(es) => {
                    let choices: Vec<_> = es
                        .iter()
                        .map(|e| render_expression(e, cx, stack))
                        .filter_map(|n| n)
                        .collect();
                    Box::new(Choice::<Box<dyn Node>>::new(choices))
                }
                ExpressionKind::Sequence(es) => {
                    let es: Vec<_> = es.iter().collect();
                    let make_seq = |es: &[&Expression]| {
                        let seq: Vec<_> = es
                            .iter()
                            .map(|e| render_expression(e, cx, stack))
                            .filter_map(|n| n)
                            .collect();
                        if seq.is_empty() {
                            return None;
                        }
                        let seq: Sequence<Box<dyn Node>> = Sequence::new(seq);
                        Some(Box::new(seq) as Box<dyn Node>)
                    };

                    if stack {
                        let es = if matches!(es.first(), Some(e) if e.is_break()) {
                            &es[1..]
                        } else {
                            &es[..]
                        };
                        let es = if matches!(es.last(), Some(e) if e.is_break()) {
                            &es[..es.len() - 1]
                        } else {
                            &es[..]
                        };

                        let mut breaks: Vec<_> =
                            es.split(|e| e.is_break()).flat_map(make_seq).collect();
                        match breaks.len() {
                            0 => return None,
                            1 => breaks.pop().unwrap(),
                            _ => Box::new(Stack::new(breaks)),
                        }
                    } else {
                        make_seq(&es)?
                    }
                }
                ExpressionKind::NegativeLookahead(e) => {
                    let forward = render_expression(e, cx, stack)?;
                    Box::new(LabeledBox::new(
                        forward,
                        Comment::new("not followed by".to_string()),
                    ))
                }
                ExpressionKind::Optional(e)
                | ExpressionKind::RepeatRange {
                    expr: e,
                    name: _,
                    min: None | Some(0),
                    max: Some(1),
                    limit: RangeLimit::Closed,
                } => {
                    let n = render_expression(e, cx, stack)?;
                    Box::new(Optional::new(n))
                }
                ExpressionKind::Repeat(e)
                | ExpressionKind::RepeatRange {
                    expr: e,
                    name: _,
                    min: None | Some(0),
                    max: None,
                    limit: RangeLimit::HalfOpen,
                } => {
                    let n = render_expression(e, cx, stack)?;
                    Box::new(Optional::new(Repeat::new(n, railroad::Empty)))
                }
                ExpressionKind::RepeatPlus(e)
                | ExpressionKind::RepeatRange {
                    expr: e,
                    name: _,
                    min: Some(1),
                    max: None,
                    limit: RangeLimit::HalfOpen,
                } => {
                    let n = render_expression(e, cx, stack)?;
                    Box::new(Repeat::new(n, railroad::Empty))
                }
                ExpressionKind::RepeatRange { max: Some(0), .. }
                | ExpressionKind::RepeatRange {
                    max: Some(1),
                    limit: RangeLimit::HalfOpen,
                    ..
                } => Box::new(railroad::Empty),
                ExpressionKind::RepeatRange {
                    expr: e,
                    name: _,
                    min: None | Some(0),
                    max: Some(b @ 2..),
                    limit,
                } => {
                    state = ExpressionKind::Optional(Box::new(Expression::new_kind(
                        ExpressionKind::RepeatRange {
                            expr: e.clone(),
                            name: None,
                            min: Some(1),
                            max: Some(*b),
                            limit: *limit,
                        },
                    )));
                    break 'cont &state;
                }
                ExpressionKind::RepeatRange {
                    expr: e,
                    name: _,
                    min: Some(1),
                    max: Some(b @ 2..),
                    limit,
                } => {
                    let n = render_expression(e, cx, stack)?;
                    let more = match limit {
                        RangeLimit::HalfOpen => b - 2,
                        RangeLimit::Closed => b - 1,
                    };
                    Box::new(Repeat::new(
                        n,
                        Comment::new(format!("at most {more} more times")),
                    ))
                }
                ExpressionKind::RepeatRange {
                    min: Some(a),
                    max: Some(b),
                    limit: RangeLimit::HalfOpen,
                    ..
                } if b <= a => Box::new(railroad::Empty),
                ExpressionKind::RepeatRange {
                    expr: e,
                    name: _,
                    min: Some(a @ 2..),
                    max: b @ None,
                    limit,
                }
                | ExpressionKind::RepeatRange {
                    expr: e,
                    name: _,
                    min: Some(a @ 2..),
                    max: b @ Some(_),
                    limit,
                } => {
                    let mut es = Vec::<Expression>::new();
                    for _ in 0..(a - 1) {
                        es.push(*e.clone());
                    }
                    es.push(Expression::new_kind(ExpressionKind::RepeatRange {
                        expr: e.clone(),
                        name: None,
                        min: Some(1),
                        max: b.map(|x| x - (a - 1)),
                        limit: *limit,
                    }));
                    state = ExpressionKind::Sequence(es);
                    break 'cont &state;
                }
                ExpressionKind::RepeatRange {
                    max: None,
                    limit: RangeLimit::Closed,
                    ..
                } => unreachable!("closed range must have upper bound"),
                ExpressionKind::RepeatRangeNamed(e, name) => {
                    let n = render_expression(e, cx, stack)?;
                    Box::new(LabeledBox::new(
                        n,
                        Comment::new(format!("repeat exactly {name} times")),
                    ))
                }
                ExpressionKind::Nt(nt) => node_for_nt(cx, nt),
                ExpressionKind::Terminal(t) => Box::new(Terminal::new(t.clone())),
                ExpressionKind::Prose(s) => Box::new(Terminal::new(s.clone())),
                ExpressionKind::Break(_) | ExpressionKind::Comment(_) => return None,
                ExpressionKind::Charset(set) => {
                    let ns: Vec<_> = set.iter().map(|c| render_characters(c, cx)).collect();
                    Box::new(Choice::<Box<dyn Node>>::new(ns))
                }
                ExpressionKind::NegExpression(e) => {
                    let n = render_expression(e, cx, stack)?;
                    let ch = node_for_nt(cx, "CHAR");
                    Box::new(Except::new(Box::new(ch), n))
                }
                ExpressionKind::Cut(e) => {
                    let rhs = render_expression(e, cx, stack)?;
                    Box::new(LabeledBox::new(
                        rhs,
                        Comment::new("no backtracking".to_string()),
                    ))
                }
                ExpressionKind::Unicode((_, s)) => Box::new(Terminal::new(format!("U+{s}"))),
            };
        }
    };
    let n = if let ExpressionKind::RepeatRange {
        name: Some(ref name),
        ..
    } = expr.kind
    {
        Box::new(LabeledBox::new(
            n,
            Comment::new(format!("repeat count {name}")),
        )) as Box<dyn Node>
    } else {
        n
    };
    let n = if let Some(binding) = &expr.binding {
        Box::new(LabeledBox::new(n, Comment::new(binding.clone()))) as Box<dyn Node>
    } else {
        n
    };
    if let Some(suffix) = &expr.suffix {
        let suffix = strip_markdown(suffix);
        return Some(Box::new(LabeledBox::new(n, Comment::new(suffix))));
    }
    Some(n)
}

fn render_characters(chars: &Characters, cx: &RenderCtx) -> Box<dyn Node> {
    match chars {
        Characters::Named(s) => node_for_nt(cx, s),
        Characters::Terminal(s) => Box::new(Terminal::new(s.clone())),
        Characters::Range(a, b) => {
            let mut s = String::new();
            let write_ch = |ch: &Character, output: &mut String| match ch {
                Character::Char(ch) => output.push(*ch),
                Character::Unicode((_, s)) => write!(output, "U+{s}").unwrap(),
            };
            write_ch(a, &mut s);
            s.push('-');
            write_ch(b, &mut s);
            Box::new(Terminal::new(s))
        }
    }
}

fn node_for_nt(cx: &RenderCtx, name: &str) -> Box<dyn Node> {
    let dest = cx
        .rr_link_map
        .get(name)
        .map(|path| path.to_string())
        .unwrap_or_else(|| "missing".to_string());
    Box::new(Link::new(NonTerminal::new(name.to_string()), dest))
}

fn strip_markdown(s: &str) -> String {
    static LINK_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?s)\[([^\]]+)\](?:\[[^\]]*\]|\([^)]*\))?").unwrap());
    LINK_RE.replace_all(s, "$1").to_string()
}

struct Except {
    inner: LabeledBox<Box<dyn Node>, Box<dyn Node>>,
}

impl Except {
    fn new(inner: Box<dyn Node>, label: Box<dyn Node>) -> Self {
        let grid = Box::new(VerticalGrid::new(vec![
            Box::new(Comment::new("with the exception of".to_owned())) as Box<dyn Node>,
            label,
        ])) as Box<dyn Node>;
        let mut this = Self {
            inner: LabeledBox::new(inner, grid),
        };
        this.inner
            .attr("class".to_owned())
            .or_default()
            .push_str(" exceptbox");
        this
    }
}

impl Node for Except {
    fn entry_height(&self) -> i64 {
        self.inner.entry_height()
    }

    fn height(&self) -> i64 {
        self.inner.height()
    }

    fn width(&self) -> i64 {
        self.inner.width()
    }

    fn draw(&self, x: i64, y: i64, h_dir: svg::HDir) -> svg::Element {
        self.inner.draw(x, y, h_dir)
    }
}
