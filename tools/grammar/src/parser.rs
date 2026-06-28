//! A parser for the book grammar syntax.

use super::{
    ActionLayout, Alternative, Character, Characters, Expression, ExpressionKind, Grammar,
    Production, RangeLimit,
};
use nom::IResult;
use nom::branch::alt;
use nom::bytes::complete::{tag, take_till, take_while, take_while1};
use nom::character::complete::{char, digit1, line_ending, space0};
use nom::combinator::{all_consuming, map, map_res, opt, peek, recognize, value};
use nom::error::{ErrorKind, ParseError};
use nom::multi::{many0, many1, separated_list1};
use nom::sequence::{delimited, pair, preceded, terminated, tuple};
use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

type NomError<'a> = nom::error::Error<&'a str>;
type PResult<'a, T> = IResult<&'a str, T, NomError<'a>>;

static HEADER_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"^\s*(?:@root\s+)?[A-Z][A-Za-z0-9_]*\s*:").unwrap());

#[derive(Debug)]
pub struct Error {
    message: String,
    line: String,
    lineno: usize,
    col: usize,
}

impl Error {
    fn new(input: &str, index: usize, message: impl Into<String>) -> Self {
        let (line, lineno, col) = translate_position(input, index);
        Error {
            message: message.into(),
            line: line.to_string(),
            lineno,
            col,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let lineno = format!("{}", self.lineno);
        let space = " ".repeat(lineno.len() + 1);
        let col = " ".repeat(self.col);
        let line = &self.line;
        let message = &self.message;
        write!(f, "\n{space}|\n{lineno} | {line}\n{space}|{col}^ {message}")
    }
}

impl std::error::Error for Error {}

type Result<T> = std::result::Result<T, Error>;

/// Whether a character can start a grammar rule name.
fn is_name_start(ch: char) -> bool {
    ch.is_alphabetic() || ch == '_' || !ch.is_ascii()
}

/// Whether a character can continue a grammar rule name.
fn is_name_continue(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_' || !ch.is_ascii()
}

fn is_token_name(name: &str) -> bool {
    name.chars()
        .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_')
        && name.chars().any(|ch| ch.is_ascii_uppercase())
}

pub fn parse_grammar(
    input: &str,
    grammar: &mut Grammar,
    category: &str,
    path: &Path,
) -> Result<()> {
    for block in split_productions(input)? {
        let p = parse_production(input, block, category, path)?;
        grammar.name_order.push(p.name.clone());
        if let Some(dupe) = grammar.productions.insert(p.name.clone(), p) {
            return Err(Error::new(
                input,
                block.start,
                format!("duplicate production {} in grammar", dupe.name),
            ));
        }
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct ProductionBlock<'a> {
    start: usize,
    text: &'a str,
}

fn split_productions(input: &str) -> Result<Vec<ProductionBlock<'_>>> {
    let mut blocks = Vec::new();
    let mut current_start = None;
    let mut offset = 0;
    for line in input.split_inclusive('\n') {
        let line_without_newline = line.strip_suffix('\n').unwrap_or(line);
        if HEADER_RE.is_match(line_without_newline) {
            if let Some(start) = current_start.replace(offset) {
                blocks.push(ProductionBlock {
                    start,
                    text: &input[start..offset],
                });
            }
        } else if current_start.is_none()
            && !line_without_newline.trim().is_empty()
            && !line_without_newline.trim_start().starts_with("//")
        {
            return Err(Error::new(input, offset, "expected production header"));
        }
        offset += line.len();
    }

    if let Some(start) = current_start {
        blocks.push(ProductionBlock {
            start,
            text: &input[start..],
        });
    }

    if blocks.is_empty() && !input.trim().is_empty() {
        return Err(Error::new(input, 0, "expected production header"));
    }

    Ok(blocks)
}

fn parse_production(
    source: &str,
    block: ProductionBlock<'_>,
    category: &str,
    path: &Path,
) -> Result<Production> {
    let (body, (is_root, name)) = parse_header(block.text)
        .map_err(|_| Error::new(source, block.start, "expected production header"))?;
    let alternatives = split_alternatives(source, block, body)?;
    let expression = match alternatives.len() {
        0 => return Err(Error::new(source, block.start, "expected an expression")),
        1 => alternatives[0].expression.clone(),
        _ => Expression::new_kind(ExpressionKind::Alt(
            alternatives
                .iter()
                .map(|alternative| alternative.expression.clone())
                .collect(),
        )),
    };

    Ok(Production {
        name,
        comments: Vec::new(),
        category: category.to_string(),
        expression,
        alternatives,
        path: PathBuf::from(path),
        is_root,
    })
}

fn parse_header(input: &str) -> PResult<'_, (bool, String)> {
    let (input, _) = space0(input)?;
    let (input, is_root) =
        map(opt(terminated(tag("@root"), space1)), |root| root.is_some())(input)?;
    let (input, name) = parse_name(input)?;
    let (input, _) = tuple((space0, char(':'), space0))(input)?;
    Ok((input, (is_root, name.to_string())))
}

