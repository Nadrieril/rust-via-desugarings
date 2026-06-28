use crate::{Expression, ExpressionKind, Grammar};
use anyhow::{Context, Result, anyhow, bail};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fmt::Write;
use std::path::Path;
use std::sync::LazyLock;

static LEXER_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?ms)^```lexer\n(.*?)^```").unwrap());
static IDENT_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b[A-Za-z_][A-Za-z0-9_]*\b").unwrap());

#[derive(Debug, Default)]
pub struct LexerSpec {
    token_type: Option<String>,
    start: Option<String>,
    tokens: Vec<TokenSpec>,
    token_by_display: HashMap<String, usize>,
    token_by_symbol: HashMap<String, usize>,
    precedence: Vec<String>,
    allow: Vec<String>,
}

#[derive(Debug)]
pub struct TokenSpec {
    pub display: String,
    pub symbol: String,
    pub variant: String,
    pub payload: Option<String>,
}

#[derive(Debug)]
struct TokenBinding {
    symbol: String,
    variable: String,
}

pub fn parse_lexer_blocks(markdown: &str, spec: &mut LexerSpec, path: &Path) -> Result<()> {
    for cap in LEXER_RE.captures_iter(markdown) {
        parse_lexer_block(cap.get(1).unwrap().as_str(), spec)
            .with_context(|| format!("failed to parse lexer block in {}", path.display()))?;
    }
    Ok(())
}

fn parse_lexer_block(input: &str, spec: &mut LexerSpec) -> Result<()> {
    for raw_line in input.lines() {
        let line = raw_line
            .split_once("//")
            .map_or(raw_line, |(line, _)| line)
            .trim();
        if line.is_empty() {
            continue;
        }
        if let Some(rest) = line.strip_prefix("%tokentype") {
            spec.token_type = Some(trim_directive(rest)?.to_string());
        } else if let Some(rest) = line.strip_prefix("%start") {
            spec.start = Some(trim_directive(rest)?.to_string());
        } else if let Some(rest) = line.strip_prefix("%precedence") {
            let terminal = trim_directive(rest)?;
            spec.precedence.push(terminal.to_string());
        } else if let Some(rest) = line.strip_prefix("%allow") {
            spec.allow.push(trim_directive(rest)?.to_string());
        } else {
            parse_token_line(line, spec)?;
        }
    }
    Ok(())
}

fn trim_directive(rest: &str) -> Result<&str> {
    rest.trim()
        .strip_suffix(';')
        .map(str::trim)
        .ok_or_else(|| anyhow!("expected `;` at end of directive"))
}

fn parse_token_line(line: &str, spec: &mut LexerSpec) -> Result<()> {
    let line = line
        .strip_suffix(';')
        .ok_or_else(|| anyhow!("expected `;` at end of token declaration"))?
        .trim();
    let (display, rest) = if let Some(rest) = line.strip_prefix('`') {
        let Some((display, rest)) = rest.split_once('`') else {
            bail!("expected closing backtick in token declaration");
        };
        (display.to_string(), rest.trim())
    } else {
        let Some((display, rest)) = line.split_once(char::is_whitespace) else {
            bail!("expected token enum variant after token display name");
        };
        (display.to_string(), rest.trim())
    };
    if rest.is_empty() {
        bail!("expected token enum variant after token display name");
    }
    let (variant, payload) = if let Some(open) = rest.find('(') {
        let close = rest
            .rfind(')')
            .ok_or_else(|| anyhow!("expected `)` in token declaration"))?;
        (
            rest[..open].trim().to_string(),
            Some(rest[open + 1..close].trim().to_string()),
        )
    } else {
        (rest.to_string(), None)
    };
    let symbol = if is_token_name(&display) {
        display.clone()
    } else {
        upper_snake(variant.trim_end_matches('_'))
    };
    let index = spec.tokens.len();
    if spec
        .token_by_display
        .insert(display.clone(), index)
        .is_some()
    {
        bail!("duplicate token display `{display}`");
    }
    if spec.token_by_symbol.insert(symbol.clone(), index).is_some() {
        bail!("duplicate token symbol `{symbol}`");
    }
    spec.tokens.push(TokenSpec {
        display,
        symbol,
        variant,
        payload,
    });
    Ok(())
}

fn is_token_name(name: &str) -> bool {
    name.chars()
        .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_')
        && name.chars().any(|ch| ch.is_ascii_uppercase())
}

