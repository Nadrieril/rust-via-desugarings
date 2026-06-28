//! Renders the grammar to markdown.

use super::{RenderCtx, require_production};
use crate::{Alternative, Character, Characters, Expression, ExpressionKind, Grammar, Production};
use anyhow::Result;
use std::fmt::Write;

pub fn render_markdown(
    grammar: &Grammar,
    cx: &RenderCtx,
    names: &[&str],
    output: &mut String,
) -> Result<()> {
    let mut iter = names.iter().peekable();
    while let Some(name) = iter.next() {
        let prod = require_production(grammar, name)?;
        render_production(prod, cx, output);
        if iter.peek().is_some() {
            output.push('\n');
        }
    }
    Ok(())
}

pub fn markdown_id(name: &str, for_summary: bool) -> String {
    if for_summary {
        format!("grammar-summary-{name}")
    } else {
        format!("grammar-{name}")
    }
}

fn render_production(prod: &Production, cx: &RenderCtx, output: &mut String) {
    let dest = cx
        .rr_link_map
        .get(&prod.name)
        .map(|path| path.to_string())
        .unwrap_or_else(|| "missing".to_string());
    for expr in &prod.comments {
        render_expression(expr, cx, output);
    }
    write!(
        output,
        "<div class=\"grammar-rule\">\
         <span class=\"grammar-text grammar-production\" id=\"{id}\" \
           onclick=\"show_railroad()\"\
         >\
           <a href=\"{dest}\">{name}</a>\
         </span>:",
        id = markdown_id(&prod.name, cx.for_summary),
        name = prod.name,
        dest = html_escape(&dest),
    )
    .unwrap();
    match prod.alternatives.as_slice() {
        [] => {}
        [alternative] => {
            output.push(' ');
            render_alternative(alternative, cx, output);
        }
        alternatives => {
            output.push_str("<br>\n");
            for (index, alternative) in alternatives.iter().enumerate() {
                output.push_str("<span class=\"grammar-alternative\">&nbsp;&nbsp;&nbsp;&nbsp;| ");
                render_alternative(alternative, cx, output);
                output.push_str("</span>");
                if index + 1 < alternatives.len() {
                    output.push_str("<br>\n");
                }
            }
        }
    }
    output.push_str("</div>\n");
}

fn render_alternative(alternative: &Alternative, cx: &RenderCtx, output: &mut String) {
    render_expression(&alternative.expression, cx, output);
    if alternative.action_layout.separator_on_new_line {
        output.push_str("<span class=\"grammar-action\"><br>\n");
        render_indent(&alternative.action_layout.separator_indent, output);
    } else {
        output.push_str("<span class=\"grammar-action\"> ");
    }
    output.push_str("=&gt; <code class=\"grammar-action-code\">");
    output.push_str(&html_escape(&alternative.action));
    output.push_str("</code></span>");
}

fn render_indent(indent: &str, output: &mut String) {
    for ch in indent.chars() {
        match ch {
            ' ' => output.push_str("&nbsp;"),
            '\t' => output.push_str("&nbsp;&nbsp;&nbsp;&nbsp;"),
            _ => {}
        }
    }
}

fn last_expr(expr: &Expression) -> &ExpressionKind {
    match &expr.kind {
        ExpressionKind::Alt(es) | ExpressionKind::Sequence(es) => last_expr(es.last().unwrap()),
        ExpressionKind::Cut(e) => last_expr(e),
        ExpressionKind::Grouped(_)
        | ExpressionKind::Optional(_)
        | ExpressionKind::NegativeLookahead(_)
        | ExpressionKind::Repeat(_)
        | ExpressionKind::RepeatPlus(_)
        | ExpressionKind::RepeatRange { .. }
        | ExpressionKind::RepeatRangeNamed(_, _)
        | ExpressionKind::Nt(_)
        | ExpressionKind::Terminal(_)
        | ExpressionKind::Prose(_)
        | ExpressionKind::Break(_)
        | ExpressionKind::Comment(_)
        | ExpressionKind::Charset(_)
        | ExpressionKind::NegExpression(_)
        | ExpressionKind::Unicode(_) => &expr.kind,
    }
}