fn split_alternatives(
    source: &str,
    block: ProductionBlock<'_>,
    body: &str,
) -> Result<Vec<Alternative>> {
    let mut segments = Vec::new();
    let body_start = block.start + (block.text.len() - body.len());
    let mut current_start = None;
    let mut offset = body_start;
    for line in body.split_inclusive('\n') {
        let line_without_newline = line.strip_suffix('\n').unwrap_or(line);
        if line_without_newline.trim_start().starts_with('|') {
            if let Some(start) = current_start.replace(offset) {
                segments.push((start, &source[start..offset]));
            }
        } else if current_start.is_none() && !line_without_newline.trim().is_empty() {
            current_start = Some(offset);
        }
        offset += line.len();
    }
    if let Some(start) = current_start {
        segments.push((start, &source[start..block.start + block.text.len()]));
    }

    let mut alternatives = Vec::new();
    for (start, segment) in segments {
        alternatives.push(parse_alternative(source, start, segment)?);
    }
    Ok(alternatives)
}

fn parse_alternative(source: &str, start: usize, segment: &str) -> Result<Alternative> {
    let segment = segment.trim();
    let segment = segment.strip_prefix('|').map_or(segment, str::trim_start);
    let Some((expr_text, action)) = segment.split_once("=>") else {
        return Err(Error::new(source, start, "expected `=>` action"));
    };
    let action_layout = parse_action_layout(expr_text);
    let expr_text = expr_text.trim();
    let expression = parse_expression_complete(expr_text).map_err(|message| {
        let relative = segment.find(expr_text).unwrap_or(0);
        Error::new(source, start + relative, message)
    })?;
    Ok(Alternative {
        expression,
        action: trim_action(action),
        action_layout,
    })
}

fn parse_action_layout(expr_text: &str) -> ActionLayout {
    let Some((_, before_separator_line)) = expr_text.rsplit_once('\n') else {
        return ActionLayout {
            separator_on_new_line: false,
            separator_indent: String::new(),
        };
    };
    let before_separator_line = before_separator_line
        .strip_suffix('\r')
        .unwrap_or(before_separator_line);
    let separator_on_new_line = before_separator_line.trim().is_empty();
    ActionLayout {
        separator_on_new_line,
        separator_indent: separator_on_new_line
            .then(|| before_separator_line.to_string())
            .unwrap_or_default(),
    }
}

fn trim_action(action: &str) -> String {
    let action = action.trim();
    action
        .strip_suffix(',')
        .map_or(action, str::trim_end)
        .to_string()
}

fn parse_expression_complete(input: &str) -> std::result::Result<Expression, String> {
    match all_consuming(terminated(parse_expression, whitespace0))(input) {
        Ok((_, expression)) => Ok(expression),
        Err(nom::Err::Error(e) | nom::Err::Failure(e)) => {
            let parsed = input.len() - e.input.len();
            Err(format!("failed to parse expression near byte {parsed}"))
        }
        Err(nom::Err::Incomplete(_)) => Err("incomplete expression".to_string()),
    }
}

fn parse_expression(input: &str) -> PResult<'_, Expression> {
    map(
        separated_list1(delimited(space0, char('|'), space0), parse_seq),
        |mut es| {
            if es.len() == 1 {
                es.pop().unwrap()
            } else {
                Expression::new_kind(ExpressionKind::Alt(es))
            }
        },
    )(input)
}

fn parse_seq(input: &str) -> PResult<'_, Expression> {
    map(many1(preceded(space0, parse_expr1)), |mut es| {
        if es.len() == 1 {
            es.pop().unwrap()
        } else {
            Expression::new_kind(ExpressionKind::Sequence(es))
        }
    })(input)
}