fn upper_snake(name: &str) -> String {
    let mut out = String::new();
    let mut prev_lower = false;
    for ch in name.chars() {
        if ch.is_ascii_uppercase() && prev_lower {
            out.push('_');
        }
        out.push(ch.to_ascii_uppercase());
        prev_lower = ch.is_ascii_lowercase() || ch.is_ascii_digit();
    }
    out
}

impl LexerSpec {
    pub fn rustylr_declarations(&self) -> Result<String> {
        let token_type = self
            .token_type
            .as_deref()
            .ok_or_else(|| anyhow!("missing `%tokentype` in lexer block"))?;
        let start = self
            .start
            .as_deref()
            .ok_or_else(|| anyhow!("missing `%start` in lexer block"))?;
        let mut out = String::new();
        writeln!(out, "%tokentype {token_type};")?;
        writeln!(out, "%start {start};")?;
        writeln!(out)?;
        for token in &self.tokens {
            writeln!(
                out,
                "%token {} {};",
                token.symbol,
                token.rust_match_pattern()
            )?;
        }
        writeln!(out)?;
        for terminal in &self.precedence {
            let symbol = self.resolve_terminal(terminal)?;
            writeln!(out, "%precedence {symbol};")?;
        }
        writeln!(out)?;
        for allow in &self.allow {
            writeln!(out, "%allow {allow};")?;
        }
        Ok(out)
    }

    fn resolve_terminal(&self, terminal: &str) -> Result<&str> {
        let display = terminal
            .strip_prefix('`')
            .and_then(|terminal| terminal.strip_suffix('`'))
            .unwrap_or(terminal);
        self.token_by_display
            .get(display)
            .or_else(|| self.token_by_symbol.get(display))
            .map(|index| self.tokens[*index].symbol.as_str())
            .ok_or_else(|| anyhow!("unknown terminal `{terminal}`"))
    }

    fn token_for_terminal(&self, terminal: &str) -> Result<&TokenSpec> {
        self.token_by_display
            .get(terminal)
            .or_else(|| self.token_by_symbol.get(terminal))
            .map(|index| &self.tokens[*index])
            .ok_or_else(|| anyhow!("unknown terminal `{terminal}`"))
    }
}

impl TokenSpec {
    fn rust_match_pattern(&self) -> String {
        match &self.payload {
            Some(_) => format!("Token::{}(_)", self.variant),
            None => format!("Token::{}", self.variant),
        }
    }

    fn rust_pattern(&self, variable: &str) -> String {
        match &self.payload {
            Some(_) => format!("Token::{}({variable})", self.variant),
            None => format!("Token::{}", self.variant),
        }
    }
}

pub fn render_rustylr(grammar: &Grammar, lexer: &LexerSpec) -> Result<String> {
    let mut out = String::new();
    for name in &grammar.name_order {
        let production = &grammar.productions[name];
        let rust_type = production.rust_type.as_deref().unwrap_or(&production.name);
        writeln!(out, "{}({})", production.name, rust_type)?;
        for (index, alternative) in production.alternatives.iter().enumerate() {
            let mut bindings = Vec::new();
            let rhs = emit_expr(
                &alternative.expression,
                lexer,
                &action_names(&alternative.action),
                true,
                &mut bindings,
            )?;
            let action = render_action(&alternative.action, &bindings, lexer)?;
            if index == 0 {
                writeln!(out, "    : {rhs} {action}")?;
            } else {
                writeln!(out, "    | {rhs} {action}")?;
            }
        }
        writeln!(out, "    ;")?;
        writeln!(out)?;
    }
    Ok(out)
}

fn render_action(action: &str, bindings: &[TokenBinding], lexer: &LexerSpec) -> Result<String> {
    let mut out = String::new();
    out.push_str("{\n");
    let mut seen = HashSet::new();
    for binding in bindings {
        if !seen.insert((binding.symbol.as_str(), binding.variable.as_str())) {
            continue;
        }
        let token = lexer.token_for_terminal(&binding.symbol)?;
        writeln!(
            out,
            "        let {} = {} else {{",
            token.rust_pattern(&binding.variable),
            binding.variable
        )?;
        writeln!(
            out,
            "            unreachable!(\"expected {} token\")",
            binding.symbol
        )?;
        writeln!(out, "        }};")?;
    }
    for line in action.lines() {
        writeln!(out, "        {line}")?;
    }
    out.push_str("    }");
    Ok(out)
}