fn render_expression(expr: &Expression, cx: &RenderCtx, output: &mut String) {
    if let Some(binding) = &expr.binding {
        write!(
            output,
            "<span class=\"grammar-binding\">{}=</span>",
            html_escape(binding)
        )
        .unwrap();
    }
    match &expr.kind {
        ExpressionKind::Grouped(e) => {
            output.push_str("( ");
            render_expression(e, cx, output);
            if !matches!(last_expr(e), ExpressionKind::Break(_)) {
                output.push(' ');
            }
            output.push(')');
        }
        ExpressionKind::Alt(es) => {
            let mut iter = es.iter().peekable();
            while let Some(e) = iter.next() {
                render_expression(e, cx, output);
                if iter.peek().is_some() {
                    if !matches!(last_expr(e), ExpressionKind::Break(_)) {
                        output.push(' ');
                    }
                    output.push_str("| ");
                }
            }
        }
        ExpressionKind::Sequence(es) => {
            let mut iter = es.iter().peekable();
            while let Some(e) = iter.next() {
                render_expression(e, cx, output);
                if iter.peek().is_some() && !matches!(last_expr(e), ExpressionKind::Break(_)) {
                    output.push(' ');
                }
            }
        }
        ExpressionKind::Optional(e) => {
            render_expression(e, cx, output);
            output.push_str("<sup>?</sup>");
        }
        ExpressionKind::NegativeLookahead(e) => {
            output.push('!');
            render_expression(e, cx, output);
        }
        ExpressionKind::Repeat(e) => {
            render_expression(e, cx, output);
            output.push_str("<sup>*</sup>");
        }
        ExpressionKind::RepeatPlus(e) => {
            render_expression(e, cx, output);
            output.push_str("<sup>+</sup>");
        }
        ExpressionKind::RepeatRange {
            expr,
            name,
            min,
            max,
            limit,
        } => {
            render_expression(expr, cx, output);
            write!(
                output,
                "<sup>{name}{min}{limit}{max}</sup>",
                name = name.as_ref().map(|n| format!("{n}:")).unwrap_or_default(),
                min = min.map(|v| v.to_string()).unwrap_or_default(),
                max = max.map(|v| v.to_string()).unwrap_or_default(),
            )
            .unwrap();
        }
        ExpressionKind::RepeatRangeNamed(e, name) => {
            render_expression(e, cx, output);
            write!(output, "<sup>{name}</sup>").unwrap();
        }
        ExpressionKind::Nt(nt) => {
            let dest = cx.md_link_map.get(nt).map_or("missing", |d| d.as_str());
            write!(
                output,
                "<span class=\"grammar-text\"><a href=\"{}\">{}</a></span>",
                html_escape(dest),
                html_escape(nt)
            )
            .unwrap();
        }
        ExpressionKind::Terminal(t) => {
            write!(
                output,
                "<span class=\"grammar-literal\">{}</span>",
                html_escape(t)
            )
            .unwrap();
        }
        ExpressionKind::Prose(s) => {
            write!(
                output,
                "<span class=\"grammar-text\">&lt;{}&gt;</span>",
                html_escape(s)
            )
            .unwrap();
        }
        ExpressionKind::Break(indent) => {
            output.push_str("<br>\n");
            output.push_str(&"&nbsp;".repeat(*indent));
        }
        ExpressionKind::Comment(s) => {
            write!(
                output,
                "<span class=\"grammar-comment\">// {}</span>",
                html_escape(s)
            )
            .unwrap();
        }
        ExpressionKind::Charset(set) => charset_render_markdown(cx, set, output),
        ExpressionKind::NegExpression(e) => {
            output.push('~');
            render_expression(e, cx, output);
        }
        ExpressionKind::Cut(e) => {
            output.push_str("^ ");
            render_expression(e, cx, output);
        }
        ExpressionKind::Unicode((_, s)) => {
            output.push_str("U+");
            output.push_str(s);
        }
    }
    if let Some(suffix) = &expr.suffix {
        write!(output, "<sub class=\"grammar-text\">{suffix}</sub>").unwrap();
    }
    if let Some(footnote) = &expr.footnote {
        write!(output, "&ZeroWidthSpace;[^{footnote}]").unwrap();
    }
}

fn charset_render_markdown(cx: &RenderCtx, set: &[Characters], output: &mut String) {
    output.push('[');
    let mut iter = set.iter().peekable();
    while let Some(chars) = iter.next() {
        render_characters(chars, cx, output);
        if iter.peek().is_some() {
            output.push(' ');
        }
    }
    output.push(']');
}

fn render_characters(chars: &Characters, cx: &RenderCtx, output: &mut String) {
    match chars {
        Characters::Named(s) => {
            let dest = cx.md_link_map.get(s).map_or("missing", |d| d.as_str());
            write!(
                output,
                "<a href=\"{}\">{}</a>",
                html_escape(dest),
                html_escape(s)
            )
            .unwrap();
        }
        Characters::Terminal(s) => write!(
            output,
            "<span class=\"grammar-literal\">{}</span>",
            html_escape(s)
        )
        .unwrap(),
        Characters::Range(a, b) => {
            let write_ch = |ch: &Character, output: &mut String| match ch {
                Character::Char(ch) => write!(
                    output,
                    "<span class=\"grammar-literal\">{}</span>",
                    html_escape(&ch.to_string())
                )
                .unwrap(),
                Character::Unicode((_, s)) => write!(output, "U+{s}").unwrap(),
            };
            write_ch(a, output);
            output.push('-');
            write_ch(b, output);
        }
    }
}

fn html_escape(s: &str) -> String {
    let mut escaped = String::new();
    for ch in s.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(ch),
        }
    }
    escaped
}