fn parse_expr1(input: &str) -> PResult<'_, Expression> {
    let (input, binding) = opt(terminated(parse_name, tuple((space0, char('='), space0))))(input)?;
    let (input, kind) = parse_atom(input)?;
    let (input, kind) = opt(parse_suffix_operator)(input).map(|(input, suffix)| {
        let kind = match suffix {
            Some(Suffix::Optional) => ExpressionKind::Optional(box_kind(kind)),
            Some(Suffix::Repeat) => ExpressionKind::Repeat(box_kind(kind)),
            Some(Suffix::RepeatPlus) => ExpressionKind::RepeatPlus(box_kind(kind)),
            Some(Suffix::RepeatRange {
                name,
                min,
                max,
                limit,
            }) => ExpressionKind::RepeatRange {
                expr: box_kind(kind),
                name,
                min,
                max,
                limit,
            },
            Some(Suffix::RepeatRangeNamed(name)) => {
                ExpressionKind::RepeatRangeNamed(box_kind(kind), name)
            }
            None => kind,
        };
        (input, kind)
    })?;
    let (input, suffix) = parse_subscript_suffix(input)?;
    let (input, footnote) = parse_footnote(input)?;
    Ok((
        input,
        Expression {
            kind,
            binding: binding.map(ToString::to_string),
            suffix,
            footnote,
        },
    ))
}

fn parse_atom(input: &str) -> PResult<'_, ExpressionKind> {
    alt((
        parse_unicode_expression,
        parse_break,
        parse_terminal,
        parse_charset,
        parse_prose,
        parse_grouped,
        parse_neg_expression,
        parse_negative_lookahead,
        parse_cut,
        parse_named_expression,
    ))(input)
}

fn parse_named_expression(input: &str) -> PResult<'_, ExpressionKind> {
    map(parse_name, |name| {
        if is_token_name(name) {
            ExpressionKind::Terminal(name.to_string())
        } else {
            ExpressionKind::Nt(name.to_string())
        }
    })(input)
}

fn parse_terminal(input: &str) -> PResult<'_, ExpressionKind> {
    map(parse_terminal_str, ExpressionKind::Terminal)(input)
}

fn parse_terminal_str(input: &str) -> PResult<'_, String> {
    delimited(
        char('`'),
        map(
            take_while1(|x| !['\n', '`'].contains(&x)),
            ToString::to_string,
        ),
        char('`'),
    )(input)
}

fn parse_charset(input: &str) -> PResult<'_, ExpressionKind> {
    map(
        delimited(
            char('['),
            many0(preceded(space0, parse_characters)),
            preceded(space0, char(']')),
        ),
        ExpressionKind::Charset,
    )(input)
}

fn parse_characters(input: &str) -> PResult<'_, Characters> {
    alt((
        map(
            pair(parse_character, opt(preceded(char('-'), parse_character))),
            |(a, b)| match b {
                Some(b) => Characters::Range(a, b),
                None => Characters::Terminal(a.get_ch().to_string()),
            },
        ),
        map(parse_name, |name| Characters::Named(name.to_string())),
    ))(input)
}

fn parse_character(input: &str) -> PResult<'_, Character> {
    alt((
        map_res(parse_terminal_str, |term| {
            let mut chars = term.chars();
            let Some(ch) = chars.next() else {
                return Err("empty terminal");
            };
            if chars.next().is_some() {
                return Err("range terminal must be one character");
            }
            Ok(Character::Char(ch))
        }),
        map(parse_unicode, Character::Unicode),
    ))(input)
}

fn parse_prose(input: &str) -> PResult<'_, ExpressionKind> {
    map(
        delimited(
            char('<'),
            map(
                take_while1(|x| !['\n', '>'].contains(&x)),
                ToString::to_string,
            ),
            char('>'),
        ),
        ExpressionKind::Prose,
    )(input)
}

fn parse_grouped(input: &str) -> PResult<'_, ExpressionKind> {
    map(
        delimited(
            tuple((char('('), whitespace0)),
            parse_expression,
            tuple((whitespace0, char(')'))),
        ),
        |e| ExpressionKind::Grouped(Box::new(e)),
    )(input)
}

fn parse_neg_expression(input: &str) -> PResult<'_, ExpressionKind> {
    map(preceded(char('~'), parse_atom), |kind| {
        ExpressionKind::NegExpression(box_kind(kind))
    })(input)
}

fn parse_negative_lookahead(input: &str) -> PResult<'_, ExpressionKind> {
    map(preceded(pair(char('!'), space0), parse_expr1), |e| {
        ExpressionKind::NegativeLookahead(Box::new(e))
    })(input)
}