fn action_names(action: &str) -> HashSet<String> {
    IDENT_RE
        .find_iter(action)
        .map(|m| m.as_str().to_string())
        .collect()
}

fn emit_expr(
    expr: &Expression,
    lexer: &LexerSpec,
    action_names: &HashSet<String>,
    allow_auto_bind: bool,
    bindings: &mut Vec<TokenBinding>,
) -> Result<String> {
    let lifted_binding = (expr.binding.is_none())
        .then(|| lift_wrapped_binding(expr, lexer, action_names))
        .flatten();
    let emit_expr = lifted_binding.as_ref().map_or(expr, |(_, expr)| expr);
    let bind = expr
        .binding
        .clone()
        .or_else(|| lifted_binding.as_ref().map(|(binding, _)| binding.clone()))
        .or_else(|| {
            (allow_auto_bind
                && !matches!(
                    expr.kind,
                    ExpressionKind::Alt(_) | ExpressionKind::Sequence(_)
                ))
            .then(|| meaningful_name(expr, lexer, action_names))
            .flatten()
        })
        .filter(|name| action_names.contains(name));
    let keep_wrapped_terminal = bind.is_some() && is_direct_or_wrapped_terminal(&emit_expr.kind);
    let mut inner_bindings = Vec::new();
    let inner = emit_expr_kind(
        &emit_expr.kind,
        lexer,
        action_names,
        allow_auto_bind && expr.binding.is_none(),
        keep_wrapped_terminal,
        &mut inner_bindings,
    )?;

    let mut code = if let Some(bind) = bind {
        let value_bindings = collect_direct_value_bindings(expr, lexer, &bind)?;
        bindings.extend(value_bindings);
        format!("{bind}={inner}")
    } else {
        bindings.append(&mut inner_bindings);
        inner
    };

    if should_discard(expr, lexer, action_names, keep_wrapped_terminal) {
        code.push('!');
    }
    Ok(code)
}

fn lift_wrapped_binding(
    expr: &Expression,
    lexer: &LexerSpec,
    action_names: &HashSet<String>,
) -> Option<(String, Expression)> {
    match &expr.kind {
        ExpressionKind::Optional(e) => {
            lift_group_binding(e, lexer, action_names).map(|(binding, e)| {
                (
                    binding,
                    Expression {
                        kind: ExpressionKind::Optional(Box::new(e)),
                        binding: None,
                        suffix: expr.suffix.clone(),
                        footnote: expr.footnote.clone(),
                    },
                )
            })
        }
        _ => None,
    }
}

fn lift_group_binding(
    expr: &Expression,
    lexer: &LexerSpec,
    action_names: &HashSet<String>,
) -> Option<(String, Expression)> {
    match &expr.kind {
        ExpressionKind::Grouped(e) => {
            lift_group_binding(e, lexer, action_names).map(|(binding, e)| {
                (
                    binding,
                    Expression {
                        kind: ExpressionKind::Grouped(Box::new(e)),
                        binding: None,
                        suffix: expr.suffix.clone(),
                        footnote: expr.footnote.clone(),
                    },
                )
            })
        }
        ExpressionKind::Sequence(es) => {
            let (binding, es) = lift_sequence_binding(es, lexer, action_names)?;
            Some((
                binding,
                Expression {
                    kind: ExpressionKind::Sequence(es),
                    binding: None,
                    suffix: expr.suffix.clone(),
                    footnote: expr.footnote.clone(),
                },
            ))
        }
        _ => None,
    }
}

fn lift_sequence_binding(
    es: &[Expression],
    lexer: &LexerSpec,
    action_names: &HashSet<String>,
) -> Option<(String, Vec<Expression>)> {
    let mut binding = None;
    let mut lifted = Vec::new();
    for expr in es {
        if let Some(name) = &expr.binding
            && action_names.contains(name)
            && matches!(expr.kind, ExpressionKind::Nt(_))
            && binding.is_none()
        {
            binding = Some(name.clone());
            let mut expr = expr.clone();
            expr.binding = None;
            lifted.push(expr);
            continue;
        }
        if !is_discardable_for_lift(expr, lexer, action_names) {
            return None;
        }
        lifted.push(expr.clone());
    }
    binding.map(|binding| (binding, lifted))
}

fn is_discardable_for_lift(
    expr: &Expression,
    lexer: &LexerSpec,
    action_names: &HashSet<String>,
) -> bool {
    match &expr.kind {
        ExpressionKind::Terminal(terminal) => lexer
            .token_for_terminal(terminal)
            .is_ok_and(|token| !action_names.contains(&token.symbol)),
        ExpressionKind::Break(_) | ExpressionKind::Comment(_) => true,
        ExpressionKind::Grouped(e) => is_discardable_for_lift(e, lexer, action_names),
        ExpressionKind::Sequence(es) => es
            .iter()
            .all(|expr| is_discardable_for_lift(expr, lexer, action_names)),
        _ => false,
    }
}

fn emit_expr_kind(
    kind: &ExpressionKind,
    lexer: &LexerSpec,
    action_names: &HashSet<String>,
    allow_auto_bind: bool,
    keep_wrapped_terminal: bool,
    bindings: &mut Vec<TokenBinding>,
) -> Result<String> {
    match kind {
        ExpressionKind::Grouped(e) => {
            let inner = emit_expr(e, lexer, action_names, false, bindings)?;
            Ok(format!("({inner})"))
        }
        ExpressionKind::Alt(es) => {
            let mut parts = Vec::new();
            for e in es {
                let part = emit_expr(e, lexer, action_names, allow_auto_bind, bindings)?;
                if !part.is_empty() {
                    parts.push(part);
                }
            }
            Ok(parts.join(" | "))
        }
        ExpressionKind::Sequence(es) => {
            let mut parts = Vec::new();
            for e in es {
                let part = emit_expr(e, lexer, action_names, allow_auto_bind, bindings)?;
                if !part.is_empty() {
                    parts.push(part);
                }
            }
            Ok(parts.join(" "))
        }
        ExpressionKind::Optional(e) => {
            let inner = if keep_wrapped_terminal {
                emit_expr_keep_terminal(e, lexer, action_names, bindings)?
            } else {
                emit_expr(e, lexer, action_names, false, bindings)?
            };
            Ok(format!("{inner}?"))
        }
        ExpressionKind::Repeat(e) => {
            let inner = if keep_wrapped_terminal {
                emit_expr_keep_terminal(e, lexer, action_names, bindings)?
            } else {
                emit_expr(e, lexer, action_names, false, bindings)?
            };
            Ok(format!("{inner}*"))
        }
        ExpressionKind::RepeatPlus(e) => {
            let inner = if keep_wrapped_terminal {
                emit_expr_keep_terminal(e, lexer, action_names, bindings)?
            } else {
                emit_expr(e, lexer, action_names, false, bindings)?
            };
            Ok(format!("{inner}+"))
        }
        ExpressionKind::RepeatRange {
            expr,
            name,
            min,
            max,
            limit,
        } => {
            let inner = emit_expr(expr, lexer, action_names, false, bindings)?;
            let name = name
                .as_ref()
                .map(|name| format!("{name}:"))
                .unwrap_or_default();
            let min = min.map(|min| min.to_string()).unwrap_or_default();
            let max = max.map(|max| max.to_string()).unwrap_or_default();
            Ok(format!("{inner}{{{name}{min}{limit}{max}}}"))
        }
        ExpressionKind::RepeatRangeNamed(e, name) => {
            let inner = emit_expr(e, lexer, action_names, false, bindings)?;
            Ok(format!("{inner}{{{name}}}"))
        }
        ExpressionKind::Nt(nt) => Ok(nt.clone()),
        ExpressionKind::Terminal(terminal) => {
            Ok(lexer.token_for_terminal(terminal)?.symbol.clone())
        }
        ExpressionKind::Break(_) | ExpressionKind::Comment(_) => Ok(String::new()),
        ExpressionKind::NegativeLookahead(_)
        | ExpressionKind::Prose(_)
        | ExpressionKind::Charset(_)
        | ExpressionKind::NegExpression(_)
        | ExpressionKind::Cut(_)
        | ExpressionKind::Unicode(_) => {
            bail!("expression kind cannot be converted to RustyLR: {kind:?}")
        }
    }
}