fn parse_cut(input: &str) -> PResult<'_, ExpressionKind> {
    map(preceded(pair(char('^'), space0), parse_seq), |e| {
        ExpressionKind::Cut(Box::new(e))
    })(input)
}

fn parse_unicode_expression(input: &str) -> PResult<'_, ExpressionKind> {
    map(parse_unicode, ExpressionKind::Unicode)(input)
}

fn parse_unicode(input: &str) -> PResult<'_, (char, String)> {
    let (input, _) = tag("U+")(input)?;
    let (input, digits) = recognize(pair(
        take_while_m_n(4, 4, |ch: char| {
            ch.is_ascii_hexdigit() && !ch.is_ascii_lowercase()
        }),
        take_while_m_n(0, 2, |ch: char| {
            ch.is_ascii_hexdigit() && !ch.is_ascii_lowercase()
        }),
    ))(input)?;
    let value = u32::from_str_radix(digits, 16)
        .ok()
        .and_then(char::from_u32)
        .ok_or_else(|| nom::Err::Failure(NomError::from_error_kind(input, ErrorKind::Verify)))?;
    Ok((input, (value, digits.to_string())))
}

fn parse_break(input: &str) -> PResult<'_, ExpressionKind> {
    let (input, _) = line_ending(input)?;
    let (input, _) = peek(take_while1(|ch: char| ch == ' '))(input)?;
    let (input, spaces) = take_while1(|ch: char| ch == ' ')(input)?;
    Ok((input, ExpressionKind::Break(spaces.len())))
}

#[derive(Clone, Debug)]
enum Suffix {
    Optional,
    Repeat,
    RepeatPlus,
    RepeatRange {
        name: Option<String>,
        min: Option<u32>,
        max: Option<u32>,
        limit: RangeLimit,
    },
    RepeatRangeNamed(String),
}

fn parse_suffix_operator(input: &str) -> PResult<'_, Suffix> {
    alt((
        value(Suffix::Optional, char('?')),
        value(Suffix::Repeat, char('*')),
        value(Suffix::RepeatPlus, char('+')),
        parse_repeat_range,
    ))(input)
}

fn parse_repeat_range(input: &str) -> PResult<'_, Suffix> {
    delimited(char('{'), parse_repeat_range_inner, char('}'))(input)
}

fn parse_repeat_range_inner(input: &str) -> PResult<'_, Suffix> {
    let start = input;
    if let Ok((input, name)) = parse_name(input) {
        if peek(char::<_, NomError<'_>>('}'))(input).is_ok() {
            return Ok((input, Suffix::RepeatRangeNamed(name.to_string())));
        }
        if let Ok((input, _)) = char::<_, NomError<'_>>(':')(input) {
            let (input, (min, limit, max)) = parse_range_bounds(input)?;
            return Ok((
                input,
                Suffix::RepeatRange {
                    name: Some(name.to_string()),
                    min,
                    max,
                    limit,
                },
            ));
        }
    }
    let (input, (min, limit, max)) = parse_range_bounds(start)?;
    Ok((
        input,
        Suffix::RepeatRange {
            name: None,
            min,
            max,
            limit,
        },
    ))
}

fn parse_range_bounds(input: &str) -> PResult<'_, (Option<u32>, RangeLimit, Option<u32>)> {
    let (input, min) = opt(map_res(digit1, str::parse::<u32>))(input)?;
    let (input, _) = tag("..")(input)?;
    let (input, limit) = alt((
        value(RangeLimit::Closed, char('=')),
        value(RangeLimit::HalfOpen, nom::combinator::success(())),
    ))(input)?;
    let (input, max) = opt(map_res(digit1, str::parse::<u32>))(input)?;
    Ok((input, (min, limit, max)))
}

fn parse_subscript_suffix(input: &str) -> PResult<'_, Option<String>> {
    opt(delimited(
        tag(" _"),
        map(take_till(|ch| ch == '\n' || ch == '_'), ToString::to_string),
        char('_'),
    ))(input)
}

fn parse_footnote(input: &str) -> PResult<'_, Option<String>> {
    opt(delimited(
        tag("[^"),
        map(
            take_while1(|ch| !['\n', ']'].contains(&ch)),
            ToString::to_string,
        ),
        char(']'),
    ))(input)
}