fn emit_expr_keep_terminal(
    expr: &Expression,
    lexer: &LexerSpec,
    action_names: &HashSet<String>,
    bindings: &mut Vec<TokenBinding>,
) -> Result<String> {
    match &expr.kind {
        ExpressionKind::Terminal(terminal) => {
            Ok(lexer.token_for_terminal(terminal)?.symbol.clone())
        }
        _ => emit_expr(expr, lexer, action_names, false, bindings),
    }
}

fn meaningful_name(
    expr: &Expression,
    lexer: &LexerSpec,
    action_names: &HashSet<String>,
) -> Option<String> {
    match &expr.kind {
        ExpressionKind::Nt(name) => Some(name.clone()),
        ExpressionKind::Terminal(name) => {
            let token = lexer.token_for_terminal(name).ok()?;
            (token.payload.is_some() || action_names.contains(&token.symbol))
                .then(|| token.symbol.clone())
        }
        ExpressionKind::Grouped(e)
        | ExpressionKind::Optional(e)
        | ExpressionKind::Repeat(e)
        | ExpressionKind::RepeatPlus(e)
        | ExpressionKind::RepeatRange { expr: e, .. }
        | ExpressionKind::RepeatRangeNamed(e, _) => meaningful_name(e, lexer, action_names),
        ExpressionKind::Sequence(es) => {
            let mut names = es
                .iter()
                .filter_map(|expr| meaningful_name(expr, lexer, action_names));
            let name = names.next()?;
            names.next().is_none().then_some(name)
        }
        ExpressionKind::Alt(_)
        | ExpressionKind::NegativeLookahead(_)
        | ExpressionKind::Prose(_)
        | ExpressionKind::Break(_)
        | ExpressionKind::Comment(_)
        | ExpressionKind::Charset(_)
        | ExpressionKind::NegExpression(_)
        | ExpressionKind::Cut(_)
        | ExpressionKind::Unicode(_) => None,
    }
}

fn collect_direct_value_bindings(
    expr: &Expression,
    lexer: &LexerSpec,
    variable: &str,
) -> Result<Vec<TokenBinding>> {
    match &expr.kind {
        ExpressionKind::Terminal(terminal) => {
            let token = lexer.token_for_terminal(terminal)?;
            Ok(token
                .payload
                .is_some()
                .then(|| TokenBinding {
                    symbol: token.symbol.clone(),
                    variable: variable.to_string(),
                })
                .into_iter()
                .collect())
        }
        ExpressionKind::Grouped(e) => collect_direct_value_bindings(e, lexer, variable),
        ExpressionKind::Sequence(es) => {
            let mut bindings = Vec::new();
            for e in es {
                bindings.extend(collect_direct_value_bindings(e, lexer, variable)?);
            }
            Ok(bindings)
        }
        ExpressionKind::Optional(_)
        | ExpressionKind::Repeat(_)
        | ExpressionKind::RepeatPlus(_)
        | ExpressionKind::RepeatRange { .. }
        | ExpressionKind::RepeatRangeNamed(_, _)
        | ExpressionKind::Alt(_)
        | ExpressionKind::Nt(_)
        | ExpressionKind::NegativeLookahead(_)
        | ExpressionKind::Prose(_)
        | ExpressionKind::Break(_)
        | ExpressionKind::Comment(_)
        | ExpressionKind::Charset(_)
        | ExpressionKind::NegExpression(_)
        | ExpressionKind::Cut(_)
        | ExpressionKind::Unicode(_) => Ok(Vec::new()),
    }
}

fn is_direct_or_wrapped_terminal(kind: &ExpressionKind) -> bool {
    match kind {
        ExpressionKind::Terminal(_) => true,
        ExpressionKind::Optional(e)
        | ExpressionKind::Repeat(e)
        | ExpressionKind::RepeatPlus(e)
        | ExpressionKind::RepeatRange { expr: e, .. }
        | ExpressionKind::RepeatRangeNamed(e, _) => is_direct_or_wrapped_terminal(&e.kind),
        _ => false,
    }
}

fn should_discard(
    expr: &Expression,
    lexer: &LexerSpec,
    action_names: &HashSet<String>,
    force_keep: bool,
) -> bool {
    if force_keep {
        return false;
    }
    if expr.binding.is_some() {
        return false;
    }
    let ExpressionKind::Terminal(terminal) = &expr.kind else {
        return false;
    };
    let Ok(token) = lexer.token_for_terminal(terminal) else {
        return false;
    };
    !action_names.contains(&token.symbol)
}