fn parse_name(input: &str) -> PResult<'_, &str> {
    recognize(pair(
        take_while_m_n(1, 1, is_name_start),
        take_while(is_name_continue),
    ))(input)
}

fn space1(input: &str) -> PResult<'_, &str> {
    take_while1(|ch| ch == ' ' || ch == '\t')(input)
}

fn whitespace0(input: &str) -> PResult<'_, &str> {
    take_while(|ch| ch == ' ' || ch == '\t' || ch == '\n' || ch == '\r')(input)
}

fn take_while_m_n<F>(m: usize, n: usize, f: F) -> impl Fn(&str) -> PResult<'_, &str>
where
    F: Fn(char) -> bool,
{
    move |input| {
        let mut count = 0;
        let mut end = 0;
        for (idx, ch) in input.char_indices() {
            if count == n || !f(ch) {
                break;
            }
            count += 1;
            end = idx + ch.len_utf8();
        }
        if count >= m {
            Ok((&input[end..], &input[..end]))
        } else {
            Err(nom::Err::Error(NomError::from_error_kind(
                input,
                ErrorKind::TakeWhileMN,
            )))
        }
    }
}

fn box_kind(kind: ExpressionKind) -> Box<Expression> {
    Box::new(Expression {
        kind,
        binding: None,
        suffix: None,
        footnote: None,
    })
}

/// Helper to translate a byte index to a `(line, line_no, col_no)` (1-based).
fn translate_position(input: &str, index: usize) -> (&str, usize, usize) {
    if input.is_empty() {
        return ("", 0, 0);
    }
    let index = index.min(input.len());

    let mut line_start = 0;
    let mut line_number = 0;
    for line in input.lines() {
        let line_end = line_start + line.len();
        if index >= line_start && index <= line_end {
            let column_number = index - line_start + 1;
            return (line, line_number + 1, column_number);
        }
        line_start = line_end + 1;
        line_number += 1;
    }
    ("", line_number + 1, 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ExpressionKind;

    fn parse(input: &str) -> crate::Grammar {
        let mut grammar = crate::Grammar::default();
        parse_grammar(input, &mut grammar, "test", Path::new("test.md")).unwrap();
        grammar
    }

    #[test]
    fn parses_single_production() {
        let grammar = parse(
            r#"
Function:
    FunctionQualifiers `fn` IDENTIFIER GenericParams?
        `(` FunctionParameters? `)`
        ( `->` Type )? WhereClause?
        FunctionBody
    => Function { name: IDENTIFIER, return_type: Type }
"#,
        );
        let function = grammar.productions.get("Function").unwrap();
        assert_eq!(function.alternatives.len(), 1);
        assert!(matches!(
            function.expression.kind,
            ExpressionKind::Sequence(_)
        ));
        assert!(function.alternatives[0].action_layout.separator_on_new_line);
        assert_eq!(
            function.alternatives[0].action_layout.separator_indent,
            "    "
        );
    }

    #[test]
    fn parses_multiple_alternatives() {
        let grammar = parse(
            r#"
ItemSafety:
    | `safe` => ItemSafety::Safe,
    | `unsafe` => ItemSafety::Unsafe,
"#,
        );
        let item_safety = grammar.productions.get("ItemSafety").unwrap();
        assert_eq!(item_safety.alternatives.len(), 2);
        assert!(matches!(
            item_safety.expression.kind,
            ExpressionKind::Alt(_)
        ));
        assert_eq!(item_safety.alternatives[0].action, "ItemSafety::Safe");
        assert!(
            !item_safety.alternatives[0]
                .action_layout
                .separator_on_new_line
        );
    }

    #[test]
    fn parses_multiple_productions_in_one_block() {
        let grammar = parse(
            r#"
FunctionParameters:
    first_arg=FunctionParam args=(`,` FunctionParam)* `,`?
    => [first_arg].into_iter().chain(args).collect()

FunctionParam: attrs=OuterAttribute* kind=FunctionParamKind
    => FunctionParam { attrs, kind }
"#,
        );
        assert!(grammar.productions.contains_key("FunctionParameters"));
        assert!(grammar.productions.contains_key("FunctionParam"));
    }

    #[test]
    fn uppercase_names_are_terminals() {
        let grammar = parse("Rule: IDENTIFIER => IDENTIFIER");
        let rule = grammar.productions.get("Rule").unwrap();
        assert!(matches!(
            &rule.alternatives[0].expression.kind,
            ExpressionKind::Terminal(name) if name == "IDENTIFIER"
        ));
    }
}
